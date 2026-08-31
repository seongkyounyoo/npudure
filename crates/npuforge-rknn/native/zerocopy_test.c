/*
 * Zero-copy 입출력 경로 실험.
 *
 * 배경:
 *   strace 측정에서 추론 한 건에 커널 ioctl 이 약 80회 발생했고,
 *   8스레드 동시 실행 시 ioctl 당 지연이 5.4배로 늘었다.
 *   드라이버 내부 직렬화가 병목이며, 우리가 줄일 수 있는 것은 호출 횟수다.
 *   docs/discuss.md §6 참조.
 *
 * 가설:
 *   rknn_inputs_set / rknn_outputs_get 이 매 호출마다 버퍼를 할당·매핑·해제한다.
 *   버퍼를 한 번 만들어 재사용하면 per-call ioctl 이 줄어든다.
 *
 * 두 경로를 같은 조건에서 비교한다.
 *
 *   [NORMAL]  rknn_inputs_set -> rknn_run -> rknn_outputs_get -> rknn_outputs_release
 *
 *   [ZEROCOPY]
 *     준비 1회:
 *       rknn_query(NATIVE_INPUT_ATTR / NATIVE_OUTPUT_ATTR)
 *       rknn_create_mem(size)
 *       rknn_set_io_mem(mem, attr)
 *     반복:
 *       memcpy -> mem->virt_addr
 *       rknn_run
 *       mem->virt_addr 에서 결과 읽기
 *     정리 1회:
 *       rknn_destroy_mem
 *
 * 주의:
 *   zero-copy 경로는 모델 네이티브 레이아웃을 그대로 쓴다.
 *   NHWC/NCHW 변환과 양자화를 런타임이 해주지 않으므로
 *   실제 사용 시에는 호출자가 그 형식에 맞춰 채워야 한다.
 *   본 실험은 경로 비용만 비교하므로 더미 데이터를 넣는다.
 *
 * 빌드:
 *   gcc -O2 -Wall -Wextra -o zerocopy_test zerocopy_test.c -lrknnrt -lpthread
 *
 * 사용:
 *   ./zerocopy_test <model.rknn> [iterations] [threads] [mode]
 *     mode 0 = NORMAL, 1 = ZEROCOPY
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

#define MAX_THREADS 16

static double now_us(void)
{
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1.0e6 + ts.tv_nsec / 1.0e3;
}

static uint8_t *read_file(const char *path, size_t *out_len)
{
    FILE *fp = fopen(path, "rb");
    if (!fp) return NULL;
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

typedef struct {
    int          thread_id;
    rknn_context ctx;
    int          mode;          /* 0 NORMAL, 1 ZEROCOPY */
    int          iterations;

    uint8_t     *input;         /* NORMAL 경로용 입력 */
    size_t       input_len;
    uint32_t     n_input;
    uint32_t     n_output;

    /* ZEROCOPY 경로용 */
    rknn_tensor_mem  *in_mem;
    rknn_tensor_mem **out_mem;
    rknn_tensor_attr  in_attr;
    rknn_tensor_attr *out_attr;

    /* 구간별 누적 (us) */
    double t_prepare;   /* NORMAL: inputs_set / ZEROCOPY: memcpy + sync */
    double t_run;
    double t_fetch;     /* NORMAL: outputs_get + release / ZEROCOPY: sync + 읽기 */
    double t_total;

    int ok, err, last_error;
} worker_arg;

