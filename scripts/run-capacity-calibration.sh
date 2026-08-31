#!/usr/bin/env bash
# S0-D 교정 — CPU 주파수 캡과 노드 지연 편차의 대응표를 만든다.
#
# 왜 필요한가
#   S0-C 4차에서 팬리스 연속 가열로 강한 이질(2.4배)을 재현하려다 실패
#   했다. 원인은 §18.3 — **열 조건은 이질의 필요조건이지 충분조건이
#   아니다.** 세 실험 모두 soc 86.8°C 로 같았고, 갈린 것은 CPU 강등의
#   보드간 편차뿐이었다.
#
#     클럭 편차  1.14x -> 1.50x -> 1.79x
#     지연 편차  1.10x -> 1.33x -> 2.40x     (4차 / S0-C 2차 / S0-A)
#
#   열 제어는 온도를 목표로 하지 편차를 목표로 하지 않으므로, 편차를
#   냉각으로 불러낼 수 없다. 대신 **열 제어가 쓰는 손잡이를 직접 잡는다** —
#   강등은 scaling_max_freq 를 내리는 방식으로 구현돼 있다.
#
# 무엇을 만드는가
#   king 의 CPU 캡을 사다리로 훑어 **캡 -> 노드 지연 편차** 대응표를
#   얻는다. 그래야 다음 정책 A/B 에서 편차를 1.3x / 1.8x / 2.4x 로
#   **원하는 값에 놓고** 비교할 수 있다.
#
#   S0-A 의 클럭을 그대로 복제하지 않는 이유: 열 로거가 cpu4 만 기록해
#   리틀 코어 값을 모른다. 클럭을 맞추는 대신 **관측량(지연 편차)에
#   직접 맞추는** 편이 정직하다.
#
# 조건
#   **팬 ON.** 열을 변수에서 뺀다. 보드가 차가우면 열 제어가 캡보다
#   낮게 내릴 이유가 없어 캡이 그대로 유지된다.
#   정책은 **round-robin** — 적응하지 않으므로 균등 부하 아래 raw
#   capacity 편차가 그대로 드러난다. S0-A 의 2.4배도 같은 정의다.
#
# 사용법
#   bash scripts/run-capacity-calibration.sh [반복수] [캡사다리]
#     기본: 2회, "2208 1608 1200 1008 816 600" (MHz)
set -u

REPS=${1:-2}
LADDER=${2:-"2208 1608 1200 1008 816 600"}

DATE=$(date +%Y%m%d)
OUT="results/capacity-calib-${DATE}${NPUFORGE_SUFFIX:-}"
REMOTE=/tmp/s0d

BENCH=/root/npuforge/target/release/npuforge-bench
SCHED_BIN=/root/npuforge/target/release/npuforge-scheduler
SCHED_CFG=/root/scheduler-s0d.toml
SCHED_URL=http://127.0.0.1:50051
DUR=60
NODES=3
CONC=36
CONNS_PER_NODE=2

# 캡을 거는 대상. S0-A 에서 실제로 느려졌던 보드와 같게 king 을 쓴다.
TARGET=npuforge-k
TARGET_ID=king
HOSTS="npuforge-k npuforge-q npuforge-j"
P0_MAX=2016000   # policy0 (cpu0-3) 상한
P4_MAX=2208000   # policy4 (cpu4-7) 상한

SUDO_PASS="${NPUFORGE_SUDO_PASS:-}"
if [ -z "$SUDO_PASS" ] && [ -r "$HOME/.npuforge/sudo-pass" ]; then
    SUDO_PASS="$(head -1 "$HOME/.npuforge/sudo-pass")"
fi
[ -n "$SUDO_PASS" ] || { echo "sudo 비밀번호가 없다 (~/.npuforge/sudo-pass 또는 NPUFORGE_SUDO_PASS)"; exit 2; }

