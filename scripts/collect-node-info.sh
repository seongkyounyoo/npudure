#!/usr/bin/env bash
# 노드 하드웨어·소프트웨어 실측 정보를 수집한다.
#
# docs/environment-matrix.md 를 채우는 데 필요한 값을 한 번에 모은다.
# 노드에서 직접 실행하거나, scripts/collect-all.sh 를 통해 원격 실행한다.
#
# 출력은 `키=값` 형식이라 파싱과 노드 간 diff 가 쉽다.

set -uo pipefail

kv() { printf '%s=%s\n' "$1" "${2:-UNKNOWN}"; }
try() { "$@" 2>/dev/null || true; }

echo "# --- identity ---"
kv hostname "$(hostname)"
kv model "$(try tr -d '\0' < /proc/device-tree/model)"
kv compatible "$(try tr '\0' ' ' < /proc/device-tree/compatible)"
kv serial "$(try tr -d '\0' < /proc/device-tree/serial-number)"

echo "# --- firmware / bootloader ---"
# 부트로더 펌웨어는 전력 관리(BL31/ATF)와 DDR 타이밍을 담당한다.
# 노드 간 버전이 다르면 고부하 안정성이 달라진다.
# 2026-08-10 king 이 구버전 펌웨어로 5스레드 이상에서 리셋되었다.
kv fwver "$(try grep -oE 'androidboot\.fwver=[^ ]*' /proc/cmdline | cut -d= -f2)"
kv fw_ddr "$(try grep -oE 'ddr-v[0-9.]+' /proc/cmdline)"
kv fw_spl "$(try grep -oE 'spl-v[0-9.]+' /proc/cmdline)"
kv fw_bl31 "$(try grep -oE 'bl31-v[0-9.]+' /proc/cmdline)"
kv fw_bl32 "$(try grep -oE 'bl32-v[0-9.]+' /proc/cmdline)"
kv fw_uboot "$(try grep -oE 'uboot-[0-9a-f]+-[0-9/]+' /proc/cmdline)"
kv pmic_init "$(try dmesg | grep -oE 'rk806 [0-9-]+: ON: .*' | head -1)"

echo "# --- os / kernel ---"
kv os "$(try grep -oP '(?<=^PRETTY_NAME=").*(?="$)' /etc/os-release)"
kv kernel "$(uname -r)"
kv arch "$(uname -m)"
kv glibc "$(try ldd --version | head -1 | grep -oE '[0-9]+\.[0-9]+$')"
kv cpu_governor "$(try cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor)"
kv uptime_s "$(try cut -d. -f1 /proc/uptime)"

echo "# --- cpu ---"
kv cpu_model "$(try grep -m1 'model name' /proc/cpuinfo | cut -d: -f2- | xargs)"
kv cpu_count "$(nproc)"
kv cpu_max_khz "$(try cat /sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq)"
for policy in /sys/devices/system/cpu/cpufreq/policy*; do
    [ -d "$policy" ] || continue
    n=$(basename "$policy")
    kv "cpufreq_${n}_max_khz" "$(try cat "$policy/cpuinfo_max_freq")"
    kv "cpufreq_${n}_cur_khz" "$(try cat "$policy/scaling_cur_freq")"
done

echo "# --- memory / storage ---"
kv mem_total_kb "$(try grep MemTotal /proc/meminfo | awk '{print $2}')"
kv mem_available_kb "$(try grep MemAvailable /proc/meminfo | awk '{print $2}')"
for d in /sys/block/mmcblk*; do
    [ -d "$d" ] || continue
    n=$(basename "$d")
    kv "blk_${n}_size_512b" "$(try cat "$d/size")"
done
kv rootfs_avail "$(try df -h / | awk 'NR==2{print $4}')"

echo "# --- network ---"
for n in /sys/class/net/*; do
    dev=$(basename "$n")
    case "$dev" in lo|docker*|veth*|br-*) continue ;; esac
    kv "net_${dev}_mac" "$(try cat "$n/address")"
    kv "net_${dev}_speed" "$(try cat "$n/speed")"
    kv "net_${dev}_operstate" "$(try cat "$n/operstate")"
    kv "net_${dev}_mtu" "$(try cat "$n/mtu")"
    kv "net_${dev}_ipv4" "$(try ip -4 -br addr show "$dev" | awk '{print $3}')"
done

echo "# --- npu ---"
kv rknpu_version "$(try cat /sys/kernel/debug/rknpu/version)"
kv rknpu_load "$(try cat /sys/kernel/debug/rknpu/load)"
kv rknpu_driver_dmesg "$(try dmesg | grep -io 'RKNPU fdab0000.npu: RKNPU driver.*' | head -1)"
kv rknpu_devnode "$(try ls /dev/rknpu* | tr '\n' ' ')"
kv librknnrt_path "$(try find /usr/lib /usr/lib64 /lib -maxdepth 3 -name 'librknnrt.so*' | head -1)"
_lib=$(try find /usr/lib /usr/lib64 /lib -maxdepth 3 -name 'librknnrt.so*' | head -1)
if [ -n "${_lib:-}" ]; then
    kv librknnrt_sha256 "$(try sha256sum "$_lib" | cut -d' ' -f1)"
    kv librknnrt_version "$(try strings "$_lib" | grep -m1 -iE 'librknnrt version[: ]*[0-9]')"
else
    kv librknnrt_sha256 "NOT_INSTALLED"
    kv librknnrt_version "NOT_INSTALLED"
fi
kv rknn_header "$(try find /usr/include -name 'rknn_api.h' | head -1)"

echo "# --- power input ---"
# 입력 전압 실측. 디바이스 트리의 regulator 이름(vcc12v_dcin 등)은
# 실제 입력 전압을 나타내지 않으므로 반드시 센서를 읽는다.
# 2026-08-10 어댑터 전류 부족으로 고부하에서 보드가 리셋되었고,
# 유휴 전압 4.983V 가 그 근거였다.
for ps in /sys/class/power_supply/*; do
    [ -d "$ps" ] || continue
    n=$(basename "$ps")
    kv "psu_${n}_type" "$(try cat "$ps/type")"
    kv "psu_${n}_online" "$(try cat "$ps/online")"
    uv=$(try cat "$ps/voltage_now")
    kv "psu_${n}_voltage_v" "$([ -n "${uv:-}" ] && awk -v v="$uv" 'BEGIN{printf "%.3f", v/1000000}' || echo UNKNOWN)"
done

echo "# --- thermal ---"
for z in /sys/class/thermal/thermal_zone*; do
    [ -d "$z" ] || continue
    n=$(basename "$z")
    t=$(try cat "$z/temp")
    kv "thermal_${n}_type" "$(try cat "$z/type")"
    kv "thermal_${n}_temp_c" "$([ -n "${t:-}" ] && awk -v v="$t" 'BEGIN{printf "%.1f", v/1000}' || echo UNKNOWN)"
done

echo "# --- toolchain ---"
kv gcc "$(try gcc --version | head -1)"
kv rustc "$(try rustc --version)"
kv python3 "$(try python3 --version)"

echo "# --- ssh host key (복제 이미지 확인용) ---"
kv ssh_host_ed25519_fp "$(try ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub | awk '{print $2}')"
