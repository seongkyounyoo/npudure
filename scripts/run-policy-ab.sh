#!/usr/bin/env bash
# S0-C — 팬리스 열 불균질 상태에서 스케줄링 정책 A/B.
#
# 무엇을 닫는 실험인가
#   S0-A 는 여기까지 관측했다.
#     - 열 편차가 있다 (CPU 816 / 1200 / 1416 MHz)
#     - king 의 service capacity 가 실제로 낮다 (p50 2.4배)
#     - **RR 은 느려진 king 에도 33.3% 를 계속 보낸다**
#
#   아직 아닌 것: "팬리스 손실 = 열 편차 × 부하 무인지 정책".
#   **정책을 바꿨을 때 손실이 실제로 회수돼야** 인과가 닫힌다.
#
#   반대 결과도 중요하다 — 정책을 켜도 분배가 1/3 로 유지되거나 성능이
#   그대로면, 현재 정책의 상태 신호가 thermal-induced capacity degradation
#   을 감지하지 못한다는 뜻이다.
#
# 왜 지금인가 (팬을 켜기 전에)
#   능동 냉각에서는 세 노드가 거의 동질적이라 **정책 차이가 사라질 가능성이
#   크다.** 지금 팬리스 상태가 adaptive scheduling 의 시험장이다.
#
# 설계상 지키는 것
#   1. **먼저 가열해 thermal steady-state 에 든 뒤 비교한다.** 정책마다
#      시작 온도가 다르면 정책 효과와 thermal drift 가 섞인다.
#   2. 정책 순서를 라운드마다 회전한다.
#   3. 처리량·p95·p99 외에 **노드별 분배·노드별 지연·노드별 CPU idle** 을
#      남긴다. "queen/jack 은 놀고 있다" 를 지연만으로 단정할 수 없다.
#
# 사용법
#   bash scripts/run-policy-ab.sh [라운드수] [예열분]     기본 5 라운드, 15분 예열
set -u

ROUNDS=${1:-5}
PREHEAT_MIN=${2:-15}
# 냉각 조건 라벨. 결과 메타·열로그 파일명에 기록된다 (fanless | fan).
COOLING=${NPUFORGE_COOLING:-fanless}
# 정책 측정 직전 RR 재예열 run 수 (각 60초). 시작 열 조건을 맞춘다.
REHEAT_RUNS=${3:-3}

DATE=$(date +%Y%m%d)
OUT="results/policy-ab-${DATE}${NPUFORGE_SUFFIX:-}"
REMOTE=/tmp/s0c

BENCH=/root/npuforge/target/release/npuforge-bench
SCHED_BIN=/root/npuforge/target/release/npuforge-scheduler
SCHED_CFG=/root/scheduler-s0c.toml
SCHED_URL=http://127.0.0.1:50051
DUR=60
NODES=3
CONC=36
CONNS_PER_NODE=2

NODE_BIN=/home/pi/npuforge/npuforge-node.s36
NODE_CFG=/home/pi/npuforge/node-s0c.toml
LOGGER=/home/pi/thermal-logger.sh
HOSTS="npuforge-k npuforge-q npuforge-j"
POLICIES="round-robin least-queue ect"

# ── 클러스터 점유 확인 ────────────────────────────────────────────────
# 다른 하네스가 돌고 있으면 시작하지 않는다. 2026-08-21 에 두 하네스가
# 같은 클러스터를 c36 씩 때려 측정이 조용히 오염된 적이 있다.
. "$(dirname "${BASH_SOURCE[0]}")/lib/remote.sh"
npuforge_assert_cluster_free "정책 A/B" || exit 1

FREEZE=$(git rev-parse --short HEAD 2>/dev/null || echo unknown)
# 결과 디렉터리를 덮어쓰지 않는다. 2026-08-21 에 같은 날짜 경로를
# 재사용해 **S0-C 1차 데이터(15 run)를 덮어썼다.** git 이 추적 중이라
# 복원됐지만, 추적 전이었다면 사라졌다.
if [ -d "$OUT" ] && [ -n "$(ls -A "$OUT" 2>/dev/null)" ]; then
    echo "!! 결과 디렉터리가 이미 있고 비어 있지 않다: $OUT" >&2
    echo "   덮어쓰면 이전 측정이 사라진다. NPUFORGE_SUFFIX 로 구분하라." >&2
    echo "     NPUFORGE_SUFFIX=-2 bash \$0 ..." >&2
    exit 1
