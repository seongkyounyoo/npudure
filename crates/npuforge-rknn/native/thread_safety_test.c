/*
 * RKNN Runtime thread-safety 검증 프로그램.
 *
 * npuforge-node 의 워커 구조가 이 결과에 직접 의존한다.
 *
 *   동시 호출 불가 → 모델당 전용 워커 스레드 + 직렬화, worker_count = 1
 *   동시 호출 가능 → worker_count > 1 허용, NPU 코어 분산 가능
 *
 * RK3576은 NPU 코어가 2개이므로(Core0, Core1) 이론상 2-way 병렬이 가능하다.
 * 실제로 되는지, 되면 어떤 방식으로 되는지를 측정한다.
 *
 * docs/environment-matrix.md §3.1
 * docs/03-DEVELOPMENT-REQUIREMENTS.md §2.4
 *
 * 빌드:
 *   gcc -O2 -Wall -Wextra -o thread_safety_test thread_safety_test.c -lrknnrt -lpthread
 *
 * 사용:
 *   ./thread_safety_test <model.rknn> [iterations_per_thread]
 */

#define _GNU_SOURCE
#include <errno.h>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include <rknn_api.h>

#define MAX_THREADS 8

static double now_ms(void)
{
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1000.0 + ts.tv_nsec / 1.0e6;
}

static uint8_t *read_file(const char *path, size_t *out_len)
{
    FILE *fp = fopen(path, "rb");
    if (!fp) {
        return NULL;
    }
    if (fseek(fp, 0, SEEK_END) != 0) { fclose(fp); return NULL; }
    long size = ftell(fp);
    if (size <= 0) { fclose(fp); return NULL; }
    rewind(fp);

    uint8_t *buf = malloc((size_t)size);
    if (!buf) { fclose(fp); return NULL; }
    size_t got = fread(buf, 1, (size_t)size, fp);
    fclose(fp);
    if (got != (size_t)size) { free(buf); return NULL; }
    *out_len = got;
    return buf;
}

/* ------------------------------------------------------------------ */
/* 공유 상태                                                            */
/* ------------------------------------------------------------------ */

typedef struct {
    int          thread_id;
    rknn_context ctx;          /* 공유 또는 전용. 시나리오에 따라 다르다 */
    uint8_t     *input;
    size_t       input_len;
    uint32_t     n_output;
    int          iterations;

    /* 결과 */
    int     ok_count;
    int     err_count;
    int     last_error;
    double  total_ms;
    double  min_ms;
    double  max_ms;
} worker_arg;

static void *worker(void *p)
{
    worker_arg *a = (worker_arg *)p;
    a->min_ms = 1e18;
    a->max_ms = 0.0;

    rknn_output *outputs = calloc(a->n_output, sizeof(rknn_output));
    if (!outputs) {
        a->err_count = a->iterations;
        a->last_error = -999;
        return NULL;
    }

    for (int i = 0; i < a->iterations; i++) {
        double t0 = now_ms();

        rknn_input in;
        memset(&in, 0, sizeof(in));
        in.index = 0;
        in.type = RKNN_TENSOR_UINT8;
        in.fmt = RKNN_TENSOR_NHWC;
        in.size = (uint32_t)a->input_len;
        in.buf = a->input;
        in.pass_through = 0;

        int rc = rknn_inputs_set(a->ctx, 1, &in);
        if (rc != RKNN_SUCC) { a->err_count++; a->last_error = rc; continue; }

        rc = rknn_run(a->ctx, NULL);
        if (rc != RKNN_SUCC) { a->err_count++; a->last_error = rc; continue; }

        for (uint32_t k = 0; k < a->n_output; k++) {
            memset(&outputs[k], 0, sizeof(rknn_output));
            outputs[k].index = k;
            outputs[k].want_float = 1;
            outputs[k].is_prealloc = 0;
        }

        rc = rknn_outputs_get(a->ctx, a->n_output, outputs, NULL);
        if (rc != RKNN_SUCC) { a->err_count++; a->last_error = rc; continue; }

        rknn_outputs_release(a->ctx, a->n_output, outputs);

        double dt = now_ms() - t0;
        a->total_ms += dt;
        if (dt < a->min_ms) a->min_ms = dt;
        if (dt > a->max_ms) a->max_ms = dt;
        a->ok_count++;
    }

    free(outputs);
    return NULL;
}

/* ------------------------------------------------------------------ */

static int query_io(rknn_context ctx, rknn_input_output_num *io)
{
    return rknn_query(ctx, RKNN_QUERY_IN_OUT_NUM, io, sizeof(*io));
}

static size_t input_size_of(rknn_context ctx)
{
    rknn_tensor_attr attr;
    memset(&attr, 0, sizeof(attr));
    attr.index = 0;
    if (rknn_query(ctx, RKNN_QUERY_INPUT_ATTR, &attr, sizeof(attr)) != RKNN_SUCC) {
        return 0;
    }
    /* uint8 입력 기준. n_elems 가 요소 수다. */
    return attr.n_elems;
}

