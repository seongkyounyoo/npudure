#!/usr/bin/env bash
# S3.9a — scale-out efficiency loss profiling.
#
# 질문 하나만 묻는다
#   optimized 3N 에서 scaling efficiency 가 95.3% 로 내려갔다(S3.8).
#   **사라진 약 4.7% 가 shared path 의 어디서 생기는가?**
#
# 범위를 넓히지 않는다
#   새 concurrency sweep 을 섞지 않는다. S3.8 이 이미 찾아 둔 **각자의
#   운영점**을 그대로 쓴다. sweep 을 섞으면 질문이 다시 흐려진다.
#
#     1N @ c12   2N @ c24   3N @ c36     (전부 노드당 커넥션 2개, 노드당 c12)
#
#   노드당 부하가 셋 다 동일하므로, 서버 쪽 비용이 **노드 수와 함께 어떻게
#   증가하는지**만 보면 된다.
#
# 서버는 보드와 다르다
#   24코어 / 10GbE / **RX 큐 24개**(보드는 1개, RSS 로 분산됨).
#   그래서 S3.5b·S3.7c 에서 보드에 걸렸던 '단일 큐 + CPU0 편중' 문제는
#   서버에 없을 가능성이 높다 — 그 예상이 맞는지도 이 실험이 확인한다.
#
# 사용법
#   bash scripts/run-scaleout-profile.sh [반복수]      기본 3
set -u

REPS=${1:-3}

DATE=$(date +%Y%m%d)
OUT="results/scaleout-profile-${DATE}"
REMOTE=/tmp/s39a

BENCH=/root/npuforge/target/release/npuforge-bench
SCHED_BIN=/root/npuforge/target/release/npuforge-scheduler
SCHED_CFG=/root/scheduler-s39.toml
SCHED_URL=http://127.0.0.1:50051
DUR=60
PROF=40          # 수집 창 (부하 안쪽)
WARM=10          # 부하 시작 후 이만큼 지나 수집 시작
CONNS_PER_NODE=2

NODE_BIN=/home/pi/npuforge/npuforge-node.s36
NODE_CFG=/home/pi/npuforge/node-s39.toml
SRV_COLLECTOR=/root/server-profile-collect.sh
NODE_COLLECTOR=/home/pi/node-profile-collect.sh

# S3.8 이 확정한 운영점. 노드당 c12 로 동일하다.
declare -A OPCONC=( [1]=12 [2]=24 [3]=36 )

FREEZE=$(git rev-parse --short HEAD 2>/dev/null || echo unknown)
mkdir -p "$OUT/raw"
CSV="$OUT/raw/results.csv"

sched_stop() { ssh -o BatchMode=yes npuforge-server 'pkill -9 npuforge-schedu 2>/dev/null; exit 0' 2>/dev/null; }
node_stop()  { ssh -o BatchMode=yes "$1" 'pkill -9 npuforge-node 2>/dev/null; exit 0' 2>/dev/null; }
node_start() {
    ssh -o BatchMode=yes "$1" \
        "pgrep npuforge-node >/dev/null 2>&1 || { setsid nohup $NODE_BIN --config $NODE_CFG \
         >>/home/pi/npuforge/node-s39.log 2>&1 </dev/null & disown; }; exit 0" 2>/dev/null
}

setup() {
    local n=$1
    sched_stop
    node_stop npuforge-k; node_stop npuforge-q; node_stop npuforge-j
    sleep 4
    {
        echo
        echo "[transport]"
        echo "node_connections = $CONNS_PER_NODE"
    } | ssh -o BatchMode=yes npuforge-server \
        "cp /root/scheduler-base.toml $SCHED_CFG && cat >> $SCHED_CFG" 2>/dev/null
    for h in k q j; do
        ssh -o BatchMode=yes "npuforge-$h" "cp /home/pi/npuforge/node.toml $NODE_CFG" 2>/dev/null
    done
    ssh -o BatchMode=yes npuforge-server \
        "setsid nohup $SCHED_BIN --config $SCHED_CFG >/tmp/sched-s39.log 2>&1 </dev/null & disown; exit 0" 2>/dev/null
    sleep 3
    node_start npuforge-k
    [ "$n" -ge 2 ] && node_start npuforge-q
    [ "$n" -ge 3 ] && node_start npuforge-j
    sleep 12
}

# S3.8 에서 이 검증이 6개 구성을 걸렀다. 프로세스 존재 != 트래픽 수신.
verify_nodes() {
    local n=$1
    ssh -o BatchMode=yes npuforge-server \
        "mkdir -p $REMOTE/probe; rm -f $REMOTE/probe/*.json; \
         $BENCH --scheduler $SCHED_URL --model yolov8n --concurrency $((n * 4)) \
         --duration 10 --policy round-robin --out $REMOTE/probe >/dev/null 2>&1" 2>/dev/null
    local seen cnt
    seen=$(ssh -o BatchMode=yes npuforge-server "python3 -c \"
import json,glob
f=sorted(glob.glob('$REMOTE/probe/*.json'))
if not f: print('NONE'); raise SystemExit
d=json.load(open(f[-1]))
print(','.join(sorted(p['node_id'] for p in d['summary']['per_node'] if p['count']>0)))
\"" 2>/dev/null | tr -d '\r')
    cnt=$(echo "$seen" | tr ',' '\n' | grep -c . || echo 0)
    if [ "$cnt" != "$n" ]; then
        echo "  !! 노드 수 불일치 — expected=$n observed=$cnt ($seen). 건너뜀"
        return 1
    fi
    echo "  노드 검증 OK — $cnt개 ($seen)"
    return 0
}

