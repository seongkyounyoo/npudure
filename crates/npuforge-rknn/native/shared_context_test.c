/* 공유 컨텍스트가 결과를 섞는지 확인한다.
 *
 * `docs/environment-matrix.md` §3.1 의 thread-safety 검증은 반환 코드만
 * 셌고 출력 내용을 대조하지 않았다. "오류 0건"은 "결과가 옳다"가 아니다.
 *
 * 추론 한 건은 세 번의 호출이다.
 *
 *   rknn_inputs_set  →  rknn_run  →  rknn_outputs_get
 *
 * 개별 호출이 thread-safe 여도 시퀀스는 원자적이지 않다. 두 스레드가
 * 같은 컨텍스트에서 이를 겹쳐 실행하면 서로의 결과를 가져갈 수 있다.
 *
 * 이 프로그램은 그것을 실제로 재현한다.
 *
 *   1. 스레드마다 다른 입력을 만든다
 *   2. 먼저 단독으로 추론해 기준 출력을 저장한다
 *   3. 컨텍스트 하나를 공유해 동시에 추론한다
 *   4. 각 스레드의 결과가 자기 기준과 같은지 대조한다
 *
 * 대조군으로 스레드별 전용 컨텍스트도 같은 방식으로 확인한다.
 *
 * 빌드:
 *   gcc -O2 -o shared_context_test shared_context_test.c -lrknnrt -lpthread
 *
 * 사용법:
 *   ./shared_context_test <model.rknn> [threads] [iterations]
 */

#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <pthread.h>

#include "rknn_api.h"

#define MAX_THREADS 16
#define MAX_OUT 32

typedef struct {
    int            id;
    rknn_context   ctx;          /* 공유 모드면 전부 같은 값 */
    uint8_t       *input;
    size_t         input_len;
    uint32_t       n_output;
    int            iterations;
    /* 기준 출력. 첫 텐서만 비교한다 — 섞이면 여기서 이미 드러난다. */
    uint8_t       *baseline;
    size_t         baseline_len;
    /* 결과 */
    int            mismatches;
    int            errors;
} worker_t;

static uint8_t *read_file(const char *p, size_t *len)
{
    FILE *f = fopen(p, "rb");
    if (!f) return NULL;
    fseek(f, 0, SEEK_END); long n = ftell(f); fseek(f, 0, SEEK_SET);
    if (n <= 0) { fclose(f); return NULL; }
    uint8_t *b = malloc((size_t)n);
    if (!b) { fclose(f); return NULL; }
    if (fread(b, 1, (size_t)n, f) != (size_t)n) { free(b); fclose(f); return NULL; }
    fclose(f); *len = (size_t)n; return b;
}

/* 추론 한 건. 첫 출력 텐서를 out 에 복사한다. */
static int infer_once(rknn_context ctx, uint8_t *input, size_t input_len,
                      uint32_t n_output, uint8_t **out, size_t *out_len)
{
    rknn_input in;
    memset(&in, 0, sizeof(in));
    in.index = 0;
    in.type  = RKNN_TENSOR_UINT8;
    in.fmt   = RKNN_TENSOR_NHWC;
    in.size  = (uint32_t)input_len;
    in.buf   = input;

    if (rknn_inputs_set(ctx, 1, &in) != RKNN_SUCC) return -1;
    if (rknn_run(ctx, NULL) != RKNN_SUCC) return -1;

    rknn_output outs[MAX_OUT];
    memset(outs, 0, sizeof(outs));
    for (uint32_t i = 0; i < n_output; i++) {
        outs[i].index = i;
        outs[i].want_float = 1;
    }
    if (rknn_outputs_get(ctx, n_output, outs, NULL) != RKNN_SUCC) return -1;

    if (*out == NULL) {
        *out = malloc(outs[0].size);
        *out_len = outs[0].size;
    }
    if (*out && *out_len == outs[0].size) {
        memcpy(*out, outs[0].buf, outs[0].size);
    }
    rknn_outputs_release(ctx, n_output, outs);
    return 0;
}

static void *worker_main(void *arg)
{
    worker_t *w = (worker_t *)arg;
    uint8_t *buf = malloc(w->baseline_len);
    if (!buf) { w->errors++; return NULL; }
    size_t buf_len = w->baseline_len;

    for (int i = 0; i < w->iterations; i++) {
        uint8_t *p = buf;
        size_t len = buf_len;
        if (infer_once(w->ctx, w->input, w->input_len, w->n_output, &p, &len) != 0) {
            w->errors++;
            continue;
        }
        if (memcmp(buf, w->baseline, w->baseline_len) != 0) {
            w->mismatches++;
        }
    }
    free(buf);
    return NULL;
}