static void *worker(void *p)
{
    worker_arg *a = (worker_arg *)p;
    rknn_output *outputs = NULL;

    if (a->mode == 0) {
        outputs = calloc(a->n_output, sizeof(rknn_output));
        if (!outputs) { a->err = a->iterations; return NULL; }
    }

    volatile uint64_t sink = 0;   /* 출력 읽기가 최적화로 제거되지 않게 한다 */

    for (int i = 0; i < a->iterations; i++) {
        double t0 = now_us();
        int rc;

        if (a->mode == 0) {
            /* ---- NORMAL ---- */
            rknn_input in;
            memset(&in, 0, sizeof(in));
            in.index = 0;
            in.type = RKNN_TENSOR_UINT8;
            in.fmt = RKNN_TENSOR_NHWC;
            in.size = (uint32_t)a->input_len;
            in.buf = a->input;
            in.pass_through = 0;

            rc = rknn_inputs_set(a->ctx, 1, &in);
            double t1 = now_us();
            if (rc != RKNN_SUCC) { a->err++; a->last_error = rc; continue; }

            rc = rknn_run(a->ctx, NULL);
            double t2 = now_us();
            if (rc != RKNN_SUCC) { a->err++; a->last_error = rc; continue; }

            for (uint32_t k = 0; k < a->n_output; k++) {
                memset(&outputs[k], 0, sizeof(rknn_output));
                outputs[k].index = k;
                outputs[k].want_float = 0;      /* §5 결론에 따라 0 고정 */
                outputs[k].is_prealloc = 0;
            }
            rc = rknn_outputs_get(a->ctx, a->n_output, outputs, NULL);
            if (rc != RKNN_SUCC) { a->err++; a->last_error = rc; continue; }
            sink += ((uint8_t *)outputs[0].buf)[0];
            rknn_outputs_release(a->ctx, a->n_output, outputs);
            double t3 = now_us();

            a->t_prepare += t1 - t0;
            a->t_run     += t2 - t1;
            a->t_fetch   += t3 - t2;
            a->t_total   += t3 - t0;
        } else {
            /* ---- ZEROCOPY ---- */
            /* 입력 버퍼는 이미 바인딩되어 있다. 내용만 갱신한다. */
            memset(a->in_mem->virt_addr, (int)(i & 0xff), a->in_mem->size);
            rknn_mem_sync(a->ctx, a->in_mem, RKNN_MEMORY_SYNC_TO_DEVICE);
            double t1 = now_us();

            rc = rknn_run(a->ctx, NULL);
            double t2 = now_us();
            if (rc != RKNN_SUCC) { a->err++; a->last_error = rc; continue; }

            for (uint32_t k = 0; k < a->n_output; k++)
                rknn_mem_sync(a->ctx, a->out_mem[k], RKNN_MEMORY_SYNC_FROM_DEVICE);
            sink += ((uint8_t *)a->out_mem[0]->virt_addr)[0];
            double t3 = now_us();

            a->t_prepare += t1 - t0;
            a->t_run     += t2 - t1;
            a->t_fetch   += t3 - t2;
            a->t_total   += t3 - t0;
        }
        a->ok++;
    }

    (void)sink;
    free(outputs);
    return NULL;
}