run_one() {
    local n=$1 rep=$2
    local conc=${OPCONC[$n]}
    local tag="n${n}_r${rep}"

    ssh -o BatchMode=yes npuforge-server \
        "mkdir -p $REMOTE/rt; rm -f $REMOTE/rt/*.json; \
         $BENCH --scheduler $SCHED_URL --model yolov8n --concurrency $conc \
         --duration $DUR --policy round-robin --out $REMOTE/rt >/dev/null 2>&1; \
         f=\$(ls $REMOTE/rt/*.json 2>/dev/null | head -1); \
         [ -n \"\$f\" ] && mv \"\$f\" $REMOTE/${tag}.json" 2>/dev/null &
    local BJ=$!

    sleep "$WARM"
    # 서버와 king 을 동시에 수집한다. 서버 쪽 비용을 노드 쪽과 나란히 놓아야
    # '서버가 늘었나 노드가 줄었나' 를 구분할 수 있다.
    ssh -o BatchMode=yes npuforge-server \
        "bash $SRV_COLLECTOR $tag $PROF $REMOTE/prof" >/dev/null 2>&1 &
    local SJ=$!
    local kpid
    kpid=$(ssh -o BatchMode=yes npuforge-k 'pgrep npuforge-node | head -1' 2>/dev/null | tr -d '\r')
    ssh -o BatchMode=yes npuforge-k \
        "bash $NODE_COLLECTOR $tag $kpid $PROF $REMOTE" >/dev/null 2>&1
    wait $SJ 2>/dev/null
    wait $BJ 2>/dev/null

    local m
    m=$(ssh -o BatchMode=yes npuforge-server \
        "python3 -c \"
import json
d=json.load(open('$REMOTE/${tag}.json'))
s=d['summary']; L=s['latency']; sb=s['stage_breakdown']
pn=[p for p in s['per_node'] if p['count']>0]
sh=sorted(p['share'] for p in pn)
print('%d,%.1f,%.2f,%.2f,%.2f,%.3f,%.3f,%.3f,%.4f,%.2f' % (
  len(pn), s['throughput'], L['p50']/1000, L['p95']/1000, L['p99']/1000,
  sb['scheduler_queue']['p50']/1000, sb['scheduler_route']['p50']/1000,
  sb['node_queue']['p50']/1000, s['error_rate'],
  (sh[-1]-sh[0])*100 if sh else 0))
\"" 2>/dev/null | tr -d '\r')

    echo "$n,$conc,$rep,$m" >> "$CSV"
    printf '  rep%s  %dN c%-3s -> %s\n' "$rep" "$n" "$conc" "${m:-실패}"
    sleep 6
}

echo "=== S3.9a scale-out profiling (freeze $FREEZE, ${CONNS_PER_NODE} conn/node, ${REPS}회) ==="
scp -q -o BatchMode=yes scripts/server-profile-collect.sh npuforge-server:$SRV_COLLECTOR
scp -q -o BatchMode=yes scripts/node-profile-collect.sh npuforge-k:$NODE_COLLECTOR
ssh -o BatchMode=yes npuforge-server "chmod +x $SRV_COLLECTOR; mkdir -p $REMOTE/prof" 2>/dev/null
ssh -o BatchMode=yes npuforge-k "chmod +x $NODE_COLLECTOR; mkdir -p $REMOTE" 2>/dev/null

echo "nodes,concurrency,rep,observed_nodes,throughput,p50_ms,p95_ms,p99_ms,sched_queue_ms,sched_route_ms,node_queue_ms,error_rate,balance_pp" > "$CSV"

for rep in $(seq 1 "$REPS"); do
    case $(( (rep - 1) % 3 )) in
        0) order="1 2 3";;
        1) order="2 3 1";;
        2) order="3 1 2";;
    esac
    for n in $order; do
        echo "--- rep $rep · ${n}노드 @ c${OPCONC[$n]} (운영점) ---"
        setup "$n"
        verify_nodes "$n" || continue
        run_one "$n" "$rep"
    done
done

echo "--- 원상복구 (3노드, 기본 설정) ---"
sched_stop
node_stop npuforge-k; node_stop npuforge-q; node_stop npuforge-j
sleep 4
ssh -o BatchMode=yes npuforge-server \
    "setsid nohup $SCHED_BIN --config /root/scheduler.toml >/tmp/sched.log 2>&1 </dev/null & disown; exit 0" 2>/dev/null
sleep 3
. "$(dirname "${BASH_SOURCE[0]}")/lib/remote.sh"
npuforge_restore_cluster

scp -q -r -o BatchMode=yes "npuforge-server:$REMOTE/prof" "$OUT/raw/server" 2>/dev/null
scp -q -r -o BatchMode=yes "npuforge-k:$REMOTE" "$OUT/raw/node" 2>/dev/null
ssh -o BatchMode=yes npuforge-server "cd $REMOTE && tar cf - *.json" 2>/dev/null \
    | tar xf - -C "$OUT/raw" 2>/dev/null
printf 'freeze_commit=%s\nconns_per_node=%s\nduration=%s\nprof=%s\nreps=%s\nop_conc=1N:12,2N:24,3N:36\n' \
    "$FREEZE" "$CONNS_PER_NODE" "$DUR" "$PROF" "$REPS" > "$OUT/meta.txt"

echo
PYTHONIOENCODING=utf-8 python scripts/analyze-scaleout-profile.py "$OUT" 2>&1 || cat "$CSV"
echo "결과: $OUT/"