static int run_mode(const char *label, int shared,
                    uint8_t *model, size_t model_len,
                    uint8_t **inputs, size_t input_len,
                    uint32_t n_output, int threads, int iters)
{
    static worker_t w[MAX_THREADS];
    static pthread_t th[MAX_THREADS];
    rknn_context shared_ctx = 0;
    rknn_context own[MAX_THREADS];
    memset(w, 0, sizeof(w));
    memset(own, 0, sizeof(own));

    if (shared) {
        if (rknn_init(&shared_ctx, model, (uint32_t)model_len, 0, NULL) != RKNN_SUCC) {
            fprintf(stderr, "%s: rknn_init 실패\n", label);
            return -1;
        }
    } else {
        for (int i = 0; i < threads; i++) {
            if (rknn_init(&own[i], model, (uint32_t)model_len, 0, NULL) != RKNN_SUCC) {
                fprintf(stderr, "%s: rknn_init[%d] 실패\n", label, i);
                return -1;
            }
        }
    }

    /* 기준 출력을 먼저 단독으로 만든다. 공유 모드에서도 이 단계는
     * 동시성이 없으므로 올바른 값이 나온다. */
    for (int i = 0; i < threads; i++) {
        w[i].id = i;
        w[i].ctx = shared ? shared_ctx : own[i];
        w[i].input = inputs[i];
        w[i].input_len = input_len;
        w[i].n_output = n_output;
        w[i].iterations = iters;
        w[i].baseline = NULL;
        w[i].baseline_len = 0;
        if (infer_once(w[i].ctx, w[i].input, input_len, n_output,
                       &w[i].baseline, &w[i].baseline_len) != 0) {
            fprintf(stderr, "%s: 기준 추론 실패\n", label);
            return -1;
        }
    }

    for (int i = 0; i < threads; i++) pthread_create(&th[i], NULL, worker_main, &w[i]);
    for (int i = 0; i < threads; i++) pthread_join(th[i], NULL);

    int total_mis = 0, total_err = 0;
    for (int i = 0; i < threads; i++) {
        total_mis += w[i].mismatches;
        total_err += w[i].errors;
        free(w[i].baseline);
    }

    int total = threads * iters;
    printf("%-28s 추론 %5d회  API오류 %4d  **결과불일치 %5d (%.1f%%)**\n",
           label, total, total_err, total_mis,
           total ? 100.0 * total_mis / total : 0.0);

    if (shared) rknn_destroy(shared_ctx);
    else for (int i = 0; i < threads; i++) rknn_destroy(own[i]);

    return total_mis;
}

int main(int argc, char **argv)
{
    if (argc < 2) {
        fprintf(stderr, "사용법: %s <model.rknn> [threads=4] [iterations=50]\n", argv[0]);
        return 2;
    }
    int threads = (argc >= 3) ? atoi(argv[2]) : 4;
    int iters   = (argc >= 4) ? atoi(argv[3]) : 50;
    if (threads < 2) threads = 2;
    if (threads > MAX_THREADS) threads = MAX_THREADS;
    if (iters < 1) iters = 1;

    size_t model_len = 0;
    uint8_t *model = read_file(argv[1], &model_len);
    if (!model) { fprintf(stderr, "모델을 읽을 수 없습니다\n"); return 1; }

    rknn_context probe = 0;
    if (rknn_init(&probe, model, (uint32_t)model_len, 0, NULL) != RKNN_SUCC) {
        fprintf(stderr, "rknn_init 실패\n"); return 1;
    }
    rknn_input_output_num io;
    memset(&io, 0, sizeof(io));
    rknn_query(probe, RKNN_QUERY_IN_OUT_NUM, &io, sizeof(io));
    rknn_tensor_attr ia;
    memset(&ia, 0, sizeof(ia));
    ia.index = 0;
    rknn_query(probe, RKNN_QUERY_INPUT_ATTR, &ia, sizeof(ia));
    size_t input_len = ia.n_elems;
    rknn_destroy(probe);

    /* 스레드마다 확실히 다른 입력 */
    uint8_t *inputs[MAX_THREADS];
    for (int i = 0; i < threads; i++) {
        inputs[i] = malloc(input_len);
        for (size_t k = 0; k < input_len; k++) {
            inputs[i][k] = (uint8_t)((k + i * 37 + i * i * 11) % 251);
        }
    }

    printf("모델    : %s\n", argv[1]);
    printf("입력    : %zu bytes, 출력 %u개\n", input_len, io.n_output);
    printf("조건    : %d스레드 × %d회, 스레드마다 다른 입력\n\n", threads, iters);
    printf("판정: 결과불일치가 0 이 아니면 그 구성은 정답을 보장하지 못한다.\n");
    printf("      API 오류가 0 이어도 마찬가지다.\n\n");

    int shared_mis = run_mode("컨텍스트 공유", 1, model, model_len,
                              inputs, input_len, io.n_output, threads, iters);
    int own_mis    = run_mode("스레드별 전용 컨텍스트", 0, model, model_len,
                              inputs, input_len, io.n_output, threads, iters);

    printf("\n결론: ");
    if (shared_mis > 0 && own_mis == 0) {
        printf("공유는 결과를 섞는다. 전용 컨텍스트가 필요하다.\n");
    } else if (shared_mis == 0 && own_mis == 0) {
        printf("이 조건에서는 둘 다 결과가 일치했다.\n");
        printf("      다만 시퀀스가 원자적이지 않다는 사실은 그대로이므로,\n");
        printf("      스레드 수·부하를 바꿔 재확인할 것.\n");
    } else {
        printf("예상과 다르다. 전용 컨텍스트에서도 불일치가 나왔다.\n");
    }

    for (int i = 0; i < threads; i++) free(inputs[i]);
    free(model);
    return 0;
}
