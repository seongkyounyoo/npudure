/*
 * NPU 점유율 판별 실험.
 *
 * 해결하려는 질문:
 *   스레드를 1 -> 8 로 늘렸을 때 처리량이 5.55배 오른 이유가 무엇인가?
 *
 *   가설 A  NPU submission pipeline 이 덜 채워져 있었다.
 *           스레드를 늘리자 NPU 점유율이 올라간 것.
 *   가설 B  호출당 시간의 상당 부분이 CPU 구간(입력 설정/출력 취득)이고,
 *           스레드가 그 CPU 작업을 병렬화한 것.
 *
 * 두 가설은 최적화 방향을 정반대로 이끈다.
 *   A 라면 -> NPU 를 더 잘 먹이는 것(배칭, 큐잉)에 집중
 *   B 라면 -> CPU 전처리/후처리 최적화에 집중
 *
 * 판별 방법:
 *   RKNN_QUERY_PERF_RUN 이 실제 NPU 추론 시간(run_duration, us)을 준다.
 *   이 값은 RKNN_FLAG_COLLECT_PERF_MASK 없이도 조회 가능하며 오버헤드가 없다.
 *   (PERF_DETAIL 은 플래그가 필요하고 프레임률을 떨어뜨린다)
 *
 *   핵심 지표:
 *     NPU 평균 점유 코어 수 = sum(run_duration) / wall_clock
 *
 *   RK3576 은 NPU 코어가 2개다.
 *     이 값이 2.0 에 가까우면  -> NPU 포화. 가설 A 는 이미 한계
 *     1 스레드에서 낮고 8 스레드에서 2.0 에 근접 -> 가설 A 지지
 *     1 스레드에서 이미 높음 -> 가설 B 지지
 *
 * 빌드:
 *   gcc -O2 -Wall -Wextra -o npu_occupancy_test npu_occupancy_test.c -lrknnrt -lpthread
 *
 * 사용:
 *   ./npu_occupancy_test <model.rknn> [iterations] [threads] [core_mode] [want_float]
 *
 * want_float:
 *   1  출력을 float 으로 역양자화 (기본, RKNN 예제의 관행)
 *   0  양자화된 원본 그대로 취득. 역양자화 비용을 제거한다
 *
 * core_mode:
 *   0  CORE_AUTO      런타임이 코어를 선택 (기본)
 *   1  ALTERNATE      스레드를 CORE_0 / CORE_1 에 번갈아 고정
 *   2  CORE_0_1       모든 스레드가 두 코어를 함께 사용
 *   3  CORE_0_ONLY    모든 스레드를 코어 0 에 고정 (대조군)
 *
 * 모드 3 은 대조군이다. 단일 코어 상한을 보면 다른 모드에서
 * 두 코어가 실제로 쓰이는지 판정할 수 있다.
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

/* RK3576 의 NPU 코어 수. /sys/kernel/debug/rknpu/load 로 확인했다. */
#define NPU_CORES 2

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

/* /proc/stat 에서 전체 CPU 사용 jiffies 를 읽는다. idle 을 제외한 합. */
static int read_cpu_jiffies(uint64_t *busy, uint64_t *total)
{
    FILE *fp = fopen("/proc/stat", "r");
    if (!fp) return -1;
    char label[16];
    uint64_t v[10] = {0};
    int n = fscanf(fp, "%15s %lu %lu %lu %lu %lu %lu %lu %lu",
                   label, &v[0], &v[1], &v[2], &v[3], &v[4], &v[5], &v[6], &v[7]);
    fclose(fp);
    if (n < 5) return -1;
    uint64_t idle = v[3] + v[4];   /* idle + iowait */
    uint64_t sum = 0;
    for (int i = 0; i < 8; i++) sum += v[i];
    *busy = sum - idle;
    *total = sum;
    return 0;
}

typedef struct {
    int          thread_id;
    rknn_context ctx;
    uint8_t     *input;
    size_t       input_len;
    uint32_t     n_output;
    int          iterations;
    int          want_float;   /* 1이면 출력을 float 으로 역양자화 */
    uint32_t     out_bytes;    /* 첫 출력의 바이트 수. 역양자화 효과 확인용 */

    /* 구간별 누적 시간 (us) */
    double  t_inputs_set;
    double  t_run;
    double  t_outputs_get;
    double  t_release;
    double  t_total;

    /* RKNN 이 보고한 실제 NPU 시간 누적 (us) */
    int64_t npu_duration;
    int     perf_ok;      /* PERF_RUN 조회 성공 횟수 */

    int ok;
    int err;
    int last_error;
} worker_arg;

