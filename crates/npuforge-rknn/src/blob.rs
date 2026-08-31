//! 다중 출력 텐서를 하나의 바이트열로 담는 형식.
//!
//! 모델이 출력을 여러 개 내면(YOLOv8n 은 9개) 어딘가에서 하나의 버퍼로
//! 합쳐야 한다. `InferenceOutput::result` 도 `Bytes` 하나이고 gRPC 의
//! `bytes result` 도 하나다.
//!
//! 단순히 이어붙이면 경계 정보가 사라져 클라이언트가 후처리를 할 수 없다.
//! 그래서 앞에 헤더를 붙여 **자기 기술적**으로 만든다.
//!
//! ```text
//! magic    u32  "RKNT" = 0x544E4B52
//! version  u32  = 2
//! count    u32  텐서 개수
//! dtype    u32  0 = 모델 원본 dtype, 1 = float32 (전체 기본값)
//! 텐서마다 (36바이트):
//!   len        u32  바이트 길이
//!   n_dims     u32
//!   dims       u32 × 4  (n_dims 를 넘는 자리는 0)
//!   qnt_type   u32  0 없음 / 1 DFP / 2 affine asymmetric
//!   zero_point i32
//!   scale      f32  (little-endian IEEE 754)
//! 이어서 텐서 데이터가 순서대로. 패딩 없음.
//! ```
//!
//! # v2 에서 양자화 파라미터를 넣은 이유
//!
//! `want_float = 0` 으로 받으면 데이터가 양자화된 정수다. 이것을 실수로
//! 되돌리려면 텐서마다 `scale` 과 `zero_point` 가 있어야 한다.
//!
//! ```text
//! real = (quantized - zero_point) × scale
//! ```
//!
//! 이 값 없이 int8 을 보내면 **받는 쪽이 해석할 방법이 없다.**
//! `want_float = 0` 은 네트워크 대역폭 때문에 M3 의 전제 조건이 되었으므로
//! (`02-HARDWARE-SETUP.md` §3.3.2), 파라미터 전달이 선택 사항이 아니다.
//!
//! 보드 검증 도구(`native/dump_output_test.c`)와 변환 도구
//! (`tools/model-converter/compare_detections.py`)가 같은 형식을 쓴다.
//! 형식을 바꾸면 세 곳을 같이 고쳐야 한다.

/// `"RKNT"` little-endian.
pub const MAGIC: u32 = 0x544E_4B52;
pub const VERSION: u32 = 2;
/// 고정 헤더 크기.
pub const HEADER_BYTES: usize = 16;
/// 텐서 하나당 기술자 크기. v2 에서 24 → 36 으로 늘었다.
///
/// `len 4 + n_dims 4 + dims 16 + qnt_type 4 + zero_point 4 + scale 4`
pub const DESC_BYTES: usize = 36;

/// `result_format` 필드에 넣는 값. 클라이언트가 이 형식을 식별한다.
pub const RESULT_FORMAT: &str = "rknn-tensors-v1";

/// 텐서 하나의 형태와 역양자화 파라미터.
///
/// `scale` 이 `f32` 라 `Eq` 를 파생할 수 없다. 비트 단위 비교가 필요하면
/// [`TensorShape::bit_eq`] 를 쓴다.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TensorShape {
    pub n_dims: u32,
    pub dims: [u32; 4],
    /// 0 없음 / 1 DFP / 2 affine asymmetric. `rknn_tensor_qnt_type` 과 같다.
    pub qnt_type: u32,
    pub zero_point: i32,
    pub scale: f32,
}

impl TensorShape {
    /// 양자화되지 않은 텐서의 형태.
    pub const fn plain(n_dims: u32, dims: [u32; 4]) -> Self {
        Self {
            n_dims,
            dims,
            qnt_type: 0,
            zero_point: 0,
            scale: 0.0,
        }
    }

    /// 이 텐서가 역양자화를 필요로 하는지 여부.
    ///
    /// `scale` 이 0 이면 파라미터가 없는 것이다. 그대로 곱하면 모든 값이
    /// 0 이 되므로, 0 을 "양자화 없음"과 구별해 다뤄야 한다.
    pub fn needs_dequant(&self) -> bool {
        self.qnt_type != 0 && self.scale != 0.0
    }

    /// 양자화된 값 하나를 실수로 되돌린다.
    pub fn dequantize(&self, q: i32) -> f32 {
        (q - self.zero_point) as f32 * self.scale
    }

