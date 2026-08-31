/* 추론 출력 덤프 — 정확도 검증용.
 *
 * 고정 입력 하나를 넣고 모든 출력 텐서를 파일로 쓴다.
 * ONNX 원본 결과와 비교해 양자화 손실을 본다.
 *
 * 시뮬레이터는 빌드된 .rknn 을 추론할 수 없으므로(load_rknn 후
 * init_runtime 이 거부한다) 실보드에서 이 도구로 검증한다.
 *
 * 출력 파일 형식 (little-endian):
 *   magic   u32  "RKNT" = 0x544E4B52
 *   version u32  = 1
 *   count   u32  텐서 개수
 *   dtype   u32  0 = 원본 dtype, 1 = float32
 *   텐서마다: len u32, n_dims u32, dims[4] u32
 *   이어서 텐서 데이터가 순서대로 (패딩 없음)
 *
 * 빌드:
 *   gcc -O2 -o dump_output_test dump_output_test.c -lrknnrt -lpthread
 *
 * 사용법:
 *   ./dump_output_test <model.rknn> <input.bin> <output.bin> [want_float]
 *     input.bin  : 640*640*3 uint8 NHWC raw
 *     want_float : 기본 1
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <errno.h>

#include "rknn_api.h"

#define MAX_OUT 32
#define MAGIC 0x544E4B52u

static uint8_t *read_file(const char *path, size_t *len)
{
    FILE *f = fopen(path, "rb");
    if (!f) return NULL;
    fseek(f, 0, SEEK_END);
    long n = ftell(f);
    fseek(f, 0, SEEK_SET);
    if (n <= 0) { fclose(f); return NULL; }
    uint8_t *b = malloc((size_t)n);
    if (!b) { fclose(f); return NULL; }
    if (fread(b, 1, (size_t)n, f) != (size_t)n) { free(b); fclose(f); return NULL; }
    fclose(f);
    *len = (size_t)n;
    return b;
}

static void put_u32(FILE *f, uint32_t v) { fwrite(&v, 4, 1, f); }

int main(int argc, char **argv)
{
    if (argc < 4) {
        fprintf(stderr,
            "사용법: %s <model.rknn> <input.bin> <output.bin> [want_float=1]\n"
            "  input.bin 은 모델 입력 크기와 정확히 같아야 합니다.\n", argv[0]);
        return 2;
    }
    const char *model_path = argv[1];
    const char *input_path = argv[2];
    const char *out_path   = argv[3];
    int want_float = (argc >= 5) ? atoi(argv[4]) : 1;

    size_t model_len = 0;
    uint8_t *model = read_file(model_path, &model_len);
    if (!model) {
        fprintf(stderr, "모델을 읽을 수 없습니다: %s (%s)\n", model_path, strerror(errno));
        return 1;
    }

    rknn_context ctx = 0;
    if (rknn_init(&ctx, model, (uint32_t)model_len, 0, NULL) != RKNN_SUCC) {
        fprintf(stderr, "rknn_init 실패\n");
        free(model);
        return 1;
    }
    free(model);

    rknn_input_output_num io;
    memset(&io, 0, sizeof(io));
    if (rknn_query(ctx, RKNN_QUERY_IN_OUT_NUM, &io, sizeof(io)) != RKNN_SUCC) {
        fprintf(stderr, "IN_OUT_NUM 실패\n"); rknn_destroy(ctx); return 1;
    }
    if (io.n_output > MAX_OUT) {
        fprintf(stderr, "출력이 너무 많습니다: %u\n", io.n_output);
        rknn_destroy(ctx); return 1;
    }

    rknn_tensor_attr ia;
    memset(&ia, 0, sizeof(ia));
    ia.index = 0;
    if (rknn_query(ctx, RKNN_QUERY_INPUT_ATTR, &ia, sizeof(ia)) != RKNN_SUCC) {
        fprintf(stderr, "INPUT_ATTR 실패\n"); rknn_destroy(ctx); return 1;
    }

    size_t input_len = 0;
    uint8_t *input = read_file(input_path, &input_len);
    if (!input) {
        fprintf(stderr, "입력을 읽을 수 없습니다: %s\n", input_path);
        rknn_destroy(ctx); return 1;
    }
    if (input_len != (size_t)ia.n_elems) {
        fprintf(stderr, "입력 크기 불일치: 파일 %zu bytes, 모델 요구 %u bytes\n",
                input_len, ia.n_elems);
        free(input); rknn_destroy(ctx); return 1;
    }

    rknn_input in;
    memset(&in, 0, sizeof(in));
    in.index = 0;
    in.type = RKNN_TENSOR_UINT8;
    in.fmt  = RKNN_TENSOR_NHWC;
    in.size = (uint32_t)input_len;
    in.buf  = input;

    if (rknn_inputs_set(ctx, 1, &in) != RKNN_SUCC) {
        fprintf(stderr, "inputs_set 실패\n"); free(input); rknn_destroy(ctx); return 1;
    }
    if (rknn_run(ctx, NULL) != RKNN_SUCC) {
        fprintf(stderr, "run 실패\n"); free(input); rknn_destroy(ctx); return 1;
    }

    rknn_output outs[MAX_OUT];
    memset(outs, 0, sizeof(outs));
    for (uint32_t i = 0; i < io.n_output; i++) {
        outs[i].index = i;
        outs[i].want_float = (uint8_t)(want_float ? 1 : 0);
    }
    if (rknn_outputs_get(ctx, io.n_output, outs, NULL) != RKNN_SUCC) {
        fprintf(stderr, "outputs_get 실패\n"); free(input); rknn_destroy(ctx); return 1;
    }

    FILE *of = fopen(out_path, "wb");
    if (!of) {
        fprintf(stderr, "출력 파일을 열 수 없습니다: %s\n", out_path);
        rknn_outputs_release(ctx, io.n_output, outs);
        free(input); rknn_destroy(ctx); return 1;
    }

    put_u32(of, MAGIC);
    put_u32(of, 1);
    put_u32(of, io.n_output);
    put_u32(of, (uint32_t)(want_float ? 1 : 0));

    for (uint32_t i = 0; i < io.n_output; i++) {
        rknn_tensor_attr oa;
        memset(&oa, 0, sizeof(oa));
        oa.index = i;
        rknn_query(ctx, RKNN_QUERY_OUTPUT_ATTR, &oa, sizeof(oa));

        put_u32(of, outs[i].size);
        uint32_t nd = oa.n_dims > 4 ? 4 : oa.n_dims;
        put_u32(of, nd);
        for (uint32_t d = 0; d < 4; d++) {
            put_u32(of, d < nd ? oa.dims[d] : 0);
        }
    }
    for (uint32_t i = 0; i < io.n_output; i++) {
        fwrite(outs[i].buf, 1, outs[i].size, of);
    }
    fclose(of);

    printf("모델   : %s\n", model_path);
    printf("입력   : %s (%zu bytes)\n", input_path, input_len);
    printf("출력   : %u개 텐서, want_float=%d\n", io.n_output, want_float);
    for (uint32_t i = 0; i < io.n_output; i++) {
        printf("  [%u] %u bytes\n", i, outs[i].size);
    }
    printf("기록   : %s\n", out_path);

    rknn_outputs_release(ctx, io.n_output, outs);
    free(input);
    rknn_destroy(ctx);
    return 0;
}
