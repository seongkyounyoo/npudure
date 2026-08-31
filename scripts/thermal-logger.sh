#!/usr/bin/env bash
# 보드에서 실행되는 온도/주파수/전압 로거.
#
# `run-thermal-comparison.sh` 가 각 보드로 복사해 실행한다.
# SSH 를 통한 중첩 따옴표는 조용히 깨지므로 파일로 분리했다.
#
# 사용법: thermal-logger.sh <출력파일> <지속초>

set -u
OUT="${1:?출력 파일이 필요하다}"
DURATION="${2:?지속 시간(초)이 필요하다}"

read_milli() { awk '{printf "%.1f", $1/1000}' "$1" 2>/dev/null || echo "0"; }

# NPU devfreq 경로는 SoC 마다 다르다. 없으면 0 으로 둔다.
NPU_FREQ=$(ls /sys/class/devfreq/*npu*/cur_freq 2>/dev/null | head -1)
CPU_FREQ=/sys/devices/system/cpu/cpu4/cpufreq/scaling_cur_freq
VOLT=/sys/class/power_supply/simple-vin/voltage_now

echo "epoch soc_c npu_c ddr_c bigcore_c cpu_mhz npu_mhz volt_v" > "$OUT"

end=$(( $(date +%s) + DURATION ))
while [ "$(date +%s)" -lt "$end" ]; do
    printf '%s %s %s %s %s %s %s %s\n' \
        "$(date +%s)" \
        "$(read_milli /sys/class/thermal/thermal_zone0/temp)" \
        "$(read_milli /sys/class/thermal/thermal_zone4/temp)" \
        "$(read_milli /sys/class/thermal/thermal_zone3/temp)" \
        "$(read_milli /sys/class/thermal/thermal_zone1/temp)" \
        "$(( $(cat "$CPU_FREQ" 2>/dev/null || echo 0) / 1000 ))" \
        "$(( ${NPU_FREQ:+$(cat "$NPU_FREQ" 2>/dev/null || echo 0)} / 1000000 ))" \
        "$(awk 'BEGIN{printf "%.3f", '"$(cat "$VOLT" 2>/dev/null || echo 0)"'/1000000}')" \
        >> "$OUT"
    sleep 1
done
