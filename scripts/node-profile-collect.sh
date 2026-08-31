#!/usr/bin/env bash
# 보드에서 실행되는 수집기. S3.5 transport profiling 용.
#
# 왜 이 스크립트가 따로 있는가
#   ssh 인라인으로 쓰면 따옴표 중첩 때문에 조용히 깨진다. 측정 스크립트가
#   조용히 깨지면 결과가 아니라 착각이 남는다(worklog §2.20 과 같은 종류).
#
# 무엇을 재는가
#   `01-TECHSPEC.md` §15.4 가 요구하는 io_uring 판단 지표의 "before" 값이다.
#   현재 저장소에 이 값이 하나도 없다(S2 raw 30건에 CPU 항목 없음).
#
#     syscalls/request      /proc/PID/io 의 syscr+syscw
#     ctx switches/request  /proc/PID/task/*/status 합
#     CPU cycles/request    /proc/PID/stat 의 utime+stime × 고정 클럭
#     코어별 %sys %soft     mpstat  ← 네트워크 스택이 어디서 도는지
#     NIC IRQ / softirq     /proc/interrupts, /proc/softirqs
#
#   원본을 그대로 남기고 계산은 개발 PC 에서 한다. 보드에서 집계해 버리면
#   나중에 다른 각도로 다시 못 본다.
#
# 사용법
#   node-profile-collect.sh <label> <pid> <duration_s> <outdir>
#
#   pid 가 0 이면 프로세스별 수집을 건너뛰고 시스템 전역만 잰다(idle 조건).
set -u

LABEL=${1:?label}
PID=${2:?pid (0 이면 시스템만)}
DUR=${3:?duration_s}
OUT=${4:?outdir}

D="$OUT/$LABEL"
mkdir -p "$D"

# ── 스냅샷 ────────────────────────────────────────────────────────────
# 전역과 프로세스별을 한 함수로 묶는다. before/after 가 대칭이어야
# 델타가 의미를 갖는다.
snap() {
    local when=$1
    cat /proc/stat            > "$D/stat.$when"
    cat /proc/net/dev         > "$D/netdev.$when"
    cat /proc/interrupts      > "$D/interrupts.$when"
    cat /proc/softirqs        > "$D/softirqs.$when"
    cat /proc/uptime          > "$D/uptime.$when"
    if [ "$PID" != "0" ] && [ -d "/proc/$PID" ]; then
        cat "/proc/$PID/io"   > "$D/pio.$when"   2>/dev/null
        cat "/proc/$PID/stat" > "$D/pstat.$when" 2>/dev/null
        ls "/proc/$PID/task" | wc -l > "$D/nthreads.$when"
        # status 의 ctxt_switches 는 메인 스레드 값만 나온다.
        # 프로세스 전체를 보려면 task/* 를 전부 더해야 한다.
        for t in /proc/"$PID"/task/*/status; do
            grep -H -E 'ctxt_switches' "$t" 2>/dev/null
        done > "$D/ctxt.$when"
    fi
    cat /sys/class/thermal/thermal_zone0/temp > "$D/temp.$when" 2>/dev/null
}

snap before

# ── 샘플러 ────────────────────────────────────────────────────────────
# mpstat 은 코어별로 %usr %sys %irq %soft %idle 를 준다. NIC IRQ 가
# CPU0 한 곳에만 붙어 있으므로(단일 큐) 코어별로 봐야 병목이 보인다.
mpstat -P ALL 1 "$DUR" > "$D/mpstat.txt" 2>&1 &
MP=$!

if [ "$PID" != "0" ] && [ -d "/proc/$PID" ]; then
    # -t 로 스레드별. tokio async 워커(네트워크)와 blocking 풀(RKNN)이
    # 같은 8 코어를 나눠 쓰는지 확인하는 것이 목적이다.
    pidstat -t -p "$PID" 1 "$DUR" > "$D/pidstat.txt" 2>&1 &
    PS=$!
fi

wait $MP 2>/dev/null
[ -n "${PS:-}" ] && wait $PS 2>/dev/null

snap after

echo "collected: $LABEL (pid=$PID dur=${DUR}s) -> $D"