static void report(const char *title, worker_arg *args, int n, double wall_ms)
{
    int ok = 0, err = 0;
    double sum = 0.0, mn = 1e18, mx = 0.0;
    int last_err = 0;

    for (int i = 0; i < n; i++) {
        ok += args[i].ok_count;
        err += args[i].err_count;
        sum += args[i].total_ms;
        if (args[i].ok_count > 0) {
            if (args[i].min_ms < mn) mn = args[i].min_ms;
            if (args[i].max_ms > mx) mx = args[i].max_ms;
        }
        if (args[i].last_error) last_err = args[i].last_error;
    }

    printf("\n--- %s ---\n", title);
    printf("threads          : %d\n", n);
    printf("ok / err         : %d / %d\n", ok, err);
    if (err) printf("last_error       : %d\n", last_err);
    if (ok) {
        printf("per-call avg     : %.2f ms\n", sum / ok);
        printf("per-call min/max : %.2f / %.2f ms\n", mn, mx);
    }
    printf("wall clock       : %.1f ms\n", wall_ms);
    if (ok && wall_ms > 0) {
        printf("throughput       : %.1f inf/s\n", ok * 1000.0 / wall_ms);
    }
}

/* 시나리오 1: 하나의 context 를 여러 스레드가 공유 */
static void scenario_shared_context(uint8_t *model, size_t model_len,
                                    uint8_t *input, size_t input_len,
                                    uint32_t n_output, int threads, int iters)
{
    rknn_context ctx = 0;
    int rc = rknn_init(&ctx, model, (uint32_t)model_len, 0, NULL);
    if (rc != RKNN_SUCC) {
        printf("\n[시나리오 1] rknn_init 실패: %d\n", rc);
        return;
    }

    worker_arg args[MAX_THREADS];
    pthread_t tid[MAX_THREADS];
    memset(args, 0, sizeof(args));

    double t0 = now_ms();
    for (int i = 0; i < threads; i++) {
        args[i].thread_id = i;
        args[i].ctx = ctx;             /* 같은 context 공유 */
        args[i].input = input;
        args[i].input_len = input_len;
        args[i].n_output = n_output;
        args[i].iterations = iters;
        pthread_create(&tid[i], NULL, worker, &args[i]);
    }
    for (int i = 0; i < threads; i++) pthread_join(tid[i], NULL);
    double wall = now_ms() - t0;

    report("시나리오 1: context 1개를 여러 스레드가 공유", args, threads, wall);
    printf("판정 근거     : err > 0 이면 동일 context 동시 호출 불가\n");

    rknn_destroy(ctx);
}

/* 시나리오 2: 스레드마다 전용 context */
static void scenario_per_thread_context(uint8_t *model, size_t model_len,
                                        uint8_t *input, size_t input_len,
                                        uint32_t n_output, int threads, int iters,
                                        int use_core_split)
{
    rknn_context ctxs[MAX_THREADS];
    memset(ctxs, 0, sizeof(ctxs));

    for (int i = 0; i < threads; i++) {
        int rc = rknn_init(&ctxs[i], model, (uint32_t)model_len, 0, NULL);
        if (rc != RKNN_SUCC) {
            printf("\n[시나리오 2] thread %d 의 rknn_init 실패: %d\n", i, rc);
            for (int k = 0; k < i; k++) rknn_destroy(ctxs[k]);
            return;
        }
        if (use_core_split) {
            /* RK3576은 NPU 코어가 2개다. 스레드를 코어에 번갈아 배정한다. */
            rknn_core_mask mask = (i % 2 == 0) ? RKNN_NPU_CORE_0 : RKNN_NPU_CORE_1;
            int mrc = rknn_set_core_mask(ctxs[i], mask);
            if (mrc != RKNN_SUCC) {
                printf("  [경고] thread %d core_mask 설정 실패: %d\n", i, mrc);
            }
        }
    }

    worker_arg args[MAX_THREADS];
    pthread_t tid[MAX_THREADS];
    memset(args, 0, sizeof(args));

    double t0 = now_ms();
    for (int i = 0; i < threads; i++) {
        args[i].thread_id = i;
        args[i].ctx = ctxs[i];         /* 전용 context */
        args[i].input = input;
        args[i].input_len = input_len;
        args[i].n_output = n_output;
        args[i].iterations = iters;
        pthread_create(&tid[i], NULL, worker, &args[i]);
    }
    for (int i = 0; i < threads; i++) pthread_join(tid[i], NULL);
    double wall = now_ms() - t0;

    report(use_core_split
              ? "시나리오 3: 스레드별 전용 context + NPU 코어 분리"
              : "시나리오 2: 스레드별 전용 context (core AUTO)",
           args, threads, wall);

    for (int i = 0; i < threads; i++) rknn_destroy(ctxs[i]);
}

