//! run 유효성 판정.
//!
//! 벤치마크 도중 노드가 재부팅하면 그 구간의 측정은 무효다. 실측에서
//! 어댑터 용량 부족으로 보드가 리셋된 적이 있고, 당시에는 결과만 보고
//! "성능이 떨어졌다"고 해석할 뻔했다. `docs/board-worklog.md` §2.17.2.
//!
//! 재부팅은 `/proc/sys/kernel/random/boot_id` 로 감지한다. 노드가
//! 하트비트로 보고하고 스케줄러가 중계한다.
//!
//! # 왜 결과를 버리지 않고 표시만 하는가
//!
//! 무효 run 을 자동으로 삭제하지 않는다. 무엇이 왜 무효인지 남아 있어야
//! 원인을 추적할 수 있고, 재부팅이 반복되면 그 자체가 발견이다.
//! 판정 결과를 결과 파일에 함께 저장하고, 요약에서 눈에 띄게 표시한다.

use serde::{Deserialize, Serialize};

/// run 전후로 관측한 노드 상태.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeSnapshot {
    pub node_id: String,
    /// 부팅 식별자. 비어 있으면 관측하지 못한 것이다.
    pub boot_id: String,
    /// 관측 시점의 온도. 없으면 `None`.
    pub temperature_c: Option<f64>,
    /// 입력 전압. 강하는 리셋의 선행 지표다.
    pub input_voltage_v: Option<f64>,
}

/// run 을 무효로 만드는 사유.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Invalidation {
    /// 노드가 run 도중 재부팅했다.
    NodeRebooted {
        node_id: String,
        before: String,
        after: String,
    },
    /// run 시작 때 있던 노드가 끝날 때 사라졌다.
    NodeDisappeared { node_id: String },
    /// run 도중 노드가 추가되었다. 노드 수가 바뀌면 확장 효율을 비교할 수 없다.
    NodeAppeared { node_id: String },
    /// 오류율이 허용치를 넘었다.
    ErrorRateTooHigh { observed: f64, limit: f64 },
    /// 성공 표본이 통계를 내기에 너무 적다.
    TooFewSamples { observed: usize, required: usize },
}

impl std::fmt::Display for Invalidation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NodeRebooted {
                node_id,
                before,
                after,
            } => write!(
                f,
                "'{node_id}' 가 run 도중 재부팅했다 (boot_id {} → {})",
                short(before),
                short(after)
            ),
            Self::NodeDisappeared { node_id } => {
                write!(f, "'{node_id}' 가 run 중에 사라졌다")
            }
            Self::NodeAppeared { node_id } => {
                write!(f, "'{node_id}' 가 run 중에 추가되었다. 노드 수가 바뀌었다")
            }
            Self::ErrorRateTooHigh { observed, limit } => write!(
                f,
                "오류율 {:.2}% 가 허용치 {:.2}% 를 넘었다",
                observed * 100.0,
                limit * 100.0
            ),
            Self::TooFewSamples { observed, required } => {
                write!(f, "성공 표본 {observed}건은 최소 {required}건에 못 미친다")
            }
        }
    }
}

fn short(id: &str) -> &str {
    if id.len() > 8 { &id[..8] } else { id }
}

/// 판정 기준.
#[derive(Debug, Clone, Copy)]
pub struct ValidityPolicy {
    /// 허용 오류율 상한. 0.0 ~ 1.0.
    pub max_error_rate: f64,
    /// 백분위를 낼 수 있는 최소 성공 표본 수.
    ///
    /// p99 를 내려면 최소 100건이 있어야 한 건이라도 상위 1%에 들어간다.
    /// 그보다 적으면 p99 는 사실상 최댓값이고, 그렇게 표기하면 오해를 부른다.
    pub min_samples: usize,
}

impl Default for ValidityPolicy {
    fn default() -> Self {
        Self {
            max_error_rate: 0.01,
            min_samples: 100,
        }
    }
}

/// 판정 결과.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verdict {
    pub valid: bool,
    pub reasons: Vec<Invalidation>,
}

impl Verdict {
    pub fn describe(&self) -> String {
        if self.valid {
            return "유효".to_owned();
        }
        let list: Vec<String> = self.reasons.iter().map(|r| format!("  - {r}")).collect();
        format!("무효\n{}", list.join("\n"))
    }
}

