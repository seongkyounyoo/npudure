//! 메트릭 타입과 노드 런타임 상태.
//!
//! `docs/01-TECHSPEC.md` §17 참조.

use serde::{Deserialize, Serialize};

/// 노드가 하트비트로 보고하는 동적 상태.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct NodeHealth {
    pub queue_depth: u32,
    pub in_flight: u32,
    /// 최근 윈도우의 오류율. 0.0 ~ 1.0.
    pub error_rate: f64,
    pub temperature_c: Option<f64>,
    pub cpu_percent: Option<f64>,
    pub memory_percent: Option<f64>,
    pub npu_percent: Option<f64>,
}

/// 지수 이동 평균.
///
/// 스케줄러가 노드별 추론시간과 네트워크 시간을 추적하는 데 사용한다.
/// ECT 점수 계산의 입력이므로 초기값 처리가 라우팅 결과에 직접 영향을 준다.
#[derive(Debug, Clone, Copy)]
pub struct Ewma {
    alpha: f64,
    value: Option<f64>,
}

impl Ewma {
    /// `alpha`는 0.0 초과 1.0 이하. 클수록 최근 값에 민감하다.
    pub fn new(alpha: f64) -> Self {
        assert!(
            alpha > 0.0 && alpha <= 1.0,
            "alpha는 (0, 1] 범위여야 한다: {alpha}"
        );
        Self { alpha, value: None }
    }

    pub fn update(&mut self, sample: f64) {
        self.value = Some(match self.value {
            // 첫 샘플은 그대로 채택한다. 0에서 시작하면 노드가 실제보다
            // 빠른 것으로 평가되어 초기 요청이 한 노드에 몰린다.
            None => sample,
            Some(prev) => self.alpha * sample + (1.0 - self.alpha) * prev,
        });
    }

    /// 아직 샘플이 없으면 `None`.
    ///
    /// 스케줄러는 이 경우 노드를 낙관적으로도 비관적으로도 취급하지 않고
    /// 별도의 초기 라우팅 규칙을 적용한다.
    pub const fn get(&self) -> Option<f64> {
        self.value
    }

    /// 샘플이 없을 때 사용할 기본값을 지정해 조회한다.
    pub fn get_or(&self, default: f64) -> f64 {
        self.value.unwrap_or(default)
    }

    pub const fn has_sample(&self) -> bool {
        self.value.is_some()
    }
}

impl Default for Ewma {
    /// alpha = 0.2. 약 최근 10개 샘플에 해당하는 반응 속도.
    fn default() -> Self {
        Self::new(0.2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_sample_is_adopted_directly() {
        // 0에서 시작하면 신규 노드가 무한히 빠른 것으로 평가된다.
        let mut ewma = Ewma::new(0.2);
        assert!(!ewma.has_sample());
        ewma.update(100.0);
        assert_eq!(ewma.get(), Some(100.0));
    }

    #[test]
    fn converges_toward_recent_samples() {
        let mut ewma = Ewma::new(0.5);
        ewma.update(100.0);
        ewma.update(200.0);
        assert_eq!(ewma.get(), Some(150.0));
        ewma.update(200.0);
        assert_eq!(ewma.get(), Some(175.0));
    }

    #[test]
    fn alpha_one_tracks_latest_only() {
        let mut ewma = Ewma::new(1.0);
        ewma.update(10.0);
        ewma.update(50.0);
        assert_eq!(ewma.get(), Some(50.0));
    }

    #[test]
    #[should_panic(expected = "alpha")]
    fn rejects_invalid_alpha() {
        Ewma::new(0.0);
    }
}
