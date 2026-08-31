/*
 * NPUForge 최소 RKNN C Wrapper.
 *
 * 검증 상태: 실장비(RK3576, RKNN Runtime 2.3.0, driver 0.9.8)에서 컴파일 및
 * 동작 검증 완료. 확정된 버전 조합은 docs/environment-matrix.md §3 참조.
 */

#include "rknn_wrapper.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <rknn_api.h>

struct npf_rknn_ctx {
    rknn_context ctx;
    uint32_t     n_input;
    uint32_t     n_output;
    int          want_float;
    /* 입력 텐서 한 장의 바이트 크기. 매 추론마다 검증한다. */
    uint32_t     input_bytes;
    /* rknn_outputs_get 에 넘길 배열. 컨텍스트 수명 동안 재사용한다.
     * 이 배열이 컨텍스트를 공유할 수 없는 이유다. */
    rknn_output *outputs;
};

/* 모델 파일을 통째로 읽는다. 호출자가 free 한다. */
static uint8_t *read_file(const char *path, size_t *out_len)
{
    FILE *fp = fopen(path, "rb");
    if (!fp) {
        return NULL;
    }
    if (fseek(fp, 0, SEEK_END) != 0) {
        fclose(fp);
        return NULL;
    }
    long size = ftell(fp);
    if (size <= 0) {
        fclose(fp);
        return NULL;
    }
    rewind(fp);

    uint8_t *buf = (uint8_t *)malloc((size_t)size);
    if (!buf) {
        fclose(fp);
        return NULL;
    }
    size_t got = fread(buf, 1, (size_t)size, fp);
    fclose(fp);

    if (got != (size_t)size) {
        free(buf);
        return NULL;
    }
    *out_len = got;
    return buf;
}

npf_rknn_status npf_rknn_create(const char *model_path,
                                int want_float,
                                npf_rknn_ctx **out_ctx)
{
    if (!model_path || !out_ctx) {
        return NPF_RKNN_ERR_INVALID_ARG;
    }
    *out_ctx = NULL;

    size_t model_len = 0;
    uint8_t *model = read_file(model_path, &model_len);
    if (!model) {
        return NPF_RKNN_ERR_MODEL_LOAD;
    }

    npf_rknn_ctx *w = (npf_rknn_ctx *)calloc(1, sizeof(npf_rknn_ctx));
    if (!w) {
        free(model);
        return NPF_RKNN_ERR_ALLOC;
    }
    w->want_float = want_float ? 1 : 0;

    int rc = rknn_init(&w->ctx, model, (uint32_t)model_len, 0, NULL);
    /* rknn_init 은 모델 버퍼를 내부로 복사한다. 즉시 해제해도 안전하다. */
    free(model);

    if (rc != RKNN_SUCC) {
        free(w);
        return NPF_RKNN_ERR_MODEL_LOAD;
    }

    rknn_input_output_num io_num;
    memset(&io_num, 0, sizeof(io_num));
    rc = rknn_query(w->ctx, RKNN_QUERY_IN_OUT_NUM, &io_num, sizeof(io_num));
    if (rc != RKNN_SUCC) {
        rknn_destroy(w->ctx);
        free(w);
        return NPF_RKNN_ERR_QUERY;
    }

    if (io_num.n_output > NPF_RKNN_MAX_OUTPUTS) {
        /* 조용히 잘라내지 않는다. 출력을 버리면 후처리가 틀린 답을 낸다. */
        rknn_destroy(w->ctx);
        free(w);
        return NPF_RKNN_ERR_TOO_MANY_OUTPUTS;
    }

    w->n_input = io_num.n_input;
    w->n_output = io_num.n_output;

    rknn_tensor_attr in_attr;
    memset(&in_attr, 0, sizeof(in_attr));
    in_attr.index = 0;
    rc = rknn_query(w->ctx, RKNN_QUERY_INPUT_ATTR, &in_attr, sizeof(in_attr));
    if (rc != RKNN_SUCC) {
        rknn_destroy(w->ctx);
        free(w);
        return NPF_RKNN_ERR_QUERY;
    }
    /* uint8 입력 기준. n_elems 가 곧 바이트 수다. */
    w->input_bytes = in_attr.n_elems;

    w->outputs = (rknn_output *)calloc(io_num.n_output, sizeof(rknn_output));
    if (!w->outputs) {
        rknn_destroy(w->ctx);
        free(w);
        return NPF_RKNN_ERR_ALLOC;
    }

    *out_ctx = w;
    return NPF_RKNN_OK;
}