    /// `f32` 를 비트로 비교한다. 테스트용.
    pub fn bit_eq(&self, other: &Self) -> bool {
        self.n_dims == other.n_dims
            && self.dims == other.dims
            && self.qnt_type == other.qnt_type
            && self.zero_point == other.zero_point
            && self.scale.to_bits() == other.scale.to_bits()
    }
}

/// 출력 텐서들을 blob 으로 직렬화한다.
///
/// `tensors` 는 (형태, 데이터) 쌍이다.
pub fn encode(tensors: &[(TensorShape, &[u8])], float_output: bool) -> Vec<u8> {
    let payload: usize = tensors.iter().map(|(_, d)| d.len()).sum();
    let mut out = Vec::with_capacity(HEADER_BYTES + DESC_BYTES * tensors.len() + payload);

    out.extend_from_slice(&MAGIC.to_le_bytes());
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.extend_from_slice(&(tensors.len() as u32).to_le_bytes());
    out.extend_from_slice(&u32::from(float_output).to_le_bytes());

    for (shape, data) in tensors {
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&shape.n_dims.to_le_bytes());
        for d in shape.dims {
            out.extend_from_slice(&d.to_le_bytes());
        }
        out.extend_from_slice(&shape.qnt_type.to_le_bytes());
        out.extend_from_slice(&shape.zero_point.to_le_bytes());
        out.extend_from_slice(&shape.scale.to_le_bytes());
    }
    for (_, data) in tensors {
        out.extend_from_slice(data);
    }
    out
}

/// blob 하나에서 읽어낸 내용.
#[derive(Debug, Clone, PartialEq)]
pub struct Decoded<'a> {
    pub float_output: bool,
    pub tensors: Vec<(TensorShape, &'a [u8])>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum DecodeError {
    #[error("blob 이 헤더보다 짧다")]
    TooShort,
    #[error("magic 이 다르다. rknn-tensors-v1 형식이 아니다")]
    BadMagic,
    #[error("알 수 없는 버전: {0}")]
    BadVersion(u32),
    #[error("텐서 기술자가 잘렸다")]
    TruncatedDescriptors,
    #[error("텐서 데이터가 잘렸다 (텐서 {index})")]
    TruncatedPayload { index: usize },
}

fn read_u32(b: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
}

/// blob 을 해석한다. 데이터는 복사하지 않고 빌려 쓴다.
pub fn decode(blob: &[u8]) -> Result<Decoded<'_>, DecodeError> {
    if blob.len() < HEADER_BYTES {
        return Err(DecodeError::TooShort);
    }
    if read_u32(blob, 0) != MAGIC {
        return Err(DecodeError::BadMagic);
    }
    let version = read_u32(blob, 4);
    if version != VERSION {
        return Err(DecodeError::BadVersion(version));
    }
    let count = read_u32(blob, 8) as usize;
    let float_output = read_u32(blob, 12) != 0;

    let desc_end = HEADER_BYTES
        .checked_add(DESC_BYTES.checked_mul(count).ok_or(DecodeError::TooShort)?)
        .ok_or(DecodeError::TooShort)?;
    if blob.len() < desc_end {
        return Err(DecodeError::TruncatedDescriptors);
    }

    let mut descs = Vec::with_capacity(count);
    for i in 0..count {
        let off = HEADER_BYTES + i * DESC_BYTES;
        let len = read_u32(blob, off) as usize;
        let n_dims = read_u32(blob, off + 4);
        let mut dims = [0u32; 4];
        for (d, slot) in dims.iter_mut().enumerate() {
            *slot = read_u32(blob, off + 8 + d * 4);
        }
        let qnt_type = read_u32(blob, off + 24);
        let zero_point = read_u32(blob, off + 28) as i32;
        let scale = f32::from_bits(read_u32(blob, off + 32));
        descs.push((
            TensorShape {
                n_dims,
                dims,
                qnt_type,
                zero_point,
                scale,
            },
            len,
        ));
    }

    let mut tensors = Vec::with_capacity(count);
    let mut off = desc_end;
    for (index, (shape, len)) in descs.into_iter().enumerate() {
        let end = off
            .checked_add(len)
            .ok_or(DecodeError::TruncatedPayload { index })?;
        if blob.len() < end {
            return Err(DecodeError::TruncatedPayload { index });
        }
        tensors.push((shape, &blob[off..end]));
        off = end;
    }

    Ok(Decoded {
        float_output,
        tensors,
    })
}