fi
mkdir -p "$OUT/raw"
CSV="$OUT/raw/results.csv"

sched_stop() { ssh -o BatchMode=yes npuforge-server 'pkill -9 npuforge-schedu 2>/dev/null; exit 0' 2>/dev/null; }
node_stop()  { ssh -o BatchMode=yes "$1" 'pkill -9 npuforge-node 2>/dev/null; exit 0' 2>/dev/null; }

# 노드는 한 번만 띄우고 끝까지 유지한다. 정책은 스케줄러 설정이므로
# 스케줄러만 재기동하면 된다 — 노드를 재기동하면 NPU 컨텍스트가 풀리고
# 열 상태도 흔들린다.
start_nodes() {
    for h in $HOSTS; do
        ssh -o BatchMode=yes "$h" "cp /home/pi/npuforge/node.toml $NODE_CFG" 2>/dev/null
        ssh -o BatchMode=yes "$h" \
            "pgrep npuforge-node >/dev/null 2>&1 || { setsid nohup $NODE_BIN --config $NODE_CFG \
             >>/home/pi/npuforge/node-s0c.log 2>&1 </dev/null & disown; }; exit 0" 2>/dev/null
    done
}

# 정책만 바꿔 스케줄러를 재기동한다. 노드는 건드리지 않는다.
set_policy() {
    local pol=$1
    sched_stop
    sleep 2
    ssh -o BatchMode=yes npuforge-server \
        "sed 's/^policy = .*/policy = \"$pol\"/' /root/scheduler-base.toml > $SCHED_CFG; \
         printf '\n[transport]\nnode_connections = $CONNS_PER_NODE\n' >> $SCHED_CFG" 2>/dev/null
    ssh -o BatchMode=yes npuforge-server \
        "setsid nohup $SCHED_BIN --config $SCHED_CFG >/tmp/sched-s0c.log 2>&1 </dev/null & disown; exit 0" 2>/dev/null
    sleep 12   # 노드 재등록 대기
    # 정말 그 정책으로 떴는지 확인한다. sed 가 빗나가면 조용히 같은 조건
    # 세 번이 된다.
    local got
    got=$(ssh -o BatchMode=yes npuforge-server "grep -oP '^policy = \"\K[^\"]+' $SCHED_CFG" 2>/dev/null | tr -d '\r')
    [ "$got" = "$pol" ] || { echo "  !! 정책 주입 실패 (원함 $pol, 실제 '$got')"; return 1; }
    return 0
}

# probe 는 노드 수 검증이 목적이지만 **본측정 concurrency 로 돈다.**
# c12 로 돌리면 측정 직전 10초가 1/3 부하 = 냉각 구간이 된다. 스케줄러
# 재기동 무부하 14초에 이 10초가 붙어, 24초를 식은 채로 측정에 들어갔다.
# 강한 이질(2.4배) 재현을 막던 것이 이것이다 — 예열분만 늘려도 마지막
# 24초는 그대로다. 검증 목적은 부하와 무관하므로 c36 으로 가열한다.
verify_nodes() {
    ssh -o BatchMode=yes npuforge-server \
        "mkdir -p $REMOTE/probe; rm -f $REMOTE/probe/*.json; \
         $BENCH --scheduler $SCHED_URL --model yolov8n --concurrency $CONC \
         --duration 10 --policy probe --out $REMOTE/probe >/dev/null 2>&1" 2>/dev/null
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
    return 0
}

# 노드별 CPU idle 을 /proc/stat 델타로 잰다. "놀고 있다" 를 지연만으로
# 단정할 수 없으므로 utilization 을 함께 남긴다.
cpu_snap() {
    local when=$1 tag=$2
    for h in $HOSTS; do
        ssh -o BatchMode=yes "$h" 'grep "^cpu " /proc/stat' 2>/dev/null \
            > "$OUT/raw/cpu_${tag}_$(echo "$h" | sed 's/npuforge-//').${when}"
    done
}

echo "=== 정책 A/B (freeze $FREEZE, 냉각=$COOLING, ${ROUNDS}라운드, 예열 ${PREHEAT_MIN}분, 재예열 ${REHEAT_RUNS}run) ==="
for h in $HOSTS; do
    scp -q -o BatchMode=yes scripts/thermal-logger.sh "$h:$LOGGER"
    ssh -o BatchMode=yes "$h" "chmod +x $LOGGER" 2>/dev/null
