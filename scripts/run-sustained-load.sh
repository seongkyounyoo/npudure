#!/usr/bin/env bash
# S0 — 지속 부하에서 운영점이 유지되는가.
#
# 왜 필요한가
#   지금까지의 모든 측정이 **60초 이하**다. 즉 전부 throttling 발현 전
#   구간이다. worklog 는 CPU 가 300초에서 −27% 강등된다고 기록하고 있다.
#   그것이 재현되면 지금의 운영점(커넥션 2/node @ c12, 3N 390 inf/s)은
#   **short-run operating point** 였다는 뜻이 된다.
#
#     short-run operating point   60초 이하 benchmark 기준
#     sustained operating point   thermal steady-state 기준   ← 이걸 잰다
#
#   Edge 장비에서는 순간 최고 처리량보다 **지속 가능한 처리량**이 현실적인
#   지표다. 둘이 같으면 현재 결과의 적용 범위가 강해지고, 다르면 그것 자체가
#   더 중요한 결과다.
#
# 왜 60초 run 을 이어 붙이는가
#   bench 는 인터벌 출력이 없어 `--duration 1800` 을 주면 30분치 요약 하나만
#   나온다. **언제 꺾이는지**를 볼 수 없다. 60초 run 을 연속으로 돌리면
#   처리량 시계열이 생긴다.
#
#   run 사이 공백은 2~4초다. 열 시상수는 수십 초~분 단위이므로 이 공백이
#   열 상태를 되돌리지 못한다. 노드·스케줄러는 **재기동하지 않는다** —
#   재기동하면 그게 진짜 공백이 된다.
#
# 조건 A/B
#   이 스크립트는 **현재 냉각 상태 그대로** 잰다. 팬리스(S0-A)는 팬을 물리적
#   으로 빼야 하므로 사람이 해야 한다. 라벨로 구분해 두 번 돌린다.
#
# 사용법
#   bash scripts/run-sustained-load.sh [분] [라벨]     기본 30분, 라벨 fan
#     bash scripts/run-sustained-load.sh 30 fan        조건 B (능동 냉각)
#     bash scripts/run-sustained-load.sh 30 fanless    조건 A (팬 제거 후)
set -u

MINUTES=${1:-30}
LABEL=${2:-fan}
DUR_TOTAL=$((MINUTES * 60))
RUN_DUR=60
RUNS=$((DUR_TOTAL / RUN_DUR))

DATE=$(date +%Y%m%d)
OUT="results/sustained-${DATE}-${LABEL}"
REMOTE=/tmp/s0

BENCH=/root/npuforge/target/release/npuforge-bench
SCHED_BIN=/root/npuforge/target/release/npuforge-scheduler
SCHED_CFG=/root/scheduler-s0.toml
SCHED_URL=http://127.0.0.1:50051

# S3.8 이 확정한 운영점: 노드당 커넥션 2개, 노드당 c12 → 3N 은 c36.
NODES=3
CONC=36
CONNS_PER_NODE=2

NODE_BIN=/home/pi/npuforge/npuforge-node.s36
NODE_CFG=/home/pi/npuforge/node-s0.toml
LOGGER=/home/pi/thermal-logger.sh
HOSTS="npuforge-k npuforge-q npuforge-j"

FREEZE=$(git rev-parse --short HEAD 2>/dev/null || echo unknown)
mkdir -p "$OUT/raw"
CSV="$OUT/raw/throughput.csv"

sched_stop() { ssh -o BatchMode=yes npuforge-server 'pkill -9 npuforge-schedu 2>/dev/null; exit 0' 2>/dev/null; }
node_stop()  { ssh -o BatchMode=yes "$1" 'pkill -9 npuforge-node 2>/dev/null; exit 0' 2>/dev/null; }

