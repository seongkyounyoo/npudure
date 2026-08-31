#!/usr/bin/env bash
# 서버(스케줄러)에서 실행되는 수집기. S3.9a 용.
#
# 왜 보드용과 따로 있는가
#   서버는 Rocky 9.4 인데 **sysstat 이 없다**(mpstat/pidstat/sar/perf 전부).
#   측정 캠페인 중에 패키지를 설치해 환경을 바꾸고 싶지 않다. 필요한 것은
#   전부 /proc 에 있으므로 원본만 떠 오고 계산은 개발 PC 에서 한다.
#
#   그리고 서버는 보드와 하드웨어가 다르다 — 24코어, 10GbE, **RX 큐 24개**
#   (보드는 1개). 그래서 큐/IRQ 분포를 코어별로 봐야 한다.
#
# 무엇을 답하려는가 (S3.9a)
#   optimized 3N 에서 scaling efficiency 가 95.3% 로 내려갔다. 사라진 약
#   4.7% 가 shared path 의 **어디서** 생기는가.
#     - 서버 CPU 가 포화하는가 (코어별 busy·softirq)
#     - 10G 링크가 막는가 (RX/TX 실사용률, drop)
#     - 스케줄러 프로세스가 직렬화 지점인가 (스레드별 CPU)
#     - 커넥션/스트림 상태가 나빠지는가 (ss: rtt, cwnd, retrans)
#
# 사용법
#   server-profile-collect.sh <label> <duration_s> <outdir>
set -u

LABEL=${1:?label}
DUR=${2:?duration_s}
OUT=${3:?outdir}
NIC=${NPUFORGE_NIC:-enp4s0}

D="$OUT/$LABEL"
mkdir -p "$D"

PID=$(pgrep npuforge-schedu | head -1)
echo "$PID" > "$D/sched.pid"

snap() {
    local when=$1
    cat /proc/stat                > "$D/stat.$when"
    cat /proc/net/dev             > "$D/netdev.$when"
    cat /proc/interrupts          > "$D/interrupts.$when"
    cat /proc/softirqs            > "$D/softirqs.$when"
    cat /proc/uptime              > "$D/uptime.$when"
    cat /proc/net/snmp            > "$D/snmp.$when"        2>/dev/null
    cat /proc/net/netstat         > "$D/netstat.$when"     2>/dev/null
    ethtool -S "$NIC"             > "$D/ethtool.$when"     2>/dev/null

    if [ -n "$PID" ] && [ -d "/proc/$PID" ]; then
        cat "/proc/$PID/io"       > "$D/pio.$when"         2>/dev/null
        cat "/proc/$PID/stat"     > "$D/pstat.$when"       2>/dev/null
        ls "/proc/$PID/task" | wc -l > "$D/nthreads.$when"
        # 스레드별 CPU. 스케줄러 안에 직렬화 지점이 있으면 특정 스레드만
        # 튄다. 프로세스 합계만 보면 그게 안 보인다.
        for t in /proc/"$PID"/task/*/stat; do
            printf '%s ' "$(basename "$(dirname "$t")")"; cat "$t"
        done > "$D/tstat.$when" 2>/dev/null
        for t in /proc/"$PID"/task/*/status; do
            grep -H -E 'ctxt_switches' "$t" 2>/dev/null
        done > "$D/ctxt.$when"
    fi
}

# 커넥션 상태. -i 로 rtt·cwnd·retrans 까지 나온다. 노드로 나가는 것만
# 본다(50051 수신은 bench→scheduler 라 이번 질문과 다른 축이다).
snap_conn() {
    local when=$1
    ss -tin state established 2>/dev/null > "$D/ss.$when"
    ss -tn state established 2>/dev/null | grep -c '192\.168\.123\.[345]:51001' \
        > "$D/nodeconn.$when" 2>/dev/null || echo 0 > "$D/nodeconn.$when"
}

snap before
snap_conn before

# 창 중간에 한 번 더 커넥션 상태를 본다. 평균이 아니라 추이를 보기 위함.
( sleep $((DUR / 2)); snap_conn mid ) &
MIDJOB=$!

sleep "$DUR"
wait $MIDJOB 2>/dev/null

snap after
snap_conn after

echo "collected: $LABEL (sched pid=${PID:-none}, ${DUR}s) -> $D"
