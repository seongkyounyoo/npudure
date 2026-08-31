#!/usr/bin/env bash
# S3.7 — 노드당 커넥션 수 sweep, 그리고 최적 N 위에서 RPS 재시도.
#
# 왜 하는가
#   S3.6 은 커넥션 1 → 4 만 비교해 +21.5% 를 봤다. 4 가 최적이라는 근거는
#   없다. 그리고 **무조건 많을수록 좋은 것도 아니다** — 어느 지점부터는
#   커넥션 관리 비용과 queueing 으로 다시 꺾일 수 있다.
#
#   더 중요한 것: S3.6 에서 처리량은 +21.5% 인데 **p95 는 46% 나빠졌다**
#   (393 → 573 ms). 최적점을 최대 처리량으로 정하면 안 된다.
#
#     4ch = 140 inf/s, p95 573
#     8ch = 148 inf/s, p95 900
#
#   이면 8ch 가 더 좋은 시스템이라고 할 수 없다. 그래서 이 스크립트는
#   **p50·p95·p99 를 전부 기록**하고, 최적점은 처리량–tail 트레이드오프로
#   고른다(analyze-connection-sweep.py 가 두 축을 같이 낸다).
#
# 두 단계
#   phase 1  sweep : 커넥션 1/2/4/8/16 → 곡선
#   phase 2  rps   : 확정한 N 위에서 RPS off/on A/B
#
#   phase 2 가 중요한 이유 — S3.5b 에서 RPS 가 무효였던 것은 흐름이 하나뿐
#   이라 분산할 대상이 없었기 때문이다. 이제 흐름이 N 개다. 그리고 S3.6 의
#   4ch 조건에서 CPU0 는 busy 81% / soft 74% 였다.
#     오르면   → 단일 커넥션 제약을 풀자 NIC 처리 병목이 드러난 것
#     그대로면 → CPU0 softirq 는 상관관계일 뿐 처리량 limiter 가 아니다
#                (S3.5b 보다 훨씬 강한 배제)
#
# 사용법
#   bash scripts/run-connection-sweep.sh sweep [라운드수]      기본 5
#   bash scripts/run-connection-sweep.sh rps <N> [라운드수]    기본 5
#
#   운영점을 지정할 때 (S3.7c 는 conn2 @ c12):
#     NPUFORGE_CONC=12 bash scripts/run-connection-sweep.sh rps 2 5
set -u

. "$(dirname "${BASH_SOURCE[0]}")/lib/remote.sh"

MODE=${1:?"sweep | rps"}
if [ "$MODE" = "rps" ]; then
    RPS_N=${2:?"rps 모드는 커넥션 수 N 이 필요하다"}
    ROUNDS=${3:-5}
else
    ROUNDS=${2:-5}
fi

DATE=$(date +%Y%m%d)
OUT="results/connection-sweep-${DATE}"
REMOTE=/tmp/csweep

BENCH=/root/npuforge/target/release/npuforge-bench
SCHED_BIN=/root/npuforge/target/release/npuforge-scheduler
SCHED_CFG=/root/scheduler-s37.toml
SCHED_URL=http://127.0.0.1:50051
CONC="${NPUFORGE_CONC:-32}"
DUR=60
PROF=40
WARM=10

NODE_BIN=/home/pi/npuforge/npuforge-node.s36
NODE_CFG=/home/pi/npuforge/node-s37.toml
COLLECTOR=/home/pi/node-profile-collect.sh
RPS_PATH=/sys/class/net/eth0/queues/rx-0/rps_cpus

# S3.6 결론: window 는 기본값을 쓴다. 64MB 급 확대는 -36.3% 였다.
CONNS_LIST="1 2 4 8 16"

FREEZE=$(git rev-parse --short HEAD 2>/dev/null || echo unknown)
mkdir -p "$OUT/raw"

# 모드별 출력 파일. run_one 이 이 변수를 쓴다 — 예전엔 헤더만 여기에 쓰고
# 데이터 행은 results.csv 로 하드코딩돼 있어서, rps 모드가 sweep 결과
# 파일에 덧붙었다. 조용한 오염이라 분석기가 "유효한 run 없음" 을 낼 때까지
# 몰랐다.
CSV="$OUT/raw/results.csv"
[ "$MODE" = "rps" ] && CSV="$OUT/raw/results-rps.csv"