# ── 클러스터 점유 확인 ────────────────────────────────────────────────
# 다른 하네스가 돌고 있으면 시작하지 않는다. 2026-08-21 에 두 하네스가
# 같은 클러스터를 c36 씩 때려 측정이 조용히 오염된 적이 있다.
. "$(dirname "${BASH_SOURCE[0]}")/lib/remote.sh"
npuforge_assert_cluster_free "capacity 교정" || exit 1

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

# ── 캡 설정/복구 ──────────────────────────────────────────────────────
# 주파수 표는 두 그룹이 공유한다(408000..2016000, policy4 만 2208000).
# 캡 값은 각 그룹 상한으로 clamp 한다.
set_cap() {
    local mhz=$1 khz p0 p4
    khz=$((mhz * 1000))
    p0=$khz; [ "$p0" -gt "$P0_MAX" ] && p0=$P0_MAX
    p4=$khz; [ "$p4" -gt "$P4_MAX" ] && p4=$P4_MAX
    ssh -o BatchMode=yes "$TARGET" "printf '%s\n' '$SUDO_PASS' | sudo -S -p '' sh -c \
        'echo $p0 > /sys/devices/system/cpu/cpufreq/policy0/scaling_max_freq; \
         echo $p4 > /sys/devices/system/cpu/cpufreq/policy4/scaling_max_freq' 2>/dev/null; exit 0" 2>/dev/null
    sleep 1
    # 되읽어 확인한다. 조용히 안 걸리면 같은 조건 여러 번이 된다.
    local got
    got=$(ssh -o BatchMode=yes "$TARGET" \
        'cat /sys/devices/system/cpu/cpufreq/policy4/scaling_max_freq' 2>/dev/null | tr -d '\r')
    [ "$got" = "$p4" ] || { echo "  !! 캡 주입 실패 (원함 $p4, 실제 '$got')"; return 1; }
    return 0
}

restore_cap() {
    ssh -o BatchMode=yes "$TARGET" "printf '%s\n' '$SUDO_PASS' | sudo -S -p '' sh -c \
        'echo $P0_MAX > /sys/devices/system/cpu/cpufreq/policy0/scaling_max_freq; \
         echo $P4_MAX > /sys/devices/system/cpu/cpufreq/policy4/scaling_max_freq' 2>/dev/null; exit 0" 2>/dev/null
}
# 중단되어도 king 을 강등된 채로 남기지 않는다.
trap 'echo; echo "--- 캡 복구 (trap) ---"; restore_cap; exit 130' INT TERM
trap 'restore_cap' EXIT

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

echo "=== capacity 교정 (freeze $FREEZE, 팬 ON, 대상 $TARGET_ID, 반복 ${REPS}, 사다리 $LADDER) ==="
ssh -o BatchMode=yes npuforge-server "mkdir -p $REMOTE" 2>/dev/null

# 스케줄러는 round-robin 으로 한 번만 띄우고 유지한다. 캡만 바뀌므로
# 재기동할 이유가 없다 — 재기동은 무부하 구간을 만든다.
ssh -o BatchMode=yes npuforge-server \
    "pkill -9 npuforge-schedu 2>/dev/null; exit 0" 2>/dev/null
sleep 2
ssh -o BatchMode=yes npuforge-server \
    "sed 's/^policy = .*/policy = \"round-robin\"/' /root/scheduler-base.toml > $SCHED_CFG; \
     printf '\n[transport]\nnode_connections = $CONNS_PER_NODE\n' >> $SCHED_CFG; \
     setsid nohup $SCHED_BIN --config $SCHED_CFG >/tmp/sched-s0d.log 2>&1 </dev/null & disown; exit 0" 2>/dev/null
sleep 12
restore_cap
verify_nodes || { echo "노드 검증 실패"; exit 1; }
echo "  노드 3개 확인"

echo "cap_mhz,rep,throughput,p50_ms,p95_ms,p99_ms,error_rate,king_share,jack_share,queen_share,king_p50,jack_p50,queen_p50,spread,king_cur_mhz,king_soc_c" > "$CSV"