void npf_rknn_destroy(npf_rknn_ctx *ctx)
{
    if (!ctx) {
        return;
    }
    free(ctx->outputs);
    rknn_destroy(ctx->ctx);
    free(ctx);
}

npf_rknn_status npf_rknn_get_model_info(npf_rknn_ctx *ctx, npf_rknn_model_info *out_info)
{
    if (!ctx || !out_info) {
        return NPF_RKNN_ERR_INVALID_ARG;
    }
    memset(out_info, 0, sizeof(*out_info));
    out_info->input_count = ctx->n_input;
    out_info->output_count = ctx->n_output;
    out_info->input_bytes = ctx->input_bytes;

    rknn_tensor_attr attr;
    memset(&attr, 0, sizeof(attr));
    attr.index = 0;
    if (rknn_query(ctx->ctx, RKNN_QUERY_INPUT_ATTR, &attr, sizeof(attr)) != RKNN_SUCC) {
        return NPF_RKNN_ERR_QUERY;
    }

    /* 레이아웃은 모델 변환 시 결정된다. NHWC 와 NCHW 를 구분해 읽는다.
     * 잘못 읽으면 노드가 640x640 을 3x640 으로 보고하고, 스케줄러가
     * 입력 크기 검증에서 멀쩡한 요청을 거부한다. */
    if (attr.n_dims >= 4) {
        if (attr.fmt == RKNN_TENSOR_NCHW) {
            out_info->input_channels = attr.dims[1];
            out_info->input_height   = attr.dims[2];
            out_info->input_width    = attr.dims[3];
        } else {
            out_info->input_height   = attr.dims[1];
            out_info->input_width    = attr.dims[2];
            out_info->input_channels = attr.dims[3];
        }
    }

    size_t total = 0;
    for (uint32_t i = 0; i < ctx->n_output; i++) {
        rknn_tensor_attr oa;
        memset(&oa, 0, sizeof(oa));
        oa.index = i;
        if (rknn_query(ctx->ctx, RKNN_QUERY_OUTPUT_ATTR, &oa, sizeof(oa)) != RKNN_SUCC) {
            return NPF_RKNN_ERR_QUERY;
        }

        uint32_t nd = oa.n_dims > NPF_RKNN_MAX_DIMS ? NPF_RKNN_MAX_DIMS : oa.n_dims;
        out_info->outputs[i].n_dims = nd;
        for (uint32_t d = 0; d < nd; d++) {
            out_info->outputs[i].dims[d] = oa.dims[d];
        }

        /* 역양자화 파라미터. want_float=0 일 때 받는 쪽이 이것으로
         * real = (q - zp) * scale 을 계산한다. */
        out_info->outputs[i].qnt_type   = (uint32_t)oa.qnt_type;
        out_info->outputs[i].zero_point = oa.zp;
        out_info->outputs[i].scale      = oa.scale;
        /* want_float=1 이면 런타임이 float32 로 변환해 주므로
         * 원본 dtype 대신 그것을 알린다. 안 그러면 받는 쪽이 int8 로
         * 읽으려 한다. */
        out_info->outputs[i].dtype =
            ctx->want_float ? (uint32_t)RKNN_TENSOR_FLOAT32 : (uint32_t)oa.type;
        /* want_float 이면 float32 로 받으므로 원소당 4바이트다.
         * oa.size 는 모델 원본 dtype 기준이라 그대로 쓰면 안 된다. */
        uint32_t bytes = ctx->want_float ? oa.n_elems * 4u : oa.size;
        out_info->outputs[i].size = bytes;
        total += bytes;
    }
    out_info->output_total_bytes = total;

    return NPF_RKNN_OK;
}