setup() {
    sched_stop
    for h in $HOSTS; do node_stop "$h"; done
    sleep 4
    {
        echo
        echo "[transport]"
        echo "node_connections = $CONNS_PER_NODE"
    } | ssh -o BatchMode=yes npuforge-server \
        "cp /root/scheduler-base.toml $SCHED_CFG && cat >> $SCHED_CFG" 2>/dev/null
    for h in $HOSTS; do
        ssh -o BatchMode=yes "$h" "cp /home/pi/npuforge/node.toml $NODE_CFG" 2>/dev/null
    done
    ssh -o BatchMode=yes npuforge-server \
        "setsid nohup $SCHED_BIN --config $SCHED_CFG >/tmp/sched-s0.log 2>&1 </dev/null & disown; exit 0" 2>/dev/null
    sleep 3
    for h in $HOSTS; do
        ssh -o BatchMode=yes "$h" \
            "pgrep npuforge-node >/dev/null 2>&1 || { setsid nohup $NODE_BIN --config $NODE_CFG \
             >>/home/pi/npuforge/node-s0.log 2>&1 </dev/null & disown; }; exit 0" 2>/dev/null
    done
    sleep 12
}

verify_nodes() {
    ssh -o BatchMode=yes npuforge-server \
        "mkdir -p $REMOTE/probe; rm -f $REMOTE/probe/*.json; \
         $BENCH --scheduler $SCHED_URL --model yolov8n --concurrency 12 \
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
    [ "$cnt" = "$NODES" ] || { echo "  !! 노드 수 불일치 expected=$NODES observed=$cnt ($seen)"; return 1; }
    echo "  노드 검증 OK — $cnt개 ($seen)"
}

echo "=== S0 지속 부하 (freeze $FREEZE, ${MINUTES}분, 라벨 $LABEL) ==="
echo "    운영점: ${NODES}노드, 커넥션 ${CONNS_PER_NODE}/node, c${CONC}"
echo "    60초 run × ${RUNS}회 연속 (재기동 없음)"

scp -q -o BatchMode=yes scripts/thermal-logger.sh npuforge-k:$LOGGER
scp -q -o BatchMode=yes scripts/thermal-logger.sh npuforge-q:$LOGGER
scp -q -o BatchMode=yes scripts/thermal-logger.sh npuforge-j:$LOGGER
for h in $HOSTS; do ssh -o BatchMode=yes "$h" "chmod +x $LOGGER" 2>/dev/null; done
ssh -o BatchMode=yes npuforge-server "mkdir -p $REMOTE" 2>/dev/null

setup
verify_nodes || exit 1

# 부하 전 유휴 온도. 시작점이 다르면 상승폭을 비교할 수 없다.
echo "--- 유휴 온도 (부하 전) ---"
for h in $HOSTS; do
    printf '  %-14s ' "$h"
    ssh -o BatchMode=yes "$h" 'awk "{printf \"soc %.1fC  \", \$1/1000}" /sys/class/thermal/thermal_zone0/temp; awk "{printf \"npu %.1fC\", \$1/1000}" /sys/class/thermal/thermal_zone4/temp' 2>/dev/null
    echo
done

# 열 로거를 먼저 띄운다. 부하 전 몇 초가 baseline 이 된다.
LOG_DUR=$((DUR_TOTAL + 120))
for h in $HOSTS; do
    ssh -o BatchMode=yes "$h" \
        "setsid nohup bash $LOGGER /tmp/thermal-${LABEL}.log $LOG_DUR >/dev/null 2>&1 </dev/null & disown; exit 0" 2>/dev/null
done
sleep 5

echo "run,epoch,elapsed_s,throughput,p50_ms,p95_ms,p99_ms,error_rate,balance_pp,observed_nodes,max_npu_c" > "$CSV"
T0=$(date +%s)