for cap in $LADDER; do
    set_cap "$cap" || continue
    for rep in $(seq 1 "$REPS"); do
        tag="cap${cap}_r${rep}"
        ssh -o BatchMode=yes npuforge-server \
            "mkdir -p $REMOTE/rt; rm -f $REMOTE/rt/*.json; \
             $BENCH --scheduler $SCHED_URL --model yolov8n --concurrency $CONC \
             --duration $DUR --policy round-robin --out $REMOTE/rt >/dev/null 2>&1; \
             f=\$(ls $REMOTE/rt/*.json 2>/dev/null | head -1); \
             [ -n \"\$f\" ] && mv \"\$f\" $REMOTE/${tag}.json" 2>/dev/null

        # 캡이 부하 중에도 유지되는지 확인한다(팬 ON 이면 유지돼야 한다).
        kcur=$(ssh -o BatchMode=yes "$TARGET" \
            'awk "{printf \"%d\", \$1/1000}" /sys/devices/system/cpu/cpufreq/policy4/scaling_cur_freq' 2>/dev/null | tr -d '\r')
        ksoc=$(ssh -o BatchMode=yes "$TARGET" \
            'awk "{printf \"%.1f\", \$1/1000}" /sys/class/thermal/thermal_zone0/temp' 2>/dev/null | tr -d '\r')

        m=$(ssh -o BatchMode=yes npuforge-server \
            "python3 -c \"
import json
d=json.load(open('$REMOTE/${tag}.json'))
s=d['summary']; L=s['latency']
pn={p['node_id']: p for p in s['per_node'] if p['count']>0}
g=lambda n,k: (pn[n]['share']*100 if k=='share' else pn[n]['latency']['p50']/1000) if n in pn else 0.0
ps=[g(n,'p50') for n in ('king','jack','queen')]
v=[x for x in ps if x>0]
sp=(max(v)/min(v)) if v else 0.0
print('%.1f,%.2f,%.2f,%.2f,%.4f,%.1f,%.1f,%.1f,%.1f,%.1f,%.1f,%.2f' % (
  s['throughput'], L['p50']/1000, L['p95']/1000, L['p99']/1000, s['error_rate'],
  g('king','share'), g('jack','share'), g('queen','share'), ps[0], ps[1], ps[2], sp))
\"" 2>/dev/null | tr -d '\r')

        echo "$cap,$rep,$m,$kcur,$ksoc" >> "$CSV"
        printf '  cap %4s MHz r%s -> %s  (king 실측 %s MHz, soc %s°C)\n' \
            "$cap" "$rep" "${m:-실패}" "${kcur:-?}" "${ksoc:-?}"
    done
done

echo "--- 캡 복구 ---"
restore_cap
ssh -o BatchMode=yes npuforge-server "cd $REMOTE && tar cf - *.json" 2>/dev/null \
    | tar xf - -C "$OUT/raw" 2>/dev/null

printf 'freeze_commit=%s\ncooling=fan\ntarget=%s\nladder=%s\nreps=%s\nnodes=%s\nconc=%s\nconns_per_node=%s\npolicy=round-robin\n' \
    "$FREEZE" "$TARGET_ID" "$LADDER" "$REPS" "$NODES" "$CONC" "$CONNS_PER_NODE" > "$OUT/meta.txt"

echo
echo "cap(MHz)  throughput   king_p50  jack_p50  queen_p50   편차"
awk -F, 'NR>1{k[$1]+=$11;j[$1]+=$12;q[$1]+=$13;t[$1]+=$3;s[$1]+=$14;n[$1]++}
  END{for(c in n) printf "  %4s %10.1f %10.1f %9.1f %10.1f %8.2fx\n", c, t[c]/n[c], k[c]/n[c], j[c]/n[c], q[c]/n[c], s[c]/n[c]}' "$CSV" | sort -rn
echo "결과: $OUT/"
