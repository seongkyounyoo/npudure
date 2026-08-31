//! 스케줄링 정책 식별자.
//!
//! `docs/01-TECHSPEC.md` §10.0에서 고정한 세 개의 식별자를 여기서만 정의한다.
//! 설정 파일, CLI 인자, 메트릭 레이블, 로그, 대시보드가 모두 이 타입을 거친다.
//!
//! 문자열 비교를 코드 여러 곳에 흩어 두지 않는 것이 목적이다.
//! `queue-aware`, `estimated-completion-time` 같은 표기는 받아들이지 않는다.

use serde::{Deserialize, Serialize};

/// 스케줄링 정책 종류.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SchedulingPolicyKind {
    /// 순차 분배. 다른 정책의 비교 기준으로만 사용한다.
    RoundRobin,
    /// 가장 짧은 큐를 가진 노드 선택.
    LeastQueue,
    /// 예상 완료시간 기반. 권장 기본값.
    #[default]
    Ect,
}

impl SchedulingPolicyKind {
    /// 설정과 CLI에서 사용하는 안정적인 식별자.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RoundRobin => "round-robin",
            Self::LeastQueue => "least-queue",
            Self::Ect => "ect",
        }
    }

    /// 벤치마크 시나리오 생성과 CLI 도움말에 사용한다.
    pub const ALL: [Self; 3] = [Self::RoundRobin, Self::LeastQueue, Self::Ect];
}

impl std::fmt::Display for SchedulingPolicyKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for SchedulingPolicyKind {
    type Err = UnknownPolicy;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "round-robin" => Ok(Self::RoundRobin),
            "least-queue" => Ok(Self::LeastQueue),
            "ect" => Ok(Self::Ect),
            other => Err(UnknownPolicy(other.to_owned())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("알 수 없는 스케줄링 정책 '{0}'. 사용 가능한 값: round-robin, least-queue, ect")]
pub struct UnknownPolicy(pub String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_round_trip() {
        for policy in SchedulingPolicyKind::ALL {
            let parsed: SchedulingPolicyKind = policy.as_str().parse().unwrap();
            assert_eq!(policy, parsed);
        }
    }

    #[test]
    fn serde_uses_the_same_identifiers() {
        // TOML 설정과 CLI가 다른 문자열을 쓰면 문서와 코드가 어긋난다.
        // serde 표현과 as_str()이 항상 일치해야 한다.
        for policy in SchedulingPolicyKind::ALL {
            let json = serde_json::to_string(&policy).unwrap();
            assert_eq!(json, format!("\"{}\"", policy.as_str()));
        }
    }

    #[test]
    fn rejected_spellings() {
        // 문서에서 명시적으로 금지한 표기들.
        for bad in [
            "queue-aware",
            "estimated-completion-time",
            "queue_aware",
            "ECT",
            "roundrobin",
        ] {
            assert!(
                bad.parse::<SchedulingPolicyKind>().is_err(),
                "'{bad}' 는 거부되어야 한다"
            );
        }
    }

    #[test]
    fn default_is_ect() {
        assert_eq!(SchedulingPolicyKind::default(), SchedulingPolicyKind::Ect);
    }
}