for i in $(seq 1 "$RUNS"); do
    ssh -o BatchMode=yes npuforge-server \
        "mkdir -p $REMOTE/rt; rm -f $REMOTE/rt/*.json; \
         $BENCH --scheduler $SCHED_URL --model yolov8n --concurrency $CONC \
         --duration $RUN_DUR --policy round-robin --out $REMOTE/rt >/dev/null 2>&1; \
         f=\$(ls $REMOTE/rt/*.json 2>/dev/null | head -1); \
         [ -n \"\$f\" ] && mv \"\$f\" $REMOTE/s0_${LABEL}_$(printf '%02d' "$i").json" 2>/dev/null

    m=$(ssh -o BatchMode=yes npuforge-server \
        "python3 -c \"
import json
d=json.load(open('$REMOTE/s0_${LABEL}_$(printf '%02d' "$i").json'))
s=d['summary']; L=s['latency']
pn=[p for p in s['per_node'] if p['count']>0]
sh=sorted(p['share'] for p in pn)
print('%s,%.1f,%.2f,%.2f,%.2f,%.4f,%.2f,%d' % (
  d['started_at'].split(':')[1], s['throughput'], L['p50']/1000, L['p95']/1000,
  L['p99']/1000, s['error_rate'], (sh[-1]-sh[0])*100 if sh else 0, len(pn)))
\"" 2>/dev/null | tr -d '\r')

    now=$(date +%s)
    # 세 보드 NPU 온도 중 최대. 팬리스에서는 임계치(degraded 80 / disable 90)
    # 접근이 결과를 바꾼다 — 노드가 제외되면 처리량 하락이 throttling 이
    # 아니라 노드 수 감소 때문이다. 둘을 구분하려면 온도와 노드 수를
    # run 단위로 남겨야 한다.
    maxnpu=$(for h in $HOSTS; do
        ssh -o BatchMode=yes "$h" "awk '{print \$1/1000}' /sys/class/thermal/thermal_zone4/temp" 2>/dev/null
    done | sort -n | tail -1)
    echo "$i,$m" | awk -v r="$i" -v t0="$T0" -v now="$now" -v mn="${maxnpu:-0}" -F, \
        '{printf "%s,%s,%d,%s,%s,%s,%s,%s,%s,%s,%.1f\n", r, $2, now-t0, $3, $4, $5, $6, $7, $8, $9, mn}' >> "$CSV"

    nodes_now=$(echo "$m" | cut -d, -f8)
    if [ "${nodes_now:-3}" != "3" ]; then
        printf '  !! run %d: 응답 노드가 %s개다 (온도 임계로 제외 의심, npu최대 %s°C)\n' \
            "$i" "$nodes_now" "${maxnpu:-?}"
    fi

    if [ $((i % 5)) -eq 0 ] || [ "$i" = 1 ]; then
        temps=$(for h in $HOSTS; do
            ssh -o BatchMode=yes "$h" 'awk "{printf \"%.0f/\", $1/1000}" /sys/class/thermal/thermal_zone0/temp; awk "{printf \"%.0f \", $1/1000}" /sys/class/thermal/thermal_zone4/temp' 2>/dev/null
        done)
        printf '  run %2d/%d  t+%4ds  %s inf/s  노드%s  soc/npu: %s\n' \
            "$i" "$RUNS" "$((now - T0))" "$(echo "$m" | cut -d, -f2)" "${nodes_now:-?}" "$temps"
    else
        printf '  run %2d/%d  t+%4ds  %s inf/s  노드%s  npu최대 %s°C\n' \
            "$i" "$RUNS" "$((now - T0))" "$(echo "$m" | cut -d, -f2)" "${nodes_now:-?}" "${maxnpu:-?}"
    fi
done

echo "--- 부하 종료. 열 로거 수거 ---"
sleep 3
mkdir -p "$OUT/raw/thermal"
for h in $HOSTS; do
    scp -q -o BatchMode=yes "$h:/tmp/thermal-${LABEL}.log" \
        "$OUT/raw/thermal/$(echo "$h" | sed 's/npuforge-//').log" 2>/dev/null
done
ssh -o BatchMode=yes npuforge-server "cd $REMOTE && tar cf - s0_${LABEL}_*.json" 2>/dev/null \
    | tar xf - -C "$OUT/raw" 2>/dev/null

echo "--- 원상복구 (3노드, 기본 설정) ---"
sched_stop
for h in $HOSTS; do node_stop "$h"; done
sleep 4
ssh -o BatchMode=yes npuforge-server \
    "setsid nohup $SCHED_BIN --config /root/scheduler.toml >/tmp/sched.log 2>&1 </dev/null & disown; exit 0" 2>/dev/null
sleep 3
. "$(dirname "${BASH_SOURCE[0]}")/lib/remote.sh"
npuforge_restore_cluster

printf 'freeze_commit=%s\nlabel=%s\nminutes=%s\nnodes=%s\nconc=%s\nconns_per_node=%s\nrun_dur=%s\n' \
    "$FREEZE" "$LABEL" "$MINUTES" "$NODES" "$CONC" "$CONNS_PER_NODE" "$RUN_DUR" > "$OUT/meta.txt"

echo
PYTHONIOENCODING=utf-8 python scripts/analyze-sustained.py "$OUT" 2>&1 || cat "$CSV"
echo "결과: $OUT/"
