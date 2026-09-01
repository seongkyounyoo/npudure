//! NPUDure 벤치마크 도구.
//!
//! 부하를 발생시키고, 지연 분포를 집계하고, run 이 유효한지 판정한다.
//!
//! # 이 도구가 지키는 것
//!
//! 지금까지 측정에서 겪은 실수들이 그대로 설계에 들어가 있다.
//!
//! - **예열을 제외한다.** 첫 추론은 지연이 크게 튄다.
//! - **`boot_id` 로 재부팅을 감지한다.** 도중에 리셋된 노드의 낮은
//!   처리량을 "성능 저하"로 읽지 않기 위해서다.
//! - **표본이 적으면 무효로 표시한다.** 100건 미만에서 p99 는 최댓값이다.
//! - **실패를 처리량에 넣지 않는다.** 넣으면 노드가 다 죽었을 때 처리량이
//!   가장 높아진다.
//! - **결과를 지우지 않는다.** 무효 run 도 사유와 함께 남긴다.
//!
//! # 이 도구가 보장하지 않는 것
//!
//! 닫힌 모델(closed loop) 부하이므로 **coordinated omission** 이 있다.
//! 절대 지연을 SLA 처럼 인용하지 말고, 구성 간 비교에만 쓴다.
//! 자세한 내용은 [`driver`] 모듈 문서에 있다.

pub mod driver;
pub mod report;
pub mod stats;
pub mod validity;

pub use driver::{LoadResult, LoadSpec};
pub use stats::{RunSummary, Sample, summarize};
pub use validity::{NodeSnapshot, ValidityPolicy, Verdict};