static void *worker(void *p)
{
    worker_arg *a = (worker_arg *)p;
    rknn_output *outputs = calloc(a->n_output, sizeof(rknn_output));
    if (!outputs) { a->err = a->iterations; return NULL; }

    for (int i = 0; i < a->iterations; i++) {
        double t0 = now_us();

        rknn_input in;
        memset(&in, 0, sizeof(in));
        in.index = 0;
        in.type = RKNN_TENSOR_UINT8;
        in.fmt = RKNN_TENSOR_NHWC;
        in.size = (uint32_t)a->input_len;
        in.buf = a->input;
        in.pass_through = 0;

        double t1 = now_us();
        int rc = rknn_inputs_set(a->ctx, 1, &in);
        double t2 = now_us();
        if (rc != RKNN_SUCC) { a->err++; a->last_error = rc; continue; }

        rc = rknn_run(a->ctx, NULL);
        double t3 = now_us();
        if (rc != RKNN_SUCC) { a->err++; a->last_error = rc; continue; }

        for (uint32_t k = 0; k < a->n_output; k++) {
            memset(&outputs[k], 0, sizeof(rknn_output));
            outputs[k].index = k;
            outputs[k].want_float = (uint8_t)a->want_float;
            outputs[k].is_prealloc = 0;
        }
        rc = rknn_outputs_get(a->ctx, a->n_output, outputs, NULL);
        double t4 = now_us();
        if (rc != RKNN_SUCC) { a->err++; a->last_error = rc; continue; }
        a->out_bytes = outputs[0].size;

        /* 실제 NPU 추론 시간. outputs_get 이후에만 유효하다. */
        rknn_perf_run perf;
        memset(&perf, 0, sizeof(perf));
        if (rknn_query(a->ctx, RKNN_QUERY_PERF_RUN, &perf, sizeof(perf)) == RKNN_SUCC) {
            a->npu_duration += perf.run_duration;
            a->perf_ok++;
        }

        rknn_outputs_release(a->ctx, a->n_output, outputs);
        double t5 = now_us();

        a->t_inputs_set  += t2 - t1;
        a->t_run         += t3 - t2;
        a->t_outputs_get += t4 - t3;
        a->t_release     += t5 - t4;
        a->t_total       += t5 - t0;
        a->ok++;
    }

    free(outputs);
    return NULL;
}