int main(int argc, char **argv)
{
    if (argc < 2) {
        fprintf(stderr,
                "사용법: %s <model.rknn> [iterations] [sweep_from] [sweep_to]\n"
                "\n"
                "  NPUForge 노드의 worker_count 를 결정하기 위한 검증 도구입니다.\n"
                "  결과를 docs/environment-matrix.md §3.1 에 기록하세요.\n"
                "\n"
                "  sweep_from/to 를 같은 값으로 주면 해당 스레드 수만 측정합니다.\n"
                "  고부하에서 보드가 불안정할 때 임계점을 좁히는 데 사용합니다.\n",
                argv[0]);
        return 2;
    }
    const char *model_path = argv[1];
    int iters = (argc >= 3) ? atoi(argv[2]) : 50;
    if (iters <= 0) iters = 50;

    int sweep_from = (argc >= 4) ? atoi(argv[3]) : 3;
    int sweep_to   = (argc >= 5) ? atoi(argv[4]) : MAX_THREADS;
    if (sweep_from < 1) sweep_from = 1;
    if (sweep_to > MAX_THREADS) sweep_to = MAX_THREADS;
    if (sweep_to < sweep_from) sweep_to = sweep_from;

    /* SDK 버전 먼저 출력 */
    rknn_sdk_version ver;
    memset(&ver, 0, sizeof(ver));

    size_t model_len = 0;
    uint8_t *model = read_file(model_path, &model_len);
    if (!model) {
        fprintf(stderr, "모델 파일을 읽을 수 없습니다: %s (%s)\n",
                model_path, strerror(errno));
        return 1;
    }
    printf("모델            : %s (%zu bytes)\n", model_path, model_len);

    /* 기준 context 로 메타데이터 조회 */
    rknn_context probe = 0;
    int rc = rknn_init(&probe, model, (uint32_t)model_len, 0, NULL);
    if (rc != RKNN_SUCC) {
        fprintf(stderr, "rknn_init 실패: %d\n", rc);
        free(model);
        return 1;
    }

    if (rknn_query(probe, RKNN_QUERY_SDK_VERSION, &ver, sizeof(ver)) == RKNN_SUCC) {
        printf("RKNN api        : %s\n", ver.api_version);
        printf("RKNN driver     : %s\n", ver.drv_version);
    }

    rknn_input_output_num io;
    memset(&io, 0, sizeof(io));
    if (query_io(probe, &io) != RKNN_SUCC) {
        fprintf(stderr, "IN_OUT_NUM 조회 실패\n");
        rknn_destroy(probe); free(model);
        return 1;
    }
    printf("입력/출력 개수  : %u / %u\n", io.n_input, io.n_output);

    size_t input_len = input_size_of(probe);
    if (input_len == 0) {
        fprintf(stderr, "입력 크기 조회 실패\n");
        rknn_destroy(probe); free(model);
        return 1;
    }
    printf("입력 크기       : %zu bytes\n", input_len);
    printf("반복 횟수/스레드: %d\n", iters);
    rknn_destroy(probe);

    /* 더미 입력. 정확성이 아니라 동시성만 본다. */
    uint8_t *input = malloc(input_len);
    if (!input) { free(model); return 1; }
    for (size_t i = 0; i < input_len; i++) input[i] = (uint8_t)(i & 0xff);

    printf("\n================ 단일 스레드 기준선 ================\n");
    scenario_per_thread_context(model, model_len, input, input_len,
                                io.n_output, 1, iters, 0);

    printf("\n================ 동시성 검증 ================\n");
    scenario_shared_context(model, model_len, input, input_len,
                            io.n_output, 2, iters);
    scenario_per_thread_context(model, model_len, input, input_len,
                                io.n_output, 2, iters, 0);
    scenario_per_thread_context(model, model_len, input, input_len,
                                io.n_output, 2, iters, 1);

    // NPU 코어는 2개지만 추론 한 건은 NPU 실행 + CPU 전후처리로 구성된다.
    // 스레드가 코어 수보다 많으면 한 스레드가 CPU 구간에 있는 동안 다른
    // 스레드가 NPU 를 점유할 수 있어 처리량이 더 오를 수 있다.
    // 어디서 꺾이는지가 worker_count 의 최적값이다.
    printf("\n================ 스레드 수 스윕 ================\n");
    printf("(스윕 범위: %d ~ %d 스레드)\n", sweep_from, sweep_to);
    for (int t = sweep_from; t <= sweep_to; t++) {
        scenario_per_thread_context(model, model_len, input, input_len,
                                    io.n_output, t, iters, 0);
        fflush(stdout);
    }

    printf("\n"
           "해석 지침\n"
           "  - 시나리오 1에서 err > 0      → 동일 context 동시 호출 불가.\n"
           "                                  모델당 전용 워커 스레드로 직렬화 필요.\n"
           "  - 시나리오 2가 기준선 대비\n"
           "    throughput 이 거의 2배      → 전용 context 로 2-way 병렬 가능.\n"
           "                                  worker_count = 2 채택 가능.\n"
           "  - 거의 1배에 머무름           → 런타임 내부에서 직렬화됨.\n"
           "                                  worker_count = 1 유지.\n"
           "  - 시나리오 3이 2보다 나음     → 명시적 코어 분리가 유효.\n"
           "  - 4스레드가 2스레드보다 나쁨  → 코어 수(2)를 넘는 워커는 역효과.\n"
           "\n"
           "결과를 docs/environment-matrix.md §3.1 에 기록하세요.\n");

    free(input);
    free(model);
    return 0;
}