/// 직렬화 결과의 총 바이트 수를 미리 계산한다.
///
/// 버퍼 풀 크기 결정과 `max_payload_bytes` 검증에 쓴다.
pub const fn encoded_len(tensor_count: usize, payload_bytes: usize) -> usize {
    HEADER_BYTES + DESC_BYTES * tensor_count + payload_bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shape(dims: &[u32]) -> TensorShape {
        let mut d = [0u32; 4];
        for (i, v) in dims.iter().enumerate().take(4) {
            d[i] = *v;
        }
        TensorShape::plain(dims.len() as u32, d)
    }

    fn quantized(dims: &[u32], zp: i32, scale: f32) -> TensorShape {
        let mut sh = shape(dims);
        sh.qnt_type = 2; // affine asymmetric
        sh.zero_point = zp;
        sh.scale = scale;
        sh
    }

    #[test]
    fn round_trip_single_tensor() {
        let data = [1u8, 2, 3, 4, 5, 6];
        let blob = encode(&[(shape(&[1, 2, 3]), &data)], true);
        let got = decode(&blob).unwrap();
        assert!(got.float_output);
        assert_eq!(got.tensors.len(), 1);
        assert!(got.tensors[0].0.bit_eq(&shape(&[1, 2, 3])));
        assert_eq!(got.tensors[0].1, &data);
    }

    #[test]
    fn round_trip_yolov8n_shape() {
        // 실제 모델과 같은 9개 출력. 경계가 보존되어야 후처리가 가능하다.
        let sizes = [
            409_600, 512_000, 6_400, 102_400, 128_000, 1_600, 25_600, 32_000, 400,
        ];
        let bufs: Vec<Vec<u8>> = sizes
            .iter()
            .enumerate()
            .map(|(i, n)| vec![i as u8; *n])
            .collect();
        let tensors: Vec<_> = bufs
            .iter()
            .enumerate()
            .map(|(i, b)| (shape(&[1, i as u32 + 1, 80, 80]), b.as_slice()))
            .collect();

        let blob = encode(&tensors, true);
        assert_eq!(blob.len(), encoded_len(9, sizes.iter().sum::<usize>()));

        let got = decode(&blob).unwrap();
        assert_eq!(got.tensors.len(), 9);
        for (i, (sh, data)) in got.tensors.iter().enumerate() {
            assert_eq!(data.len(), sizes[i], "텐서 {i} 길이");
            assert!(data.iter().all(|&b| b == i as u8), "텐서 {i} 내용");
            assert_eq!(sh.dims[1], i as u32 + 1);
        }
    }

    #[test]
    fn empty_tensor_list_round_trips() {
        let blob = encode(&[], false);
        let got = decode(&blob).unwrap();
        assert!(got.tensors.is_empty());
        assert!(!got.float_output);
    }

    #[test]
    fn zero_length_tensor_is_preserved() {
        // 크기 0 텐서를 조용히 버리면 인덱스가 밀려 후처리가 어긋난다.
        let blob = encode(&[(shape(&[0]), &[]), (shape(&[2]), &[7, 8])], false);
        let got = decode(&blob).unwrap();
        assert_eq!(got.tensors.len(), 2);
        assert!(got.tensors[0].1.is_empty());
        assert_eq!(got.tensors[1].1, &[7, 8]);
    }

    #[test]
    fn dtype_flag_round_trips() {
        for want in [true, false] {
            let blob = encode(&[(shape(&[1]), &[1])], want);
            assert_eq!(decode(&blob).unwrap().float_output, want);
        }
    }

    #[test]
    fn rejects_short_blob() {
        assert_eq!(decode(&[]).unwrap_err(), DecodeError::TooShort);
        assert_eq!(decode(&[0; 15]).unwrap_err(), DecodeError::TooShort);
    }

    #[test]
    fn rejects_bad_magic() {
        let mut blob = encode(&[(shape(&[1]), &[1])], true);
        blob[0] ^= 0xff;
        assert_eq!(decode(&blob).unwrap_err(), DecodeError::BadMagic);
    }

    #[test]
    fn rejects_future_version() {
        // 상위 버전 노드가 보낸 형식을 조용히 잘못 해석하면 안 된다.
        let mut blob = encode(&[(shape(&[1]), &[1])], true);
        blob[4..8].copy_from_slice(&3u32.to_le_bytes());
        assert_eq!(decode(&blob).unwrap_err(), DecodeError::BadVersion(3));
    }

    #[test]
    fn rejects_truncated_descriptors() {
        let blob = encode(&[(shape(&[1]), &[1, 2, 3])], true);
        let cut = &blob[..HEADER_BYTES + 4];
        assert_eq!(decode(cut).unwrap_err(), DecodeError::TruncatedDescriptors);
    }

    #[test]
    fn rejects_truncated_payload() {
        let blob = encode(&[(shape(&[3]), &[1, 2, 3])], true);
        let cut = &blob[..blob.len() - 1];
        assert_eq!(
            decode(cut).unwrap_err(),
            DecodeError::TruncatedPayload { index: 0 }
        );
    }

    #[test]
    fn rejects_absurd_tensor_count_without_allocating() {
        // count 를 크게 조작한 blob 이 와도 거대한 Vec 을 잡지 않아야 한다.
        let mut blob = encode(&[], true);
        blob[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(matches!(
            decode(&blob).unwrap_err(),
            DecodeError::TruncatedDescriptors | DecodeError::TooShort
        ));
    }

    #[test]
    fn quantization_params_round_trip() {
        // want_float=0 으로 보낸 int8 을 받는 쪽이 해석하려면 이 값이 필요하다.
        // 없으면 데이터가 그냥 못 읽는 바이트열이다.
        let sh = quantized(&[1, 80, 80, 80], -128, 0.003_921_569);
        let blob = encode(&[(sh, &[1, 2, 3])], false);
        let got = decode(&blob).unwrap();
        assert!(got.tensors[0].0.bit_eq(&sh), "실제: {:?}", got.tensors[0].0);
        assert!(!got.float_output);
    }

    #[test]
    fn dequantize_matches_the_affine_formula() {
        // real = (q - zp) * scale
        let sh = quantized(&[1], -128, 0.5);
        assert_eq!(sh.dequantize(-128), 0.0);
        assert_eq!(sh.dequantize(-126), 1.0);
        assert_eq!(sh.dequantize(127), 127.5);
    }

    #[test]
    fn zero_scale_is_not_treated_as_quantized() {
        // scale 이 0 인데 역양자화하면 모든 값이 0 이 된다.
        // "파라미터 없음"과 "값이 0"을 구별해야 한다.
        let mut sh = shape(&[1]);
        sh.qnt_type = 2;
        sh.scale = 0.0;
        assert!(!sh.needs_dequant());

        let ok = quantized(&[1], 0, 0.25);
        assert!(ok.needs_dequant());

        // 양자화 타입이 없으면 scale 이 있어도 역양자화 대상이 아니다
        let mut plain = shape(&[1]);
        plain.scale = 0.25;
        assert!(!plain.needs_dequant());
    }

    #[test]
    fn negative_zero_point_survives_the_round_trip() {
        // zero_point 는 부호가 있다. u32 로 읽고 i32 로 캐스팅하는 경로가
        // 맞는지 확인한다.
        for zp in [-128i32, -1, 0, 1, 127] {
            let sh = quantized(&[1], zp, 0.01);
            let blob = encode(&[(sh, &[0])], false);
            assert_eq!(
                decode(&blob).unwrap().tensors[0].0.zero_point,
                zp,
                "zp={zp}"
            );
        }
    }

    #[test]
    fn descriptor_size_matches_the_constant() {
        // DESC_BYTES 를 바꾸면서 인코딩을 안 고치면 조용히 어긋난다.
        let blob = encode(&[(shape(&[1]), &[7])], true);
        assert_eq!(blob.len(), HEADER_BYTES + DESC_BYTES + 1);
    }

    #[test]
    fn v1_blob_is_rejected() {
        // 형식이 바뀌었으므로 옛 blob 을 조용히 잘못 읽으면 안 된다.
        let mut blob = encode(&[(shape(&[1]), &[1])], true);
        blob[4..8].copy_from_slice(&1u32.to_le_bytes());
        assert_eq!(decode(&blob).unwrap_err(), DecodeError::BadVersion(1));
    }

    #[test]
    fn magic_matches_the_c_and_python_tools() {
        // native/dump_output_test.c 와 tools/model-converter/*.py 가 같은 값을 쓴다.
        assert_eq!(MAGIC, u32::from_le_bytes(*b"RKNT"));
    }
}