int main(int argc, char **argv)
{
    if (argc < 2) {
        fprintf(stderr,
                "사용법: %s <model.rknn> [iterations] [threads]\n\n"
                "  스레드 수에 따른 NPU 점유율을 측정해\n"
                "  처리량 증가의 원인이 NPU pipeline occupancy 인지\n"
                "  CPU 구간 병렬화인지 판별합니다.\n",
                argv[0]);
        return 2;
    }
    const char *model_path = argv[1];
    int iters   = (argc >= 3) ? atoi(argv[2]) : 100;
    int threads = (argc >= 4) ? atoi(argv[3]) : 1;
    int core_mode = (argc >= 5) ? atoi(argv[4]) : 0;
    if (core_mode < 0 || core_mode > 3) core_mode = 0;
    int want_float = (argc >= 6) ? atoi(argv[5]) : 1;
    want_float = want_float ? 1 : 0;
    if (iters   <= 0) iters = 100;
    if (threads <= 0) threads = 1;
    if (threads > MAX_THREADS) threads = MAX_THREADS;

    size_t model_len = 0;
    uint8_t *model = read_file(model_path, &model_len);
    if (!model) {
        fprintf(stderr, "모델을 읽을 수 없습니다: %s (%s)\n", model_path, strerror(errno));
        return 1;
    }

    /* 메타데이터 조회용 임시 컨텍스트 */
    rknn_context probe = 0;
    if (rknn_init(&probe, model, (uint32_t)model_len, 0, NULL) != RKNN_SUCC) {
        fprintf(stderr, "rknn_init 실패\n"); free(model); return 1;
    }
    rknn_input_output_num io;
    memset(&io, 0, sizeof(io));
    rknn_query(probe, RKNN_QUERY_IN_OUT_NUM, &io, sizeof(io));
    rknn_tensor_attr attr;
    memset(&attr, 0, sizeof(attr));
    attr.index = 0;
    rknn_query(probe, RKNN_QUERY_INPUT_ATTR, &attr, sizeof(attr));
    size_t input_len = attr.n_elems;
    rknn_destroy(probe);

    uint8_t *input = malloc(input_len);
    if (!input) { free(model); return 1; }
    for (size_t i = 0; i < input_len; i++) input[i] = (uint8_t)(i & 0xff);

    /* 스레드별 전용 context */
    rknn_context ctxs[MAX_THREADS];
    memset(ctxs, 0, sizeof(ctxs));
    static const char *mode_name[] = {"CORE_AUTO", "ALTERNATE", "CORE_0_1", "CORE_0_ONLY"};
    for (int i = 0; i < threads; i++) {
        if (rknn_init(&ctxs[i], model, (uint32_t)model_len, 0, NULL) != RKNN_SUCC) {
            fprintf(stderr, "thread %d rknn_init 실패\n", i);
            for (int k = 0; k < i; k++) rknn_destroy(ctxs[k]);
            free(input); free(model);
            return 1;
        }
        rknn_core_mask mask;
        switch (core_mode) {
            case 1:  mask = (i % 2 == 0) ? RKNN_NPU_CORE_0 : RKNN_NPU_CORE_1; break;
            case 2:  mask = RKNN_NPU_CORE_0_1; break;
            case 3:  mask = RKNN_NPU_CORE_0;   break;
            default: mask = RKNN_NPU_CORE_AUTO; break;
        }
        if (rknn_set_core_mask(ctxs[i], mask) != RKNN_SUCC)
            fprintf(stderr, "thread %d core_mask 설정 실패\n", i);
    }

    worker_arg args[MAX_THREADS];
    pthread_t  tid[MAX_THREADS];
    memset(args, 0, sizeof(args));

    uint64_t cpu_busy0 = 0, cpu_total0 = 0, cpu_busy1 = 0, cpu_total1 = 0;
    read_cpu_jiffies(&cpu_busy0, &cpu_total0);

    double wall0 = now_us();
    for (int i = 0; i < threads; i++) {
        args[i].thread_id  = i;
        args[i].ctx        = ctxs[i];
        args[i].input      = input;
        args[i].input_len  = input_len;
        args[i].n_output   = io.n_output;
        args[i].iterations = iters;
        args[i].want_float = want_float;
        pthread_create(&tid[i], NULL, worker, &args[i]);
    }
    for (int i = 0; i < threads; i++) pthread_join(tid[i], NULL);
    double wall = now_us() - wall0;

    read_cpu_jiffies(&cpu_busy1, &cpu_total1);

    for (int i = 0; i < threads; i++) rknn_destroy(ctxs[i]);

    /* ---- 집계 ---- */
    int ok = 0, err = 0, perf_ok = 0;
    double s_in = 0, s_run = 0, s_out = 0, s_rel = 0, s_tot = 0;
    int64_t npu_us = 0;
    for (int i = 0; i < threads; i++) {
        ok += args[i].ok; err += args[i].err; perf_ok += args[i].perf_ok;
        s_in  += args[i].t_inputs_set;
        s_run += args[i].t_run;
        s_out += args[i].t_outputs_get;
        s_rel += args[i].t_release;
        s_tot += args[i].t_total;
        npu_us += args[i].npu_duration;
    }

    double cpu_pct = 0.0;
    if (cpu_total1 > cpu_total0)
        cpu_pct = 100.0 * (double)(cpu_busy1 - cpu_busy0) / (double)(cpu_total1 - cpu_total0);

    /* 핵심 지표: NPU 가 평균 몇 개 코어만큼 바빴는가 */
    double npu_cores_busy = (wall > 0) ? (double)npu_us / wall : 0.0;

    uint32_t out_bytes = 0;
    for (int i = 0; i < threads; i++)
        if (args[i].out_bytes) { out_bytes = args[i].out_bytes; break; }

    printf("threads=%d iterations=%d core_mode=%d(%s) want_float=%d out0_bytes=%u ok=%d err=%d\n",
           threads, iters, core_mode, mode_name[core_mode], want_float, out_bytes, ok, err);
    printf("wall_ms=%.1f throughput_ips=%.1f\n", wall / 1000.0, ok * 1.0e6 / wall);
    printf("cpu_busy_pct=%.1f\n", cpu_pct);
    printf("perf_query_ok=%d\n", perf_ok);

    if (ok > 0) {
        printf("per_call_us total=%.0f inputs_set=%.0f run=%.0f outputs_get=%.0f release=%.0f\n",
               s_tot / ok, s_in / ok, s_run / ok, s_out / ok, s_rel / ok);
        if (perf_ok > 0) {
            printf("per_call_npu_us=%.0f\n", (double)npu_us / perf_ok);
            printf("npu_share_of_call_pct=%.1f\n",
                   100.0 * ((double)npu_us / perf_ok) / (s_tot / ok));
        }
    }
    printf("npu_total_us=%lld\n", (long long)npu_us);
    printf("npu_cores_busy=%.3f  (코어 %d개 중)\n", npu_cores_busy, NPU_CORES);
    printf("npu_utilization_pct=%.1f\n", 100.0 * npu_cores_busy / NPU_CORES);

    free(input);
    free(model);
    return 0;
}