sched_stop() { ssh -o BatchMode=yes npuforge-server 'pkill -9 npuforge-schedu 2>/dev/null; exit 0' 2>/dev/null; }
node_stop()  { ssh -o BatchMode=yes npuforge-k 'pkill -9 npuforge-node 2>/dev/null; exit 0' 2>/dev/null; }

set_rps() {
    npuforge_ssh_sudo npuforge-k "sh -c 'echo $1 > $RPS_PATH'" >/dev/null
    local got
    got=$(ssh -o BatchMode=yes npuforge-k "cat $RPS_PATH" 2>/dev/null | tr -d '\r ')
    [ "$got" = "$1" ] || { echo "  !! rps_cpus 설정 실패 (원함 $1, 실제 $got)"; return 1; }
}

# 커넥션 수 N 으로 스케줄러·노드를 재기동한다. window 는 건드리지 않는다.
apply() {
    local conns=$1
    sched_stop; node_stop
    sleep 4

    {
        echo
        echo "[transport]"
        echo "node_connections = $conns"
    } | ssh -o BatchMode=yes npuforge-server \
        "cp /root/scheduler-base.toml $SCHED_CFG && cat >> $SCHED_CFG" 2>/dev/null
    ssh -o BatchMode=yes npuforge-k "cp /home/pi/npuforge/node.toml $NODE_CFG" 2>/dev/null

    ssh -o BatchMode=yes npuforge-server \
        "setsid nohup $SCHED_BIN --config $SCHED_CFG >/tmp/sched-s37.log 2>&1 </dev/null & disown; exit 0" 2>/dev/null
    sleep 3
    ssh -o BatchMode=yes npuforge-k \
        "setsid nohup $NODE_BIN --config $NODE_CFG >/tmp/node-s37.log 2>&1 </dev/null & disown; exit 0" 2>/dev/null
    sleep 12

    # 설정이 조용히 무시되면 sweep 이 아니라 같은 조건 5번이 된다.
    if ! ssh -o BatchMode=yes npuforge-server "grep -q node_connections /tmp/sched-s37.log" 2>/dev/null; then
        echo "  !! 스케줄러가 전송 설정을 읽지 못했다"
        ssh -o BatchMode=yes npuforge-server 'tail -5 /tmp/sched-s37.log' 2>/dev/null
        return 1
    fi
    if ! ssh -o BatchMode=yes npuforge-k "pgrep npuforge-node >/dev/null" 2>/dev/null; then
        echo "  !! 노드가 뜨지 않았다"
        ssh -o BatchMode=yes npuforge-k 'tail -5 /tmp/node-s37.log' 2>/dev/null
        return 1
    fi
    return 0
}

# label / 커넥션수 / rps / 라운드
run_one() {
    local cond=$1 conns=$2 rps=$3 round=$4
    if ! apply "$conns"; then
        echo "$cond,$round,$conns,$rps,,,,,,,,," >> "$CSV"
        return 1
    fi
    if [ "$rps" != "-" ]; then set_rps "$rps" || return 1; fi

    local pid
    pid=$(ssh -o BatchMode=yes npuforge-k 'pgrep npuforge-node | head -1' 2>/dev/null | tr -d '\r')

    ssh -o BatchMode=yes npuforge-server \
        "mkdir -p $REMOTE/rt; rm -f $REMOTE/rt/*.json; \
         $BENCH --scheduler $SCHED_URL --model yolov8n --concurrency $CONC \
         --duration $DUR --policy round-robin --out $REMOTE/rt >/dev/null 2>&1; \
         f=\$(ls $REMOTE/rt/*.json 2>/dev/null | head -1); \
         [ -n \"\$f\" ] && mv \"\$f\" $REMOTE/${cond}_r${round}.json" 2>/dev/null &
    local BJ=$!

    sleep "$WARM"
    local nconn
    nconn=$(ssh -o BatchMode=yes npuforge-k \
        "ss -tn state established '( sport = :51001 )' 2>/dev/null | tail -n +2 | wc -l" 2>/dev/null | tr -d '\r')
    ssh -o BatchMode=yes npuforge-k "bash $COLLECTOR ${cond}_r${round} $pid $PROF $REMOTE" >/dev/null 2>&1
    wait $BJ 2>/dev/null

    # p99 를 S3.6 보다 추가로 뽑는다. tail 로 최적점을 정하려면 필요하다.
    local m
    m=$(ssh -o BatchMode=yes npuforge-server \
        "python3 -c \"
import json
d=json.load(open('$REMOTE/${cond}_r${round}.json'))
s=d['summary']; L=s['latency']; sb=s['stage_breakdown']
print('%.1f,%.2f,%.2f,%.2f,%.2f,%.2f,%.3f,%.4f' % (
  s['throughput'], L['p50']/1000, L['p95']/1000, L['p99']/1000, L['max']/1000,
  sb['network_to_node']['p50']/1000, sb['node_queue']['p50']/1000, s['error_rate']))
\"" 2>/dev/null | tr -d '\r')

    echo "$cond,$round,$conns,$rps,$nconn,$m" >> "$CSV"
    printf '  r%s  %-12s conn=%-2s rps=%-4s tcp=%-2s -> %s\n' \
        "$round" "$cond" "$conns" "$rps" "${nconn:-?}" "${m:-실패}"
    sleep 8
}

