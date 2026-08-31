/* 지속 부하 시험 — 열 거동 비교 전용.
 *
 * `thread_safety_test` 는 스윕 전에 단일/2스레드 기준선을 먼저 돌린다.
 * 열 시험에서는 그 구간이 순수한 낭비이고, 시간으로 끊으면 목표 스레드
 * 수에 도달하지도 못한다. 그래서 부하 전용 프로그램을 분리했다.
 *
 * 이 프로그램이 하는 일:
 *   - 지정한 스레드 수로 곧바로 최대 부하를 건다 (스레드당 전용 context)
 *   - 지정한 시간(초) 동안 유지한다. 반복 횟수가 아니라 시간으로 끊어야
 *     보드마다 처리속도가 달라도 부하 구간 길이가 같다.
 *   - 10초마다 중간 처리량을 출력한다. timeout 으로 죽어도 데이터가 남는다.
 *     (thread_safety_test 는 끝까지 가야 출력해서, 죽으면 아무것도 없다)
 *   - 스레드별 성공/실패 수를 마지막에 요약한다.
 *
 * 빌드:
 *   gcc -O2 -o sustained_load_test sustained_load_test.c -lrknnrt -lpthread
 *
 * 사용법:
 *   ./sustained_load_test <model.rknn> <지속초> <스레드수> [want_float=0]
 *
 * want_float 은 출력 형식을 정한다. 0 이면 모델 원본 dtype(INT8 모델이면
 * int8), 1 이면 런타임이 float32 로 역양자화해 준다. 출력 크기가 4배
 * 차이나므로 네트워크 설계에 직접 영향을 준다.
 */

#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <errno.h>
#include <pthread.h>
#include <time.h>
#include <unistd.h>

#include "rknn_api.h"

#define MAX_THREADS 32

static volatile int g_stop = 0;

static double now_sec(void)
{
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec + ts.tv_nsec / 1e9;
}

static uint8_t *read_file(const char *path, size_t *len)
{
    FILE *f = fopen(path, "rb");
    if (!f) return NULL;
    fseek(f, 0, SEEK_END);
    long n = ftell(f);
    fseek(f, 0, SEEK_SET);
    if (n <= 0) { fclose(f); return NULL; }
    uint8_t *buf = malloc((size_t)n);
    if (!buf) { fclose(f); return NULL; }
    if (fread(buf, 1, (size_t)n, f) != (size_t)n) {
        free(buf); fclose(f); return NULL;
    }
    fclose(f);
    *len = (size_t)n;
    return buf;
}

typedef struct {
    int             want_float;
    int             id;
    uint8_t        *model;
    size_t          model_len;
    uint8_t        *input;
    size_t          input_len;
    uint32_t        n_output;
    /* 결과 */
    unsigned long   ok;
    unsigned long   err;
    double          total_us;
    int             init_failed;
} worker_t;

static void *worker_main(void *arg)
{
    worker_t *w = (worker_t *)arg;

    /* 스레드당 전용 context. 동일 context 동시 호출은 불가하다는 것이
     * 이미 검증되었다. docs/environment-matrix.md §3.1 */
    rknn_context ctx = 0;
    if (rknn_init(&ctx, w->model, (uint32_t)w->model_len, 0, NULL) != RKNN_SUCC) {
        w->init_failed = 1;
        return NULL;
    }

    rknn_input in;
    memset(&in, 0, sizeof(in));
    in.index = 0;
    in.type  = RKNN_TENSOR_UINT8;
    in.fmt   = RKNN_TENSOR_NHWC;
    in.size  = (uint32_t)w->input_len;
    in.buf   = w->input;

    rknn_output outs[16];
    memset(outs, 0, sizeof(outs));
    uint32_t n_out = w->n_output > 16 ? 16 : w->n_output;
    for (uint32_t i = 0; i < n_out; i++) outs[i].want_float = (uint8_t)w->want_float;

    while (!g_stop) {
        double t0 = now_sec();
        if (rknn_inputs_set(ctx, 1, &in) != RKNN_SUCC) { w->err++; continue; }
        if (rknn_run(ctx, NULL) != RKNN_SUCC)          { w->err++; continue; }
        if (rknn_outputs_get(ctx, n_out, outs, NULL) != RKNN_SUCC) { w->err++; continue; }
        rknn_outputs_release(ctx, n_out, outs);
        w->total_us += (now_sec() - t0) * 1e6;
        w->ok++;
    }

    rknn_destroy(ctx);
    return NULL;
}

