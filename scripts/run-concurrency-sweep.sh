#!/usr/bin/env bash
# S3.7b — 후보 커넥션 수에 대한 concurrency sweep.
#
# 왜 필요한가
#   S3.7a 는 부하를 c32 로 고정했다. 커넥션 수의 순수 효과 비교로는 좋지만
#   **각 설정의 ceiling 은 아니다.** 커넥션을 늘리면 saturation concurrency
#   가 c32 위로 이동했을 수 있다.
#
#     c2 @ c32 = 134.4   ← 아직 포화가 아닐 수 있다
#     c4 @ c32 = 139.5   ← 아직 포화가 아닐 수 있다
#
#   실제 ceiling 이 c2@c48 = 145, c4@c48 = 147 처럼 나오면 S3.7a 의 서열이
#   바뀔 수도 있다. 그래서 후보 1~2개만 골라 부하를 흔들어 본다.
#
#   운영점은 최대 처리량이 아니라 **처리량–tail 트레이드오프**로 정한다.
#   그래서 p50/p95/p99/max 를 전부 남긴다.
#
# 왜 커넥션 수별로 묶어 도는가
#   concurrency 는 bench 인자라 프로세스 재기동이 필요 없다. 커넥션 수만
#   재기동이 필요하다. 커넥션 수 블록 안에서 concurrency 를 훑으면 재기동이
#   rep 당 후보 수만큼으로 줄어 30분 넘게 아낀다.
#   대신 두 후보가 시간상 블록으로 갈리므로, **rep 마다 후보 순서를 뒤집어**
#   온도·시간 드리프트가 한쪽에 몰리지 않게 한다.
#
# 사용법
#   bash scripts/run-concurrency-sweep.sh [반복수]      기본 3
#
#   탐색 범위를 바꿀 때 (첫 sweep 이 엉뚱한 방향이었을 때 쓴다):
#     NPUFORGE_CONCS="8 12 16 20 24" NPUFORGE_LABEL=low bash scripts/run-concurrency-sweep.sh 3
set -u

REPS=${1:-3}
LABEL=${NPUFORGE_LABEL:-}

DATE=$(date +%Y%m%d)
OUT="results/concurrency-sweep-${DATE}${LABEL:+-$LABEL}"
REMOTE=/tmp/ccsweep

BENCH=/root/npuforge/target/release/npuforge-bench
SCHED_BIN=/root/npuforge/target/release/npuforge-scheduler
SCHED_CFG=/root/scheduler-s37b.toml
SCHED_URL=http://127.0.0.1:50051
DUR=60

NODE_BIN=/home/pi/npuforge/npuforge-node.s36
NODE_CFG=/home/pi/npuforge/node-s37b.toml

# 후보. conn1 baseline 도 같은 범위에서 재야 공정 비교가 된다 —
# 그러지 않으면 커넥션 수와 concurrency 가 동시에 바뀌어 인과가 섞인다.
CANDIDATES="${NPUFORGE_CONNS:-2 4}"
# 첫 sweep(24~64)에서 두 후보 모두 c24 가 최고였다 — 즉 포화점이 c24 이하다.
# 부하를 더 주면 처리량은 그대로인데 tail 만 커진다. 아래로 훑어야 한다.
CONCS="${NPUFORGE_CONCS:-24 32 40 48 64}"

FREEZE=$(git rev-parse --short HEAD 2>/dev/null || echo unknown)
mkdir -p "$OUT/raw"
CSV="$OUT/raw/results.csv"

sched_stop() { ssh -o BatchMode=yes npuforge-server 'pkill -9 npuforge-schedu 2>/dev/null; exit 0' 2>/dev/null; }
node_stop()  { ssh -o BatchMode=yes npuforge-k 'pkill -9 npuforge-node 2>/dev/null; exit 0' 2>/dev/null; }

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
        "setsid nohup $SCHED_BIN --config $SCHED_CFG >/tmp/sched-s37b.log 2>&1 </dev/null & disown; exit 0" 2>/dev/null
    sleep 3
    ssh -o BatchMode=yes npuforge-k \
        "setsid nohup $NODE_BIN --config $NODE_CFG >/tmp/node-s37b.log 2>&1 </dev/null & disown; exit 0" 2>/dev/null
    sleep 12
    ssh -o BatchMode=yes npuforge-server "grep -q node_connections /tmp/sched-s37b.log" 2>/dev/null || {
        echo "  !! 스케줄러 전송 설정 실패"; return 1; }
    ssh -o BatchMode=yes npuforge-k "pgrep npuforge-node >/dev/null" 2>/dev/null || {
        echo "  !! 노드 기동 실패"; return 1; }
    return 0
}

