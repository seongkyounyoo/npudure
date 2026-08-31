#!/usr/bin/env bash
# S3.8 — optimized gRPC scale-out 재검증.
#
# 이전 재측정과 무엇이 다른가
#   S2 는 노드당 부하를 c8 로 고정해 1/2/3 노드를 비교했다. S3.7 이 보인 것은
#   **고정 concurrency 비교가 configuration effect 가 아니라 overload
#   behavior 를 보여줄 수 있다**는 것이다(c32 는 모든 구성에게 과부하였다).
#
#   그래서 이번에는 노드 수마다 **각자의 운영점을 찾아** 비교한다.
#
#     1N + 2conn/node : concurrency sweep → operating point
#     2N + 2conn/node : concurrency sweep → operating point
#     3N + 2conn/node : concurrency sweep → operating point
#
#   그리고 각자의 98% operating point 끼리 비교해 scaling·efficiency 를 낸다.
#   같은 함정에 두 번 빠지지 않기 위한 설계다.
#
# ⚠️ 커넥션 단위 — 노드당이다
#   `[transport] node_connections` 는 **노드당** 값이다. GrpcNodePool 이
#   NodeId 마다 채널을 N 개 만든다. 클러스터 전체 합이 아니다.
#
#     1N → 2 total     2N → 4 total     3N → 6 total
#
#   전체를 2개로 고정하면 노드당 조건이 보존되지 않고 3N 에서 커넥션 공급
#   자체가 새 병목이 된다 — 완전히 다른 실험이다.
#
# ⚠️ 노드 수 검증을 측정 전에 한다
#   "프로세스가 떠 있다" 와 "실제로 트래픽을 받는다" 는 다르다. 각 구성마다
#   짧은 probe bench 를 던져 **응답한 노드 ID 분포**까지 확인하고,
#   expected == observed 가 아니면 그 구성을 건너뛴다. jack 이 죽은 채로
#   3N 을 쟀다면 조용히 2N 결과가 나왔을 것이다.
#
# 사용법
#   bash scripts/run-scaleout-optimized.sh [반복수]      기본 3
set -u

REPS=${1:-3}

DATE=$(date +%Y%m%d)
OUT="results/scaleout-optimized-${DATE}"
REMOTE=/tmp/s38

BENCH=/root/npuforge/target/release/npuforge-bench
SCHED_BIN=/root/npuforge/target/release/npuforge-scheduler
SCHED_CFG=/root/scheduler-s38.toml
SCHED_URL=http://127.0.0.1:50051
DUR=60
CONNS_PER_NODE=2        # S3.7b 운영점

NODE_BIN=/home/pi/npuforge/npuforge-node.s36
NODE_CFG=/home/pi/npuforge/node-s38.toml

# 노드 수별 concurrency 후보. S3.7b 에서 노드당 운영점이 c12 였으므로
# N×12 를 중심에 두고 아래위를 함께 훑는다. 가정하지 않고 각자 찾는다.
CONCS_1N="8 12 16 20"
CONCS_2N="16 24 32 40"
CONCS_3N="24 36 48 60"

FREEZE=$(git rev-parse --short HEAD 2>/dev/null || echo unknown)
mkdir -p "$OUT/raw"
CSV="$OUT/raw/results.csv"

sched_stop() { ssh -o BatchMode=yes npuforge-server 'pkill -9 npuforge-schedu 2>/dev/null; exit 0' 2>/dev/null; }
node_stop()  { ssh -o BatchMode=yes "$1" 'pkill -9 npuforge-node 2>/dev/null; exit 0' 2>/dev/null; }
node_start() {
    ssh -o BatchMode=yes "$1" \
        "pgrep npuforge-node >/dev/null 2>&1 || { setsid nohup $NODE_BIN --config $NODE_CFG \
         >>/home/pi/npuforge/node-s38.log 2>&1 </dev/null & disown; }; exit 0" 2>/dev/null
}

# 노드 수 N 구성 + 스케줄러 재기동
setup() {
    local n=$1
    sched_stop
    node_stop npuforge-k; node_stop npuforge-q; node_stop npuforge-j
    sleep 4     # NPU 컨텍스트 정리

    {
        echo
        echo "[transport]"
        echo "node_connections = $CONNS_PER_NODE"
    } | ssh -o BatchMode=yes npuforge-server \
        "cp /root/scheduler-base.toml $SCHED_CFG && cat >> $SCHED_CFG" 2>/dev/null

    for h in k q j; do
        ssh -o BatchMode=yes "npuforge-$h" \
            "cp /home/pi/npuforge/node.toml $NODE_CFG" 2>/dev/null
    done

    ssh -o BatchMode=yes npuforge-server \
        "setsid nohup $SCHED_BIN --config $SCHED_CFG >/tmp/sched-s38.log 2>&1 </dev/null & disown; exit 0" 2>/dev/null
    sleep 3

    node_start npuforge-k
    [ "$n" -ge 2 ] && node_start npuforge-q
    [ "$n" -ge 3 ] && node_start npuforge-j
    sleep 12    # 등록 + warmup
    return 0
}