done
ssh -o BatchMode=yes npuforge-server "mkdir -p $REMOTE" 2>/dev/null

sched_stop
for h in $HOSTS; do node_stop "$h"; done
sleep 4
start_nodes
set_policy round-robin || exit 1
verify_nodes || exit 1
echo "  노드 3개 확인"

# 열 로거 (예열 + 본측정 전체)
# 정책 하나당 벽시계: 재예열 REHEAT_RUNS*60 + set_policy 2회(~28초)
# + probe 10초 + 본측정 60초 + 여유. REHEAT_RUNS 를 빼먹으면 로거가
# 본측정 도중에 죽어 열 로그가 잘린다.
LOG_DUR=$(( PREHEAT_MIN * 60 + ROUNDS * 3 * (REHEAT_RUNS * 60 + 150) + 600 ))
for h in $HOSTS; do
    ssh -o BatchMode=yes "$h" \
        "setsid nohup bash $LOGGER /tmp/thermal-policy-$COOLING.log $LOG_DUR >/dev/null 2>&1 </dev/null & disown; exit 0" 2>/dev/null
done

# ── 예열 ──────────────────────────────────────────────────────────────
# 정책 비교 전에 thermal steady-state 에 넣는다. S0-A 는 약 10~25분에
# 평탄역에 들었다.
echo "--- 예열 ${PREHEAT_MIN}분 (round-robin, 결과는 버린다) ---"
PRE_RUNS=$((PREHEAT_MIN))
for i in $(seq 1 "$PRE_RUNS"); do
    ssh -o BatchMode=yes npuforge-server \
        "mkdir -p $REMOTE/pre; rm -f $REMOTE/pre/*.json; \
         $BENCH --scheduler $SCHED_URL --model yolov8n --concurrency $CONC \
         --duration $DUR --policy preheat --out $REMOTE/pre >/dev/null 2>&1" 2>/dev/null
    if [ $((i % 5)) -eq 0 ] || [ "$i" = "$PRE_RUNS" ]; then
        t=$(for h in $HOSTS; do
            ssh -o BatchMode=yes "$h" "awk '{printf \"%.0f \", \$1/1000}' /sys/class/thermal/thermal_zone0/temp" 2>/dev/null
        done)
        printf '  예열 %2d/%d  soc: %s\n' "$i" "$PRE_RUNS" "$t"
    fi
done
echo "  예열 완료 — thermal steady-state 가정"

# ── 본측정 ────────────────────────────────────────────────────────────
echo "policy,round,throughput,p50_ms,p95_ms,p99_ms,error_rate,king_share,jack_share,queen_share,king_p50,jack_p50,queen_p50,max_soc_c,start_soc_c" > "$CSV"

