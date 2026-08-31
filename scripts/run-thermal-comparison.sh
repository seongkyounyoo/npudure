#!/usr/bin/env bash
# 세 보드의 열 거동을 동일 조건에서 비교한다.
#
# 배경: 2026-08-10 지속 부하 시험에서 king 만 NPU 91.3°C 에 도달해
#       queen/jack(70.2/72.1°C)보다 약 19°C 높았다. 유휴에서는 3°C 이내라
#       방열 조건 차이 가설이 유력하다. `docs/board-worklog.md` §2.19.
#
# 이 스크립트가 지키는 것:
#   - 별칭이 가리키는 hostname 을 먼저 검증한다. 결과가 엉뚱한 노드에
#     귀속되는 것이 접속 실패보다 훨씬 위험하다. §2.20
#   - 세 보드의 바이너리/모델 해시가 같은지 확인한다.
#   - 세 보드에 동시에 부하를 건다. 시작이 어긋나면 평탄역 진입 시점이
#     달라져 비교가 무의미해진다.
#   - 부하 전 유휴 온도를 먼저 잰다. 시작점이 다르면 상승폭을 못 비교한다.
#   - run 전후 boot_id 를 비교한다. 도중에 리셋된 보드의 측정은 무효다.
#
# 사용법:
#   ./scripts/run-thermal-comparison.sh [부하지속초] [스레드]
#
# 결과: results/thermal-<타임스탬프>/

set -uo pipefail

DURATION="${1:-600}"
THREADS="${2:-8}"
HOSTS=(npuforge-k npuforge-q npuforge-j)
MODEL="/home/pi/npuforge-rknn-test/yolov8n-fp16.rknn"
BIN="/home/pi/npuforge-rknn-test/sustained_load_test"
REMOTE_DIR="/home/pi/thermal-run"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

STAMP="$(date +%Y%m%d-%H%M%S)"
OUT="${SCRIPT_DIR}/../results/thermal-${STAMP}"
mkdir -p "$OUT"

log()  { printf '[%s] %s\n' "$(date +%H:%M:%S)" "$*"; }
fail() { printf 'ERROR: %s\n' "$*" >&2; exit 1; }

declare -A NAME BOOT

# --- 0. 사전 검증 -----------------------------------------------------------
log "사전 검증"
for h in "${HOSTS[@]}"; do
    info=$(ssh -o ConnectTimeout=8 "$h" \
        "hostname; sha256sum '$BIN' | cut -c1-16; sha256sum '$MODEL' | cut -c1-16; cat /proc/sys/kernel/random/boot_id" 2>&1) \
        || fail "$h 접속 실패: $info"
    mapfile -t f <<<"$info"
    NAME[$h]="${f[0]}"
    BOOT[$h]="${f[3]}"
    printf '  %-12s hostname=%-7s bin=%s model=%s\n' "$h" "${f[0]}" "${f[1]}" "${f[2]}"
    echo "${h} ${f[0]} ${f[1]} ${f[2]} ${f[3]}" >> "$OUT/preflight.txt"
done

[ "${NAME[npuforge-k]}" = "king"  ] || fail "npuforge-k 가 king 이 아니다: ${NAME[npuforge-k]}"
[ "${NAME[npuforge-q]}" = "queen" ] || fail "npuforge-q 가 queen 이 아니다: ${NAME[npuforge-q]}"
[ "${NAME[npuforge-j]}" = "jack"  ] || fail "npuforge-j 가 jack 이 아니다: ${NAME[npuforge-j]}"

if [ "$(awk '{print $3, $4}' "$OUT/preflight.txt" | sort -u | wc -l)" -ne 1 ]; then
    fail "보드마다 바이너리/모델 해시가 다르다. $OUT/preflight.txt 확인"
fi
log "  hostname·해시 일치 확인"

# --- 1. 정리 및 로거 배포 ---------------------------------------------------
LOG_SECS=$(( DURATION + 120 ))
for h in "${HOSTS[@]}"; do
    # 패턴을 대괄호로 감싸지 않으면 pkill -f 가 자기 자신의 명령줄까지 매치해
    # 원격 셸을 죽인다. 그러면 뒤의 mkdir 이 실행되지 않는다.
    ssh "$h" "pkill -f '[t]hread_safety_test' >/dev/null 2>&1; pkill -f '[t]hermal-logger' >/dev/null 2>&1; mkdir -p $REMOTE_DIR; rm -f $REMOTE_DIR/*.log" >/dev/null 2>&1
    ssh "$h" "test -d $REMOTE_DIR" || fail "$h 에 $REMOTE_DIR 를 만들지 못했다"
    scp -q "$SCRIPT_DIR/thermal-logger.sh" "$h:$REMOTE_DIR/thermal-logger.sh" || fail "$h 로거 복사 실패"
    ssh "$h" "chmod +x $REMOTE_DIR/thermal-logger.sh"