echo "=== S3.7 ($MODE, freeze $FREEZE, ${ROUNDS}라운드) ==="
ssh -o BatchMode=yes npuforge-q 'pkill -9 npuforge-node 2>/dev/null; exit 0' 2>/dev/null
ssh -o BatchMode=yes npuforge-j 'pkill -9 npuforge-node 2>/dev/null; exit 0' 2>/dev/null
scp -q -o BatchMode=yes scripts/node-profile-collect.sh npuforge-k:$COLLECTOR
ssh -o BatchMode=yes npuforge-k "chmod +x $COLLECTOR; mkdir -p $REMOTE" 2>/dev/null
ssh -o BatchMode=yes npuforge-server "mkdir -p $REMOTE" 2>/dev/null

ORIG_RPS=$(ssh -o BatchMode=yes npuforge-k "cat $RPS_PATH" 2>/dev/null | tr -d '\r ')

echo "cond,round,conns,rps,tcp_conns,throughput,p50_ms,p95_ms,p99_ms,max_ms,net_to_node_ms,node_queue_ms,error_rate" > "$CSV"

if [ "$MODE" = "sweep" ]; then
    for r in $(seq 1 "$ROUNDS"); do
        # 라운드마다 순서를 뒤집는다. 온도·시간 경과가 한 조건에 몰리면
        # sweep 이 아니라 '먼저 잰 것 vs 나중 잰 것' 비교가 된다.
        if [ $((r % 2)) -eq 1 ]; then order="$CONNS_LIST"; else order="16 8 4 2 1"; fi
        for n in $order; do
            run_one "c${n}" "$n" "-" "$r"
        done
    done
else
    for r in $(seq 1 "$ROUNDS"); do
        if [ $((r % 2)) -eq 1 ]; then
            run_one "rpsoff" "$RPS_N" 00 "$r"; run_one "rpson" "$RPS_N" fe "$r"
        else
            run_one "rpson" "$RPS_N" fe "$r"; run_one "rpsoff" "$RPS_N" 00 "$r"
        fi
    done
fi

echo "--- 원상복구 ---"
set_rps "$ORIG_RPS" >/dev/null 2>&1
apply 1 >/dev/null 2>&1
ssh -o BatchMode=yes npuforge-q \
    "setsid nohup /home/pi/npuforge/npuforge-node --config /home/pi/npuforge/node.toml >/tmp/node.log 2>&1 </dev/null & disown; exit 0" 2>/dev/null
sleep 3
for h in npuforge-k npuforge-q npuforge-j; do
    printf '  %-16s ' "$h"
    ssh -o BatchMode=yes "$h" 'pgrep npuforge-node >/dev/null && echo running || echo DOWN' 2>/dev/null | tr -d '\r'
done
echo "  rps_cpus: $(ssh -o BatchMode=yes npuforge-k "cat $RPS_PATH" 2>/dev/null | tr -d '\r')"

scp -q -r -o BatchMode=yes "npuforge-k:$REMOTE" "$OUT/raw/profile" 2>/dev/null
ssh -o BatchMode=yes npuforge-server "cd $REMOTE && tar cf - *.json" 2>/dev/null \
    | tar xf - -C "$OUT/raw" 2>/dev/null

printf 'freeze_commit=%s\nmode=%s\nconcurrency=%s\nduration=%s\nrounds=%s\nconns=%s\n' \
    "$FREEZE" "$MODE" "$CONC" "$DUR" "$ROUNDS" "$CONNS_LIST" > "$OUT/meta-${MODE}.txt"

echo
PYTHONIOENCODING=utf-8 python scripts/analyze-connection-sweep.py "$CSV" 2>&1 || cat "$CSV"
echo "결과: $OUT/"