for r in $(seq 1 "$ROUNDS"); do
    # 라운드마다 정책 순서를 회전한다. 시간·열 드리프트가 한 정책에
    # 몰리면 정책 비교가 아니라 먼저/나중 비교가 된다.
    case $(( (r - 1) % 3 )) in
        0) order="round-robin least-queue ect";;
        1) order="least-queue ect round-robin";;
        2) order="ect round-robin least-queue";;
    esac
    for pol in $order; do
        tag="${pol}_r${r}"
        # **정책마다 RR 로 재예열해 시작 thermal state 를 맞춘다.**
        #
        # 1차 S0-C 는 이걸 안 해서 실패했다 — 저부하 정책이 도는 동안 보드가
        # 식어 뒤따르는 RR 이 시원한 상태에서 돌았다(RR 355 → 385).
        # 열 불균질 조건 자체가 사라졌다.
        #
        # 맞춰야 하는 것은 **시작** 조건이다. 정책 실행 중에 온도가 갈라지는
        # 것은 그대로 둔다 — adaptive scheduler 가 king 의 부하를 줄여
        # king 이 식는다면 그것 자체가 정책 효과의 일부다.
        if [ "$REHEAT_RUNS" -gt 0 ]; then
            set_policy round-robin >/dev/null 2>&1
            for _ in $(seq 1 "$REHEAT_RUNS"); do
                ssh -o BatchMode=yes npuforge-server                     "mkdir -p $REMOTE/pre; rm -f $REMOTE/pre/*.json;                      $BENCH --scheduler $SCHED_URL --model yolov8n --concurrency $CONC                      --duration $DUR --policy reheat --out $REMOTE/pre >/dev/null 2>&1" 2>/dev/null
            done
            preheat_soc=$(for h in $HOSTS; do
                ssh -o BatchMode=yes "$h" "awk '{print \$1/1000}' /sys/class/thermal/thermal_zone0/temp" 2>/dev/null
            done | sort -n | tail -1)
        fi
        set_policy "$pol" || continue
        verify_nodes || { echo "  !! $pol r$r 노드 검증 실패, 건너뜀"; continue; }
        cpu_snap before "$tag"
        ssh -o BatchMode=yes npuforge-server \
            "mkdir -p $REMOTE/rt; rm -f $REMOTE/rt/*.json; \
             $BENCH --scheduler $SCHED_URL --model yolov8n --concurrency $CONC \
             --duration $DUR --policy $pol --out $REMOTE/rt >/dev/null 2>&1; \
             f=\$(ls $REMOTE/rt/*.json 2>/dev/null | head -1); \
             [ -n \"\$f\" ] && mv \"\$f\" $REMOTE/${tag}.json" 2>/dev/null
        cpu_snap after "$tag"

        m=$(ssh -o BatchMode=yes npuforge-server \
            "python3 -c \"
import json
d=json.load(open('$REMOTE/${tag}.json'))
s=d['summary']; L=s['latency']
pn={p['node_id']: p for p in s['per_node'] if p['count']>0}
g=lambda n,k: (pn[n]['share']*100 if k=='share' else pn[n]['latency']['p50']/1000) if n in pn else 0.0
print('%.1f,%.2f,%.2f,%.2f,%.4f,%.1f,%.1f,%.1f,%.1f,%.1f,%.1f' % (
  s['throughput'], L['p50']/1000, L['p95']/1000, L['p99']/1000, s['error_rate'],
  g('king','share'), g('jack','share'), g('queen','share'),
  g('king','p50'), g('jack','p50'), g('queen','p50')))
\"" 2>/dev/null | tr -d '\r')

        maxsoc=$(for h in $HOSTS; do
            ssh -o BatchMode=yes "$h" "awk '{print \$1/1000}' /sys/class/thermal/thermal_zone0/temp" 2>/dev/null
        done | sort -n | tail -1)

        echo "$pol,$r,$m,$maxsoc,${preheat_soc:-}" >> "$CSV"
        printf '  r%s  %-12s -> %s  soc최대 %s°C\n' "$r" "$pol" "${m:-실패}" "${maxsoc:-?}"
        sleep 5
    done
done

echo "--- 열 로거 수거 ---"
mkdir -p "$OUT/raw/thermal"
for h in $HOSTS; do
    scp -q -o BatchMode=yes "$h:/tmp/thermal-policy-$COOLING.log" \
        "$OUT/raw/thermal/$(echo "$h" | sed 's/npuforge-//').log" 2>/dev/null
done
ssh -o BatchMode=yes npuforge-server "cd $REMOTE && tar cf - *.json" 2>/dev/null \
    | tar xf - -C "$OUT/raw" 2>/dev/null

echo "--- 원상복구 (기본 설정, 팬리스 유지) ---"
sched_stop
sleep 2
ssh -o BatchMode=yes npuforge-server \
    "setsid nohup $SCHED_BIN --config /root/scheduler.toml >/tmp/sched.log 2>&1 </dev/null & disown; exit 0" 2>/dev/null
sleep 3
. "$(dirname "${BASH_SOURCE[0]}")/lib/remote.sh"
npuforge_restore_cluster

printf 'freeze_commit=%s\ncooling=%s\nrounds=%s\npreheat_min=%s\nnodes=%s\nconc=%s\nconns_per_node=%s\n' \
    "$FREEZE" "$COOLING" "$ROUNDS" "$PREHEAT_MIN" "$NODES" "$CONC" "$CONNS_PER_NODE" > "$OUT/meta.txt"

echo
PYTHONIOENCODING=utf-8 python scripts/analyze-policy-ab.py "$OUT" 2>&1 || cat "$CSV"
echo "결과: $OUT/"