npf_rknn_status npf_rknn_infer(npf_rknn_ctx *ctx,
                               const uint8_t *input,
                               size_t input_len,
                               npf_rknn_output *out_output)
{
    if (!ctx || !input || !out_output || input_len == 0) {
        return NPF_RKNN_ERR_INVALID_ARG;
    }
    /* 크기가 다르면 RKNN 이 조용히 다른 것을 읽는다. 여기서 막는다. */
    if (ctx->input_bytes != 0 && input_len != (size_t)ctx->input_bytes) {
        return NPF_RKNN_ERR_INVALID_ARG;
    }
    memset(out_output, 0, sizeof(*out_output));

    rknn_input in;
    memset(&in, 0, sizeof(in));
    in.index = 0;
    in.type = RKNN_TENSOR_UINT8;
    in.fmt = RKNN_TENSOR_NHWC;
    in.size = (uint32_t)input_len;
    /* rknn_inputs_set 은 const 를 받지 않는다. 호출 중 수정하지 않으므로
     * 캐스팅으로 넘긴다. 입력 버퍼는 호출이 끝날 때까지 살아 있어야 한다. */
    in.buf = (void *)(uintptr_t)input;
    in.pass_through = 0;

    if (rknn_inputs_set(ctx->ctx, 1, &in) != RKNN_SUCC) {
        return NPF_RKNN_ERR_INPUT_SET;
    }

    if (rknn_run(ctx->ctx, NULL) != RKNN_SUCC) {
        return NPF_RKNN_ERR_RUN;
    }

    for (uint32_t i = 0; i < ctx->n_output; i++) {
        memset(&ctx->outputs[i], 0, sizeof(rknn_output));
        ctx->outputs[i].index = i;
        ctx->outputs[i].want_float = (uint8_t)ctx->want_float;
        ctx->outputs[i].is_prealloc = 0;
    }

    if (rknn_outputs_get(ctx->ctx, ctx->n_output, ctx->outputs, NULL) != RKNN_SUCC) {
        return NPF_RKNN_ERR_OUTPUT_GET;
    }

    out_output->count = ctx->n_output;
    for (uint32_t i = 0; i < ctx->n_output; i++) {
        out_output->data[i] = (uint8_t *)ctx->outputs[i].buf;
        out_output->len[i] = ctx->outputs[i].size;
    }
    out_output->_held = 1;

    return NPF_RKNN_OK;
}

void npf_rknn_release_output(npf_rknn_ctx *ctx, npf_rknn_output *output)
{
    if (!ctx || !output) {
        return;
    }
    /* 두 번 해제해도 안전해야 한다. Rust 쪽 Drop 과 명시적 해제가
     * 겹칠 수 있다. */
    if (!output->_held) {
        return;
    }
    rknn_outputs_release(ctx->ctx, ctx->n_output, ctx->outputs);
    memset(output, 0, sizeof(*output));
}

npf_rknn_status npf_rknn_get_runtime_version(char *buf, size_t buf_len)
{
    if (!buf || buf_len == 0) {
        return NPF_RKNN_ERR_INVALID_ARG;
    }
    buf[0] = '\0';

    /* rknn_query 로 SDK 버전을 얻으려면 컨텍스트가 필요하다.
     * 컨텍스트 없이 얻는 공식 API 가 없으므로 드라이버 버전과 함께
     * 파일에서 읽는다. 노드 간 런타임 일치 확인이 목적이다. */
    FILE *fp = fopen("/sys/kernel/debug/rknpu/version", "r");
    if (fp) {
        char drv[128] = {0};
        if (fgets(drv, sizeof(drv), fp)) {
            size_t n = strlen(drv);
            while (n > 0 && (drv[n - 1] == '\n' || drv[n - 1] == '\r')) {
                drv[--n] = '\0';
            }
            snprintf(buf, buf_len, "%s", drv);
        }
        fclose(fp);
    }

    if (buf[0] == '\0') {
        snprintf(buf, buf_len, "unknown");
    }
    return NPF_RKNN_OK;
}

/*
 * 컨텍스트가 있을 때의 정확한 SDK 버전.
 *
 * 노드는 모델을 로딩한 뒤 이 값을 스케줄러에 보고한다.
 * 세 노드가 다른 런타임을 쓰면 벤치마크 결과를 비교할 수 없다.
 */
npf_rknn_status npf_rknn_get_sdk_version(npf_rknn_ctx *ctx, char *buf, size_t buf_len)
{
    if (!ctx || !buf || buf_len == 0) {
        return NPF_RKNN_ERR_INVALID_ARG;
    }
    rknn_sdk_version ver;
    memset(&ver, 0, sizeof(ver));
    if (rknn_query(ctx->ctx, RKNN_QUERY_SDK_VERSION, &ver, sizeof(ver)) != RKNN_SUCC) {
        return NPF_RKNN_ERR_QUERY;
    }
    snprintf(buf, buf_len, "%s (driver %s)", ver.api_version, ver.drv_version);
    return NPF_RKNN_OK;
}