bench_once() {
    local conns=$1 conc=$2 rep=$3
    local tag="n${conns}_c${conc}_r${rep}"
    ssh -o BatchMode=yes npuforge-server \
        "mkdir -p $REMOTE/rt; rm -f $REMOTE/rt/*.json; \
         $BENCH --scheduler $SCHED_URL --model yolov8n --concurrency $conc \
         --duration $DUR --policy round-robin --out $REMOTE/rt >/dev/null 2>&1; \
         f=\$(ls $REMOTE/rt/*.json 2>/dev/null | head -1); \
         [ -n \"\$f\" ] && mv \"\$f\" $REMOTE/${tag}.json" 2>/dev/null

    local m
    m=$(ssh -o BatchMode=yes npuforge-server \
        "python3 -c \"
import json
d=json.load(open('$REMOTE/${tag}.json'))
s=d['summary']; L=s['latency']; sb=s['stage_breakdown']
print('%.1f,%.2f,%.2f,%.2f,%.2f,%.2f,%.3f,%.4f' % (
  s['throughput'], L['p50']/1000, L['p95']/1000, L['p99']/1000, L['max']/1000,
  sb['network_to_node']['p50']/1000, sb['node_queue']['p50']/1000, s['error_rate']))
\"" 2>/dev/null | tr -d '\r')

    echo "$conns,$conc,$rep,$m" >> "$CSV"
    printf '  rep%s  conn=%s  c%-3s -> %s\n' "$rep" "$conns" "$conc" "${m:-실패}"
    sleep 6
}

echo "=== S3.7b concurrency sweep (freeze $FREEZE, 후보 [$CANDIDATES] × [$CONCS] × ${REPS}회) ==="
ssh -o BatchMode=yes npuforge-q 'pkill -9 npuforge-node 2>/dev/null; exit 0' 2>/dev/null
ssh -o BatchMode=yes npuforge-j 'pkill -9 npuforge-node 2>/dev/null; exit 0' 2>/dev/null
ssh -o BatchMode=yes npuforge-server "mkdir -p $REMOTE" 2>/dev/null

echo "conns,concurrency,rep,throughput,p50_ms,p95_ms,p99_ms,max_ms,net_to_node_ms,node_queue_ms,error_rate" > "$CSV"

for rep in $(seq 1 "$REPS"); do
    # rep 마다 후보 순서를 뒤집는다 (블록 드리프트 상쇄)
    if [ $((rep % 2)) -eq 1 ]; then
        cand_order="$CANDIDATES"
    else
        cand_order=$(echo "$CANDIDATES" | tr " " "
" | tac | tr "
" " ")
    fi
    for n in $cand_order; do
        echo "--- rep $rep · 커넥션 $n 개 ---"
        apply "$n" || continue
        # concurrency 순서도 뒤집는다
        if [ $((rep % 2)) -eq 1 ]; then
            cc_order="$CONCS"
        else
            cc_order=$(echo "$CONCS" | tr ' ' '
' | tac | tr '
' ' ')
        fi
        for c in $cc_order; do
            bench_once "$n" "$c" "$rep"
        done
    done
done

echo "--- 원상복구 ---"
apply 1 >/dev/null 2>&1
ssh -o BatchMode=yes npuforge-q \
    "setsid nohup /home/pi/npuforge/npuforge-node --config /home/pi/npuforge/node.toml >/tmp/node.log 2>&1 </dev/null & disown; exit 0" 2>/dev/null
sleep 3
for h in npuforge-k npuforge-q npuforge-j; do
    printf '  %-16s ' "$h"
    ssh -o BatchMode=yes "$h" 'pgrep npuforge-node >/dev/null && echo running || echo DOWN' 2>/dev/null | tr -d '\r'
done

ssh -o BatchMode=yes npuforge-server "cd $REMOTE && tar cf - *.json" 2>/dev/null \
    | tar xf - -C "$OUT/raw" 2>/dev/null
printf 'freeze_commit=%s\ncandidates=%s\nconcurrencies=%s\nduration=%s\nreps=%s\n' \
    "$FREEZE" "$CANDIDATES" "$CONCS" "$DUR" "$REPS" > "$OUT/meta.txt"

echo
PYTHONIOENCODING=utf-8 python scripts/analyze-concurrency-sweep.py "$CSV" 2>&1 || cat "$CSV"
echo "결과: $OUT/"