int main(int argc, char **argv)
{
    if (argc < 2) {
        fprintf(stderr,
                "사용법: %s <model.rknn> [iterations] [threads] [mode]\n"
                "  mode 0 = NORMAL (inputs_set/outputs_get)\n"
                "  mode 1 = ZEROCOPY (create_mem/set_io_mem)\n", argv[0]);
        return 2;
    }
    const char *model_path = argv[1];
    int iters   = (argc >= 3) ? atoi(argv[2]) : 100;
    int threads = (argc >= 4) ? atoi(argv[3]) : 1;
    int mode    = (argc >= 5) ? atoi(argv[4]) : 0;
    if (iters   <= 0) iters = 100;
    if (threads <= 0) threads = 1;
    if (threads > MAX_THREADS) threads = MAX_THREADS;
    mode = mode ? 1 : 0;

    size_t model_len = 0;
    uint8_t *model = read_file(model_path, &model_len);
    if (!model) { fprintf(stderr, "모델 읽기 실패\n"); return 1; }

    rknn_context ctxs[MAX_THREADS];
    memset(ctxs, 0, sizeof(ctxs));
    worker_arg args[MAX_THREADS];
    pthread_t  tid[MAX_THREADS];
    memset(args, 0, sizeof(args));

    rknn_input_output_num io;
    memset(&io, 0, sizeof(io));
    size_t input_len = 0;
    uint8_t *input = NULL;

    for (int i = 0; i < threads; i++) {
        if (rknn_init(&ctxs[i], model, (uint32_t)model_len, 0, NULL) != RKNN_SUCC) {
            fprintf(stderr, "thread %d rknn_init 실패\n", i);
            return 1;
        }
        if (i == 0) {
            rknn_query(ctxs[0], RKNN_QUERY_IN_OUT_NUM, &io, sizeof(io));
            rknn_tensor_attr a0;
            memset(&a0, 0, sizeof(a0));
            a0.index = 0;
            rknn_query(ctxs[0], RKNN_QUERY_INPUT_ATTR, &a0, sizeof(a0));
            input_len = a0.n_elems;
            input = malloc(input_len);
            for (size_t k = 0; k < input_len; k++) input[k] = (uint8_t)(k & 0xff);
        }

        args[i].thread_id = i;
        args[i].ctx = ctxs[i];
        args[i].mode = mode;
        args[i].iterations = iters;
        args[i].input = input;
        args[i].input_len = input_len;
        args[i].n_input = io.n_input;
        args[i].n_output = io.n_output;

        if (mode == 1) {
            /* 네이티브 레이아웃을 조회해 그 크기로 버퍼를 만든다. */
            memset(&args[i].in_attr, 0, sizeof(rknn_tensor_attr));
            args[i].in_attr.index = 0;
            if (rknn_query(ctxs[i], RKNN_QUERY_NATIVE_INPUT_ATTR,
                           &args[i].in_attr, sizeof(rknn_tensor_attr)) != RKNN_SUCC) {
                fprintf(stderr, "NATIVE_INPUT_ATTR 조회 실패\n"); return 1;
            }
            args[i].in_mem = rknn_create_mem(ctxs[i], args[i].in_attr.size_with_stride);
            if (!args[i].in_mem) { fprintf(stderr, "create_mem(input) 실패\n"); return 1; }
            if (rknn_set_io_mem(ctxs[i], args[i].in_mem, &args[i].in_attr) != RKNN_SUCC) {
                fprintf(stderr, "set_io_mem(input) 실패\n"); return 1;
            }

            args[i].out_attr = calloc(io.n_output, sizeof(rknn_tensor_attr));
            args[i].out_mem  = calloc(io.n_output, sizeof(rknn_tensor_mem *));
            for (uint32_t k = 0; k < io.n_output; k++) {
                args[i].out_attr[k].index = k;
                if (rknn_query(ctxs[i], RKNN_QUERY_NATIVE_OUTPUT_ATTR,
                               &args[i].out_attr[k], sizeof(rknn_tensor_attr)) != RKNN_SUCC) {
                    fprintf(stderr, "NATIVE_OUTPUT_ATTR[%u] 조회 실패\n", k); return 1;
                }
                args[i].out_mem[k] = rknn_create_mem(ctxs[i], args[i].out_attr[k].size_with_stride);
                if (!args[i].out_mem[k]) { fprintf(stderr, "create_mem(out %u) 실패\n", k); return 1; }
                if (rknn_set_io_mem(ctxs[i], args[i].out_mem[k], &args[i].out_attr[k]) != RKNN_SUCC) {
                    fprintf(stderr, "set_io_mem(out %u) 실패\n", k); return 1;
                }
            }
        }
    }

    double wall0 = now_us();
    for (int i = 0; i < threads; i++) pthread_create(&tid[i], NULL, worker, &args[i]);
    for (int i = 0; i < threads; i++) pthread_join(tid[i], NULL);
    double wall = now_us() - wall0;

    int ok = 0, err = 0;
    double sp = 0, sr = 0, sf = 0, st = 0;
    for (int i = 0; i < threads; i++) {
        ok += args[i].ok; err += args[i].err;
        sp += args[i].t_prepare; sr += args[i].t_run;
        sf += args[i].t_fetch;   st += args[i].t_total;
    }

    printf("mode=%s threads=%d iterations=%d ok=%d err=%d\n",
           mode ? "ZEROCOPY" : "NORMAL", threads, iters, ok, err);
    printf("wall_ms=%.1f throughput_ips=%.1f\n", wall / 1000.0, ok * 1.0e6 / wall);
    if (ok > 0)
        printf("per_call_us total=%.0f prepare=%.0f run=%.0f fetch=%.0f\n",
               st / ok, sp / ok, sr / ok, sf / ok);
    if (mode == 1)
        printf("native_in_bytes=%u native_out0_bytes=%u\n",
               args[0].in_attr.size_with_stride, args[0].out_attr[0].size_with_stride);

    for (int i = 0; i < threads; i++) {
        if (mode == 1) {
            if (args[i].in_mem) rknn_destroy_mem(ctxs[i], args[i].in_mem);
            for (uint32_t k = 0; k < io.n_output; k++)
                if (args[i].out_mem && args[i].out_mem[k])
                    rknn_destroy_mem(ctxs[i], args[i].out_mem[k]);
            free(args[i].out_mem); free(args[i].out_attr);
        }
        rknn_destroy(ctxs[i]);
    }
    free(input); free(model);
    return 0;
}