int main(int argc, char **argv)
{
    if (argc < 4) {
        fprintf(stderr,
            "사용법: %s <model.rknn> <지속초> <스레드수>\n"
            "\n"
            "  열 거동 비교용 지속 부하 도구입니다.\n"
            "  10초마다 중간 처리량을 출력하므로 중간에 종료되어도\n"
            "  그때까지의 데이터가 남습니다.\n", argv[0]);
        return 2;
    }

    const char *model_path = argv[1];
    int duration = atoi(argv[2]);
    int threads  = atoi(argv[3]);
    int want_float = (argc >= 5) ? atoi(argv[4]) : 0;
    if (duration <= 0) duration = 60;
    if (threads   <= 0) threads = 1;
    if (threads > MAX_THREADS) threads = MAX_THREADS;

    size_t model_len = 0;
    uint8_t *model = read_file(model_path, &model_len);
    if (!model) {
        fprintf(stderr, "모델을 읽을 수 없습니다: %s (%s)\n",
                model_path, strerror(errno));
        return 1;
    }

    /* 입력 크기와 출력 개수는 probe context 로 조회한다. */
    rknn_context probe = 0;
    if (rknn_init(&probe, model, (uint32_t)model_len, 0, NULL) != RKNN_SUCC) {
        fprintf(stderr, "rknn_init 실패\n");
        free(model);
        return 1;
    }
    rknn_input_output_num io;
    memset(&io, 0, sizeof(io));
    if (rknn_query(probe, RKNN_QUERY_IN_OUT_NUM, &io, sizeof(io)) != RKNN_SUCC) {
        fprintf(stderr, "IN_OUT_NUM 조회 실패\n");
        rknn_destroy(probe); free(model);
        return 1;
    }
    rknn_tensor_attr attr;
    memset(&attr, 0, sizeof(attr));
    attr.index = 0;
    if (rknn_query(probe, RKNN_QUERY_INPUT_ATTR, &attr, sizeof(attr)) != RKNN_SUCC) {
        fprintf(stderr, "INPUT_ATTR 조회 실패\n");
        rknn_destroy(probe); free(model);
        return 1;
    }
    size_t input_len = attr.n_elems;
    rknn_destroy(probe);

    uint8_t *input = malloc(input_len);
    if (!input) { free(model); return 1; }
    for (size_t i = 0; i < input_len; i++) input[i] = (uint8_t)(i & 0xff);

    printf("모델      : %s (%zu bytes)\n", model_path, model_len);
    printf("입력      : %zu bytes\n", input_len);
    printf("스레드    : %d\n", threads);
    printf("지속      : %d초\n\n", duration);
    fflush(stdout);

    static worker_t w[MAX_THREADS];
    static pthread_t th[MAX_THREADS];
    memset(w, 0, sizeof(w));

    for (int i = 0; i < threads; i++) {
        w[i].id        = i;
        w[i].model     = model;
        w[i].model_len = model_len;
        w[i].input     = input;
        w[i].input_len = input_len;
        w[i].n_output  = io.n_output;
        w[i].want_float = want_float;
    }

    double t_start = now_sec();
    for (int i = 0; i < threads; i++) {
        if (pthread_create(&th[i], NULL, worker_main, &w[i]) != 0) {
            fprintf(stderr, "스레드 %d 생성 실패\n", i);
            g_stop = 1;
            break;
        }
    }

    /* 10초마다 중간 결과. timeout 으로 죽어도 여기까지는 남는다. */
    printf("%8s %10s %10s %8s\n", "elapsed", "total", "inf/s", "err");
    fflush(stdout);

    unsigned long prev_ok = 0;
    double prev_t = t_start;
    while (1) {
        sleep(10);
        double t = now_sec();
        double elapsed = t - t_start;

        unsigned long ok = 0, err = 0;
        for (int i = 0; i < threads; i++) { ok += w[i].ok; err += w[i].err; }

        printf("%7.0fs %10lu %10.1f %8lu\n",
               elapsed, ok, (ok - prev_ok) / (t - prev_t), err);
        fflush(stdout);

        prev_ok = ok;
        prev_t  = t;

        if (elapsed >= duration) break;
    }

    g_stop = 1;
    for (int i = 0; i < threads; i++) pthread_join(th[i], NULL);

    double total_elapsed = now_sec() - t_start;
    unsigned long ok = 0, err = 0;
    double sum_us = 0;
    int init_fail = 0;
    for (int i = 0; i < threads; i++) {
        ok  += w[i].ok;
        err += w[i].err;
        sum_us += w[i].total_us;
        init_fail += w[i].init_failed;
    }

    printf("\n================ 요약 ================\n");
    printf("경과 시간      : %.1f초\n", total_elapsed);
    printf("총 추론        : %lu\n", ok);
    printf("처리량         : %.1f inf/s\n", ok ? ok / total_elapsed : 0.0);
    printf("평균 지연      : %.0f us\n", ok ? sum_us / ok : 0.0);
    printf("오류           : %lu\n", err);
    if (init_fail) printf("init 실패 스레드: %d\n", init_fail);

    free(input);
    free(model);
    return 0;
}