# 실제로 N 개 노드가 트래픽을 받는지 확인한다. 프로세스 존재만으로는
# 부족하다 — 등록 실패나 주소 오타면 프로세스는 떠 있는데 요청은 안 온다.
verify_nodes() {
    local n=$1
    ssh -o BatchMode=yes npuforge-server \
        "mkdir -p $REMOTE/probe; rm -f $REMOTE/probe/*.json; \
         $BENCH --scheduler $SCHED_URL --model yolov8n --concurrency $((n * 4)) \
         --duration 10 --policy round-robin --out $REMOTE/probe >/dev/null 2>&1" 2>/dev/null
    local seen
    seen=$(ssh -o BatchMode=yes npuforge-server "python3 -c \"
import json,glob
f=sorted(glob.glob('$REMOTE/probe/*.json'))
if not f: print('NONE'); raise SystemExit
d=json.load(open(f[-1]))
pn=d['summary']['per_node']
print(','.join(sorted(p['node_id'] for p in pn if p['count']>0)))
\"" 2>/dev/null | tr -d '\r')
    local cnt
    cnt=$(echo "$seen" | tr ',' '\n' | grep -c . || echo 0)
    if [ "$cnt" != "$n" ]; then
        echo "  !! 노드 수 불일치 — expected=$n observed=$cnt ($seen). 이 구성 건너뜀"
        return 1
    fi
    echo "  노드 검증 OK — expected=$n observed=$cnt ($seen)"
    return 0
}

bench_once() {
    local n=$1 conc=$2 rep=$3
    local tag="n${n}_c${conc}_r${rep}"
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
s=d['summary']; L=s['latency']
pn=[p for p in s['per_node'] if p['count']>0]
shares=sorted(p['share'] for p in pn)
bal=(shares[-1]-shares[0])*100 if shares else 0
print('%d,%.1f,%.2f,%.2f,%.2f,%.2f,%.4f,%.2f' % (
  len(pn), s['throughput'], L['p50']/1000, L['p95']/1000, L['p99']/1000,
  L['max']/1000, s['error_rate'], bal))
\"" 2>/dev/null | tr -d '\r')

    echo "$n,$conc,$rep,$CONNS_PER_NODE,$((n * CONNS_PER_NODE)),$m" >> "$CSV"
    printf '  rep%s  %dN c%-3s -> %s\n' "$rep" "$n" "$conc" "${m:-실패}"
    sleep 6
}

echo "=== S3.8 optimized scale-out (freeze $FREEZE, ${CONNS_PER_NODE} conn/node, ${REPS}회) ==="
ssh -o BatchMode=yes npuforge-server "mkdir -p $REMOTE" 2>/dev/null
echo "nodes,concurrency,rep,conns_per_node,conns_total,observed_nodes,throughput,p50_ms,p95_ms,p99_ms,max_ms,error_rate,balance_pp" > "$CSV"

for rep in $(seq 1 "$REPS"); do
    # rep 마다 노드 수 순서를 돌린다. 시간 경과·온도가 한 구성에 몰리면
    # scale-out 비교가 아니라 '먼저 잰 것 vs 나중 잰 것' 비교가 된다.
    case $(( (rep - 1) % 3 )) in
        0) order="1 2 3";;
        1) order="2 3 1";;
        2) order="3 1 2";;
    esac
    for n in $order; do
        echo "--- rep $rep · ${n}노드 (커넥션 ${CONNS_PER_NODE}/node = $((n * CONNS_PER_NODE)) total) ---"
        setup "$n"
        verify_nodes "$n" || continue
        case $n in
            1) concs="$CONCS_1N";;
            2) concs="$CONCS_2N";;
            3) concs="$CONCS_3N";;
        esac
        # concurrency 순서도 rep 마다 뒤집는다
        if [ $((rep % 2)) -eq 0 ]; then
            concs=$(echo "$concs" | tr ' ' '\n' | tac | tr '\n' ' ')
        fi
        for c in $concs; do
            bench_once "$n" "$c" "$rep"
        done
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

ssh -o BatchMode=yes npuforge-server "cd $REMOTE && tar cf - *.json" 2>/dev/null \
    | tar xf - -C "$OUT/raw" 2>/dev/null
printf 'freeze_commit=%s\nconns_per_node=%s\nduration=%s\nreps=%s\n' \
    "$FREEZE" "$CONNS_PER_NODE" "$DUR" "$REPS" > "$OUT/meta.txt"

echo
PYTHONIOENCODING=utf-8 python scripts/analyze-scaleout.py "$CSV" 2>&1 || cat "$CSV"
echo "결과: $OUT/"