/// run 이 유효한지 판정한다.
///
/// `before` / `after` 는 run 시작·종료 시점의 노드 스냅샷이다.
pub fn judge(
    before: &[NodeSnapshot],
    after: &[NodeSnapshot],
    error_rate: f64,
    succeeded: usize,
    policy: ValidityPolicy,
) -> Verdict {
    let mut reasons = Vec::new();

    for b in before {
        match after.iter().find(|a| a.node_id == b.node_id) {
            None => reasons.push(Invalidation::NodeDisappeared {
                node_id: b.node_id.clone(),
            }),
            Some(a) => {
                // 한쪽이라도 boot_id 를 못 얻었으면 판정하지 않는다.
                // 빈 값과 실제 값을 비교하면 매번 재부팅으로 잡힌다.
                if !b.boot_id.is_empty() && !a.boot_id.is_empty() && b.boot_id != a.boot_id {
                    reasons.push(Invalidation::NodeRebooted {
                        node_id: b.node_id.clone(),
                        before: b.boot_id.clone(),
                        after: a.boot_id.clone(),
                    });
                }
            }
        }
    }

    for a in after {
        if !before.iter().any(|b| b.node_id == a.node_id) {
            reasons.push(Invalidation::NodeAppeared {
                node_id: a.node_id.clone(),
            });
        }
    }

    if error_rate > policy.max_error_rate {
        reasons.push(Invalidation::ErrorRateTooHigh {
            observed: error_rate,
            limit: policy.max_error_rate,
        });
    }

    if succeeded < policy.min_samples {
        reasons.push(Invalidation::TooFewSamples {
            observed: succeeded,
            required: policy.min_samples,
        });
    }

    Verdict {
        valid: reasons.is_empty(),
        reasons,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: &str, boot: &str) -> NodeSnapshot {
        NodeSnapshot {
            node_id: id.into(),
            boot_id: boot.into(),
            temperature_c: Some(70.0),
            input_voltage_v: Some(5.1),
        }
    }

    fn ok_policy() -> ValidityPolicy {
        ValidityPolicy {
            max_error_rate: 0.01,
            min_samples: 100,
        }
    }

    #[test]
    fn stable_cluster_is_valid() {
        let before = vec![node("king", "boot-a"), node("queen", "boot-b")];
        let after = before.clone();
        let v = judge(&before, &after, 0.0, 1000, ok_policy());
        assert!(v.valid, "{}", v.describe());
    }

    #[test]
    fn reboot_invalidates_the_run() {
        // 이 판정이 없으면 리셋된 노드의 낮은 처리량을 "성능 저하"로 읽는다.
        let before = vec![node("king", "boot-a")];
        let after = vec![node("king", "boot-DIFFERENT")];
        let v = judge(&before, &after, 0.0, 1000, ok_policy());
        assert!(!v.valid);
        assert!(matches!(v.reasons[0], Invalidation::NodeRebooted { .. }));
        assert!(v.describe().contains("king"), "{}", v.describe());
    }

    #[test]
    fn missing_boot_id_does_not_trigger_a_false_reboot() {
        // boot_id 를 못 읽는 환경(비리눅스 등)에서 매 run 이 무효가 되면 안 된다.
        let before = vec![node("king", "")];
        let after = vec![node("king", "boot-a")];
        assert!(judge(&before, &after, 0.0, 1000, ok_policy()).valid);

        let before2 = vec![node("king", "boot-a")];
        let after2 = vec![node("king", "")];
        assert!(judge(&before2, &after2, 0.0, 1000, ok_policy()).valid);
    }

    #[test]
    fn disappearing_node_invalidates() {
        let before = vec![node("king", "a"), node("queen", "b")];
        let after = vec![node("king", "a")];
        let v = judge(&before, &after, 0.0, 1000, ok_policy());
        assert!(!v.valid);
        assert_eq!(
            v.reasons,
            vec![Invalidation::NodeDisappeared {
                node_id: "queen".into()
            }]
        );
    }

    #[test]
    fn appearing_node_invalidates() {
        // 확장 효율 실험 도중 노드가 늘면 1노드 결과인지 2노드 결과인지 알 수 없다.
        let before = vec![node("king", "a")];
        let after = vec![node("king", "a"), node("queen", "b")];
        let v = judge(&before, &after, 0.0, 1000, ok_policy());
        assert!(!v.valid);
        assert!(matches!(v.reasons[0], Invalidation::NodeAppeared { .. }));
    }

    #[test]
    fn high_error_rate_invalidates() {
        let n = vec![node("king", "a")];
        let v = judge(&n, &n, 0.05, 1000, ok_policy());
        assert!(!v.valid);
        assert!(v.describe().contains("5.00%"), "{}", v.describe());
    }

    #[test]
    fn error_rate_exactly_at_limit_is_valid() {
        // 경계에서 흔들리면 같은 조건의 run 이 어떤 날은 유효, 어떤 날은
        // 무효가 된다. 한계값은 포함한다.
        let n = vec![node("king", "a")];
        assert!(judge(&n, &n, 0.01, 1000, ok_policy()).valid);
    }

    #[test]
    fn too_few_samples_invalidates() {
        // p99 를 내려면 최소 100건이 필요하다. 그보다 적으면 p99 = max 다.
        let n = vec![node("king", "a")];
        let v = judge(&n, &n, 0.0, 42, ok_policy());
        assert!(!v.valid);
        assert!(v.describe().contains("42"), "{}", v.describe());
    }

    #[test]
    fn multiple_problems_are_all_reported() {
        // 하나만 보고하면 고치고 다시 돌렸을 때 또 다른 이유로 실패한다.
        let before = vec![node("king", "a"), node("queen", "b")];
        let after = vec![node("king", "CHANGED")];
        let v = judge(&before, &after, 0.5, 10, ok_policy());
        assert!(!v.valid);
        assert_eq!(v.reasons.len(), 4, "{:?}", v.reasons);
    }

    #[test]
    fn verdict_serializes_with_readable_tags() {
        // 결과 JSON 을 나중에 사람이 읽는다.
        let v = judge(
            &[node("king", "a")],
            &[node("king", "b")],
            0.0,
            1000,
            ok_policy(),
        );
        let json = serde_json::to_string(&v).unwrap();
        assert!(json.contains("node-rebooted"), "{json}");
    }

    #[test]
    fn boot_id_is_shortened_in_messages() {
        // 전체 UUID 를 찍으면 메시지가 읽기 어려워진다.
        let v = judge(
            &[node("king", "0efd0d97-e55e-416b-809b-255ca50553ca")],
            &[node("king", "1111aaaa-bbbb-cccc-dddd-eeeeffff0000")],
            0.0,
            1000,
            ok_policy(),
        );
        let msg = v.describe();
        assert!(msg.contains("0efd0d97"), "{msg}");
        assert!(!msg.contains("255ca50553ca"), "전체 UUID 는 길다: {msg}");
    }
}