done

# --- 2. 유휴 기준선 ---------------------------------------------------------
log "유휴 안정화 30초"
sleep 30
log "유휴 기준선"
for h in "${HOSTS[@]}"; do
    read -r npu load < <(ssh "$h" 'printf "%.1f %s\n" $(awk "{print \$1/1000}" /sys/class/thermal/thermal_zone4/temp) $(cut -d" " -f1 /proc/loadavg)')
    printf '  %-6s NPU %s°C  load %s\n' "${NAME[$h]}" "$npu" "$load"
    echo "${NAME[$h]} $npu $load" >> "$OUT/baseline.txt"
done

# --- 3. 로거 시작 (부하보다 먼저) -------------------------------------------
log "온도 로거 시작"
for h in "${HOSTS[@]}"; do
    ssh -n "$h" "setsid nohup $REMOTE_DIR/thermal-logger.sh $REMOTE_DIR/temp.log $LOG_SECS >/dev/null 2>&1 < /dev/null &"
done
sleep 3

# --- 4. 동시 부하 -----------------------------------------------------------
# `sustained_load_test` 는 시간으로 끊는다. 반복 횟수로 끊으면 보드마다
# 처리속도가 달라 부하 구간 길이가 어긋나고 평탄역 비교가 무의미해진다.
# (`thread_safety_test` 를 쓰면 안 된다 — 스윕 전에 단일/2스레드 기준선을
#  먼저 돌리므로 목표 스레드 수에 도달하기까지 시간이 걸린다.)
log "부하 시작 (${THREADS}스레드, ${DURATION}초)"
# 절대경로로 띄운다. `cd DIR && setsid nohup ... &` 형태는 ssh 가 즉시
# 끊길 때 백그라운드 서브셸이 setsid 에 닿기 전에 죽는 경합이 있다.
# 실측에서 이 형태는 조용히 실행되지 않았다.
for h in "${HOSTS[@]}"; do
    ssh -n "$h" "setsid nohup $BIN $MODEL $DURATION $THREADS > $REMOTE_DIR/load.log 2>&1 < /dev/null &"
done
START=$(date +%s)

# 실제로 떴는지 확인한다. 백그라운드 실행은 실패해도 아무 신호가 없어서,
# 확인하지 않으면 "부하 없는 상태의 온도"를 15분 동안 측정하게 된다.
sleep 5
NOT_RUNNING=""
for h in "${HOSTS[@]}"; do
    running=$(ssh -n "$h" 'n=0; for p in /proc/[0-9]*; do
        case "$(readlink "$p/exe" 2>/dev/null)" in *sustained_load_test) n=$((n+1));; esac
      done; echo $n' 2>/dev/null)
    [ "${running:-0}" = "0" ] && NOT_RUNNING="$NOT_RUNNING ${NAME[$h]}"
done
if [ -n "$NOT_RUNNING" ]; then
    fail "부하가 시작되지 않은 노드가 있다:$NOT_RUNNING (로그: $REMOTE_DIR/load.log)"
fi
log "  세 노드 모두 부하 실행 확인"

elapsed=0
while [ "$elapsed" -lt "$DURATION" ]; do
    sleep 60
    elapsed=$(( $(date +%s) - START ))
    line=$(printf '  +%4ds ' "$elapsed")
    for h in "${HOSTS[@]}"; do
        t=$(ssh -o ConnectTimeout=5 "$h" 'awk "{printf \"%.1f\", \$1/1000}" /sys/class/thermal/thermal_zone4/temp' 2>/dev/null || echo "?")
        line+=$(printf '%s=%s ' "${NAME[$h]}" "$t")
    done
    echo "$line"
done

log "부하 종료. 60초 하강 구간 기록"
sleep 60

# --- 5. 수집 및 무효 판정 ---------------------------------------------------
log "결과 수집"
for h in "${HOSTS[@]}"; do
    n="${NAME[$h]}"
    scp -q "$h:$REMOTE_DIR/temp.log" "$OUT/${n}-temp.log" 2>/dev/null || echo "  $n 온도 로그 없음"
    scp -q "$h:$REMOTE_DIR/load.log" "$OUT/${n}-load.log" 2>/dev/null || echo "  $n 부하 로그 없음"

    after=$(ssh "$h" 'cat /proc/sys/kernel/random/boot_id')
    if [ "$after" != "${BOOT[$h]}" ]; then
        echo "$n 이 run 도중 재부팅했다. 이 보드의 측정은 무효다." | tee -a "$OUT/INVALID.txt"
    fi
done

log "완료: $OUT"
echo "$OUT"
