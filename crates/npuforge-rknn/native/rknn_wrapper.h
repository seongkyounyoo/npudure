/*
 * NPUDure 최소 RKNN C Wrapper.
 *
 * Rust가 RKNN 헤더에 직접 강하게 결합되지 않도록 하는 얇은 경계다.
 * docs/03-DEVELOPMENT-REQUIREMENTS.md §2.4 참조.
 *
 * 설계 원칙
 *   - 함수 수를 최소로 유지한다
 *   - 오류 코드를 단순화해 Rust 쪽 변환을 쉽게 한다
 *   - 구조체를 노출하지 않고 불투명 핸들만 주고받는다
 *   - RKNN 헤더 타입이 이 헤더를 통해 새어 나가지 않게 한다
 *
 * 동시성
 *   하나의 npf_rknn_ctx 를 여러 스레드가 동시에 쓰면 안 된다.
 *   rknn_inputs_set → rknn_run → rknn_outputs_get 은 세 번의 호출이고
 *   컨텍스트 내부 상태를 공유하므로, 개별 호출이 thread-safe 여도
 *   시퀀스 전체는 원자적이지 않다. 스레드가 끼어들면 오류 없이
 *   다른 요청의 결과를 가져갈 수 있다.
 *   → 동시 실행 수만큼 컨텍스트를 만들어 하나씩 점유해 쓴다.
 */

#ifndef NPF_RKNN_WRAPPER_H
#define NPF_RKNN_WRAPPER_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* 텐서 하나가 가질 수 있는 최대 차원 수. RKNN 은 4차원까지 쓴다. */
#define NPF_RKNN_MAX_DIMS 4

/* 지원하는 최대 출력 텐서 수. YOLOv8n(rknn_model_zoo)은 9개다. */
#define NPF_RKNN_MAX_OUTPUTS 32

/* 오류 코드. Rust 쪽에서 NPUDure 오류 코드로 변환한다. */
typedef enum {
    NPF_RKNN_OK = 0,
    NPF_RKNN_ERR_INVALID_ARG = -1,
    NPF_RKNN_ERR_MODEL_LOAD = -2,
    NPF_RKNN_ERR_QUERY = -3,
    NPF_RKNN_ERR_INPUT_SET = -4,
    NPF_RKNN_ERR_RUN = -5,
    NPF_RKNN_ERR_OUTPUT_GET = -6,
    NPF_RKNN_ERR_ALLOC = -7,
    NPF_RKNN_ERR_RUNTIME = -8,
    /* 모델의 출력 텐서가 NPF_RKNN_MAX_OUTPUTS 를 넘는다. */
    NPF_RKNN_ERR_TOO_MANY_OUTPUTS = -9
} npf_rknn_status;

/* 불투명 컨텍스트 핸들. Rust는 내부 구조를 알지 못한다. */
typedef struct npf_rknn_ctx npf_rknn_ctx;

/* 텐서 하나의 형태. 출력 해석에 필요하다.
 *
 * want_float=0 으로 받으면 데이터가 양자화된 정수다. 이것을 실수로
 * 되돌리려면 scale 과 zero_point 가 있어야 한다.
 *
 *     real = (quantized - zero_point) * scale
 *
 * 이 값 없이 int8 을 보내면 받는 쪽이 해석할 방법이 없다. want_float=0
 * 을 쓰는 이상 반드시 함께 전달한다. */
typedef struct {
    uint32_t n_dims;
    uint32_t dims[NPF_RKNN_MAX_DIMS];
    /* 이 텐서의 바이트 크기. want_float 설정에 따라 달라진다. */
    uint32_t size;
    /* 0 = 양자화 없음, 1 = DFP, 2 = affine asymmetric.
     * rknn_tensor_qnt_type 과 같은 값이다. */
    uint32_t qnt_type;
    int32_t  zero_point;
    float    scale;
    /* rknn_tensor_type. want_float=1 이면 RKNN_TENSOR_FLOAT32 로 덮어쓴다. */
    uint32_t dtype;
} npf_rknn_tensor_shape;

