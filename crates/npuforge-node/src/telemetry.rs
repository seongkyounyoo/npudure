//! 노드 상태 수집.
//!
//! 온도, 전압, boot_id 를 읽어 하트비트에 실어 보낸다.
//!
//! 세 값 모두 오늘의 실측 경험에서 나왔다.
//!   - 온도: 팬리스 보드가 지속 부하에서 91°C 까지 올라간다
//!   - 전압: 어댑터 용량 부족 시 리셋되며, 강하가 선행 지표다
//!   - boot_id: 벤치마크 도중 리셋되면 해당 run 을 무효 처리해야 한다
//!
//! `docs/board-worklog.md` §2.17 참조.

use std::path::Path;

/// 노드가 주기적으로 읽는 하드웨어 상태.
#[derive(Debug, Clone, Default)]
pub struct HardwareStatus {
    pub temperature_c: Option<f64>,
    pub input_voltage_v: Option<f64>,
    pub cpu_percent: Option<f64>,
    pub memory_percent: Option<f64>,
    /// 부팅마다 바뀌는 식별자. 벤치마크 run 무효화 판정에 쓴다.
    pub boot_id: String,
}

/// 전원 입력 전압 센서.
///
/// 실측 경로는 `/sys/class/power_supply/simple-vin/voltage_now` 이며 단위는 µV.
/// 디바이스 트리의 regulator 이름(`vcc12v_dcin` 등)은 실제 입력 전압을
/// 나타내지 않으므로 반드시 이 센서를 읽는다.
const VOLTAGE_PATHS: &[&str] = &[
    "/sys/class/power_supply/simple-vin/voltage_now",
    "/sys/class/power_supply/vin/voltage_now",
];

const BOOT_ID_PATH: &str = "/proc/sys/kernel/random/boot_id";

/// 하드웨어 상태를 수집한다.
///
/// 읽을 수 없는 항목은 `None` 으로 둔다. **0 으로 채우지 않는다** —
/// "센서 없음"과 "값이 0"은 스케줄링 판단에서 다르게 다뤄야 한다.
pub fn collect(temperature_path: Option<&str>) -> HardwareStatus {
    HardwareStatus {
        temperature_c: temperature_path.and_then(read_temperature),
        input_voltage_v: read_input_voltage(),
        cpu_percent: None, // 샘플링이 필요하므로 CpuSampler 가 채운다
        memory_percent: read_memory_percent(),
        boot_id: read_boot_id(),
    }
}

/// thermal zone 은 밀리도(°C × 1000) 단위다.
fn read_temperature(path: &str) -> Option<f64> {
    let raw = std::fs::read_to_string(path).ok()?;
    let milli: f64 = raw.trim().parse().ok()?;
    Some(milli / 1000.0)
}

fn read_input_voltage() -> Option<f64> {
    for p in VOLTAGE_PATHS {
        if Path::new(p).exists()
            && let Ok(raw) = std::fs::read_to_string(p)
            && let Ok(micro) = raw.trim().parse::<f64>()
        {
            return Some(micro / 1_000_000.0);
        }
    }
    None
}

fn read_memory_percent() -> Option<f64> {
    let raw = std::fs::read_to_string("/proc/meminfo").ok()?;
    let mut total = 0.0_f64;
    let mut available = 0.0_f64;
    for line in raw.lines() {
        let mut it = line.split_whitespace();
        match it.next() {
            Some("MemTotal:") => total = it.next()?.parse().ok()?,
            Some("MemAvailable:") => available = it.next()?.parse().ok()?,
            _ => continue,
        }
    }
    if total <= 0.0 {
        return None;
    }
    Some(100.0 * (total - available) / total)
}

/// 부팅 식별자. 값이 바뀌면 노드가 재시작한 것이다.
pub fn read_boot_id() -> String {
    std::fs::read_to_string(BOOT_ID_PATH)
        .map(|s| s.trim().to_owned())
        .unwrap_or_default()
}

/// `/proc/stat` 을 두 시점 비교해 CPU 사용률을 낸다.
///
/// 순간값을 읽을 수 없으므로 이전 샘플을 보관한다.
#[derive(Debug, Default)]
pub struct CpuSampler {
    prev: Option<(u64, u64)>, // (busy, total)
}

impl CpuSampler {
    pub fn new() -> Self {
        Self::default()
    }

    /// 직전 호출 이후의 CPU 사용률. 첫 호출은 `None` 을 반환한다.
    pub fn sample(&mut self) -> Option<f64> {
        let (busy, total) = read_cpu_jiffies()?;
        let result = match self.prev {
            Some((pb, pt)) if total > pt => Some(100.0 * (busy - pb) as f64 / (total - pt) as f64),
            _ => None,
        };
        self.prev = Some((busy, total));
        result
    }
}

fn read_cpu_jiffies() -> Option<(u64, u64)> {
    let raw = std::fs::read_to_string("/proc/stat").ok()?;
    let line = raw.lines().next()?;
    let v: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|s| s.parse().ok())
        .collect();
    if v.len() < 5 {
        return None;
    }
    let idle = v[3] + v[4]; // idle + iowait
    let total: u64 = v.iter().sum();
    Some((total.saturating_sub(idle), total))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_temperature_path_yields_none() {
        // 센서가 없을 때 0.0 으로 채우면 스케줄러가 "매우 시원한 노드"로 오인한다.
        assert_eq!(read_temperature("/nonexistent/thermal"), None);
    }

    #[test]
    fn collect_without_temperature_path_is_safe() {
        let s = collect(None);
        assert_eq!(s.temperature_c, None);
        // boot_id 는 리눅스에서만 존재한다. 없으면 빈 문자열이다.
        assert!(s.boot_id.is_empty() || s.boot_id.len() >= 32);
    }

    #[test]
    fn cpu_sampler_needs_two_samples() {
        let mut s = CpuSampler::new();
        // 첫 샘플은 비교 대상이 없다
        let first = s.sample();
        if read_cpu_jiffies().is_some() {
            assert!(first.is_none(), "첫 호출은 None 이어야 한다");
        }
    }

    #[test]
    fn temperature_parses_millidegrees() {
        let dir = std::env::temp_dir();
        let path = dir.join("npuforge_test_thermal");
        std::fs::write(&path, "72345\n").unwrap();
        let t = read_temperature(path.to_str().unwrap());
        std::fs::remove_file(&path).ok();
        assert_eq!(t, Some(72.345));
    }
}