/* 모델 메타데이터. RKNN 구조체를 그대로 노출하지 않기 위한 축약형. */
typedef struct {
    uint32_t input_count;
    uint32_t output_count;
    uint32_t input_width;
    uint32_t input_height;
    uint32_t input_channels;
    /* 입력 텐서 한 장의 바이트 크기. 요청 검증에 쓴다. */
    uint32_t input_bytes;
    /* 출력 버퍼 총 바이트. 버퍼 풀 크기 결정에 사용한다. */
    size_t   output_total_bytes;
    /* 출력 텐서별 형태. output_count 개까지 유효하다. */
    npf_rknn_tensor_shape outputs[NPF_RKNN_MAX_OUTPUTS];
} npf_rknn_model_info;

/*
 * 추론 출력.
 *
 * 모델이 출력을 여러 개 내면 전부 담는다. YOLOv8n 은 9개다.
 * outputs[0] 만 쓰면 나머지를 조용히 버리게 되므로 그렇게 하지 않는다.
 *
 * npf_rknn_release_output 으로 반드시 해제한다.
 */
typedef struct {
    uint32_t count;
    uint8_t *data[NPF_RKNN_MAX_OUTPUTS];
    uint32_t len[NPF_RKNN_MAX_OUTPUTS];
    /* 내부 해제에 필요한 표식. Rust 는 건드리지 않는다. */
    int      _held;
} npf_rknn_output;

/*
 * 모델 파일을 로딩해 컨텍스트를 만든다.
 * 성공 시 *out_ctx 에 핸들을 채운다. 실패 시 *out_ctx 는 NULL 이다.
 *
 * want_float 이 0 이 아니면 출력을 float32 로 역양자화해 받는다.
 * 0 이면 모델의 원본 dtype 그대로 받는다(INT8 모델이면 int8).
 * 실측상 want_float=0 이 약 5.4% 빠르지만 후처리에서 역양자화를
 * 직접 해야 한다. docs/discuss.md §5 참조.
 */
npf_rknn_status npf_rknn_create(const char *model_path,
                                int want_float,
                                npf_rknn_ctx **out_ctx);

/* 컨텍스트를 해제한다. NULL 을 넘겨도 안전하다. */
void npf_rknn_destroy(npf_rknn_ctx *ctx);

/* 모델 메타데이터를 조회한다. */
npf_rknn_status npf_rknn_get_model_info(npf_rknn_ctx *ctx, npf_rknn_model_info *out_info);

/*
 * 추론을 수행한다.
 *
 * input 은 호출이 반환될 때까지 유효해야 한다.
 * 성공 시 out_output 에 결과를 채우며, 호출자가 npf_rknn_release_output 을
 * 호출할 책임을 진다.
 *
 * 같은 ctx 로 동시에 호출하면 안 된다. 헤더 상단의 동시성 항목 참조.
 */
npf_rknn_status npf_rknn_infer(npf_rknn_ctx *ctx,
                               const uint8_t *input,
                               size_t input_len,
                               npf_rknn_output *out_output);

/* 추론 출력을 해제한다. 같은 출력을 두 번 해제해도 안전하다. */
void npf_rknn_release_output(npf_rknn_ctx *ctx, npf_rknn_output *output);

/*
 * RKNN 드라이버 버전 문자열을 buf 에 채운다.
 *
 * 컨텍스트 없이 호출할 수 있어야 하므로(노드가 모델 로딩 전에 부른다)
 * sysfs 에서 읽는다. 정확한 SDK 버전은 npf_rknn_get_sdk_version 을 쓴다.
 */
npf_rknn_status npf_rknn_get_runtime_version(char *buf, size_t buf_len);

/*
 * 컨텍스트가 있을 때의 정확한 SDK 버전 (api + driver).
 * 노드가 모델 로딩 후 스케줄러에 보고한다. 세 노드가 달라선 안 된다.
 */
npf_rknn_status npf_rknn_get_sdk_version(npf_rknn_ctx *ctx, char *buf, size_t buf_len);

#ifdef __cplusplus
}
#endif

#endif /* NPF_RKNN_WRAPPER_H */
