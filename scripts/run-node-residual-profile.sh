#!/usr/bin/env bash
# S3.9b — node-side residual cost profiling.
#
# 질문 (좁게)
#   161.5 -> 135.5 사이의 residual gap 에서 node-side serialization /
#   copy / syscall 비용이 유의미한 비중을 차지하는가?
#
#   목적은 gap 을 전부 설명하는 것이 아니다. S3.9a 에서 scale-out
#   tail/TCP 쪽 비용이 별도로 드러났으므로 node-side 프로파일이
#   26.0 inf/s 전부를 설명해야 할 이유가 없다.
#
# 판정
#   syscall/copy 가 충분히 큼  -> S4 io_uring 진입
#   작음                       -> S4 취소/보류
#   다른 항이 큼               -> 그 항만 기록. 핵심 범위 밖이면 더 안 판다
#
# S3.5 와 무엇이 다른가 — **운영점에서 잰다**
#   S3.5 는 c32 / 노드당 커넥션 1개에서 쟀다. 그것은 **과부하 구간**이고
#   baseline transport 다. 같은 함정에 이 저장소가 이미 한 번 걸렸다
#   (experiments/README §4.1, 그리고 13.2% 오인용 사건).
#
#     S3.5    c32 · conn1   116.6 inf/s   과부하 · baseline
#     S3.9b   c12 · conn2   135.5 inf/s   운영점 · optimized
#
# 무엇을 새로 재는가
#   1. utime/stime 분리. 커널 시간에 syscall 진입 · TCP 스택 ·
#      copy_to_user 가 들어가고, 유저 시간에 protobuf 직렬화 ·
#      유저공간 copy · HTTP/2 프레이밍이 들어간다.
#      **io_uring 이 줄이는 것은 전자다.**
#   2. strace -c 로 syscall 체류시간. ptrace 오버헤드 때문에 실제보다
#      **부풀려진** 값이 나온다 -> 그래서 **상한**으로 쓴다. 부풀린 값이
#      작으면 실제는 확정적으로 더 작다. 한쪽 방향으로만 유효한 검정이다.
#
# 사용법
#   bash scripts/run-node-residual-profile.sh            네 조건 모두
#   bash scripts/run-node-residual-profile.sh --only op  한 조건만
set -u

ONLY=all
[ "${1:-}" = "--only" ] && ONLY=${2:?--only 뒤에 idle|op|strace|local}
want() { [ "$ONLY" = all ] || [ "$ONLY" = "$1" ]; }

DATE=$(date +%Y%m%d)
OUT="results/node-residual-${DATE}${NPUFORGE_SUFFIX:-}"
REMOTE_OUT=/tmp/s39b

BENCH=/root/npuforge/target/release/npuforge-bench
SCHED_BIN=/root/npuforge/target/release/npuforge-scheduler
SCHED_CFG=/root/scheduler-s39b.toml
SCHED=http://127.0.0.1:50051

LOAD_DUR=80
WARM=20        # 부하 시작 후 이만큼 지나서 수집 시작
PROF_DUR=45    # 수집 창 (WARM+PROF_DUR < LOAD_DUR)
STRACE_DUR=10  # ptrace 오버헤드가 크므로 짧게
CONC=12        # 운영점 (노드당 c12)
CONNS=2        # 운영점 (노드당 커넥션 2개)

NODE_BIN=/home/pi/npuforge/npuforge-node.s36
NODE_CFG=/home/pi/npuforge/node.toml
RKNN_MODEL=yolov8n-int8.rknn
COLLECTOR=/home/pi/node-profile-collect.sh
LOCALW=/home/pi/node-profile-local.sh

# ── 하네스 불변조건 (experiments/README §4.12) ────────────────────────
. "$(dirname "${BASH_SOURCE[0]}")/lib/remote.sh"
npuforge_assert_cluster_free "S3.9b 프로파일" || exit 1
if [ -d "$OUT" ] && [ -n "$(ls -A "$OUT" 2>/dev/null)" ]; then
    echo "!! 결과 디렉터리가 이미 있고 비어 있지 않다: $OUT" >&2
    echo "   NPUFORGE_SUFFIX 로 구분하라." >&2
    exit 1
fi

FREEZE=$(git rev-parse --short HEAD 2>/dev/null || echo unknown)
mkdir -p "$OUT/raw"

on()  { ssh -o BatchMode=yes "$1" "pgrep npuforge-node >/dev/null 2>&1 || { setsid nohup $NODE_BIN --config $NODE_CFG >>/home/pi/npuforge/node.log 2>&1 </dev/null & disown; }; exit 0" 2>/dev/null; }
# -f 를 쓰면 패턴이 자기 셸에도 걸려 ssh 세션을 죽인다 (ADR-017).
off() { ssh -o BatchMode=yes "$1" 'pkill -9 npuforge-node 2>/dev/null; exit 0' 2>/dev/null; }
kpid() { ssh -o BatchMode=yes npuforge-k 'pgrep npuforge-node | head -1' 2>/dev/null | tr -d '\r'; }

# 원격 python 한 줄짜리는 따옴표가 세 겹으로 중첩돼 조용히 깨진다.
# 스크립트 파일로 보내서 실행한다 (node-profile-collect.sh 와 같은 이유).
mk_reader() {
    cat > /tmp/s39b-read.py <<'PYEOF'
import json, glob, sys
f = sorted(glob.glob(sys.argv[1]))
if not f:
    print("NONE"); raise SystemExit
d = json.load(open(f[-1]))
s = d["summary"]
if sys.argv[2] == "nodes":
    print(",".join(sorted(p["node_id"] for p in s["per_node"] if p["count"] > 0)))
else:
    print(round(s["throughput"], 1))
PYEOF
    scp -q -o BatchMode=yes /tmp/s39b-read.py npuforge-server:/tmp/s39b-read.py 2>/dev/null
}
read_bench() {  # <glob> <nodes|throughput>
    ssh -o BatchMode=yes npuforge-server "python3 /tmp/s39b-read.py '$1' $2" 2>/dev/null | tr -d '\r'
}

echo "=== S3.9b node-side residual profiling (freeze $FREEZE, 운영점 c$CONC conn$CONNS) ==="

scp -q -o BatchMode=yes scripts/node-profile-collect.sh npuforge-k:$COLLECTOR || { echo "수집기 전송 실패"; exit 1; }
scp -q -o BatchMode=yes scripts/node-profile-local.sh    npuforge-k:$LOCALW    || { echo "래퍼 전송 실패"; exit 1; }
ssh -o BatchMode=yes npuforge-k "chmod +x $COLLECTOR $LOCALW; mkdir -p $REMOTE_OUT; rm -rf $REMOTE_OUT/$ONLY; exit 0" 2>/dev/null
mk_reader

# ── 스케줄러: 운영점 transport 설정 ───────────────────────────────────
ssh -o BatchMode=yes npuforge-server "pkill -9 npuforge-schedu 2>/dev/null; exit 0" 2>/dev/null
sleep 2
ssh -o BatchMode=yes npuforge-server \
  "sed 's/^policy = .*/policy = \"round-robin\"/' /root/scheduler-base.toml > $SCHED_CFG; \
   printf '\n[transport]\nnode_connections = $CONNS\n' >> $SCHED_CFG; \
   setsid nohup $SCHED_BIN --config $SCHED_CFG >/tmp/sched-s39b.log 2>&1 </dev/null & disown; exit 0" 2>/dev/null

# ── 1노드 구성 (king only) ────────────────────────────────────────────
# queen·jack 이 떠 있으면 round-robin 이 나눠 가져 king 부하가 달라진다.
echo "--- 1노드 구성 (king only) ---"
off npuforge-q
off npuforge-j
on  npuforge-k
sleep 12

PID=$(kpid)
[ -n "$PID" ] || { echo "king 노드가 뜨지 않았다"; exit 1; }
echo "king npuforge-node pid=$PID"

# 노드가 실제로 1개인지 물증을 남긴다 (조용한 실패 방지).
ssh -o BatchMode=yes npuforge-server \
  "mkdir -p $REMOTE_OUT/probe; rm -f $REMOTE_OUT/probe/*.json; \
   $BENCH --scheduler $SCHED --model yolov8n --concurrency $CONC --duration 10 \
   --policy probe --out $REMOTE_OUT/probe >/dev/null 2>&1; exit 0" 2>/dev/null
seen=$(read_bench "$REMOTE_OUT/probe/*.json" nodes)
[ "$seen" = "king" ] || { echo "!! 노드 구성 불일치: 기대 king, 관측 '$seen'"; exit 1; }
echo "  1노드(king) 확인"

# ── 조건 1: idle ──────────────────────────────────────────────────────
if want idle; then
echo "--- [1/4] idle (${PROF_DUR}s) — 계측기 바닥값 ---"
ssh -o BatchMode=yes npuforge-k "bash $COLLECTOR idle $PID $PROF_DUR $REMOTE_OUT" 2>&1 | tail -1
fi

# ── 조건 2: 운영점 ────────────────────────────────────────────────────
if want op; then
echo "--- [2/4] 운영점 1N c$CONC conn$CONNS (부하 ${LOAD_DUR}s / 수집 ${PROF_DUR}s @ t+${WARM}) ---"
ssh -o BatchMode=yes npuforge-server \
  "mkdir -p $REMOTE_OUT/op; rm -f $REMOTE_OUT/op/*.json; $BENCH --scheduler $SCHED --model yolov8n \
   --concurrency $CONC --duration $LOAD_DUR --policy round-robin --out $REMOTE_OUT/op \
   >$REMOTE_OUT/op/bench.log 2>&1" 2>/dev/null &
BJ=$!
sleep $WARM
ssh -o BatchMode=yes npuforge-k "bash $COLLECTOR op $PID $PROF_DUR $REMOTE_OUT" 2>&1 | tail -1
wait $BJ 2>/dev/null
OP_TP=$(read_bench "$REMOTE_OUT/op/*.json" throughput)
echo "  운영점 throughput: $OP_TP"
mkdir -p "$OUT/raw/op"; printf '%s\n' "$OP_TP" > "$OUT/raw/op/throughput.txt"
sleep 10
fi

# ── 조건 3: strace (syscall 체류시간 상한) ────────────────────────────
if want strace; then
echo "--- [3/4] strace -c (부하 중 ${STRACE_DUR}s) — ptrace 오버헤드로 부풀려진 **상한** ---"
ssh -o BatchMode=yes npuforge-server \
  "mkdir -p $REMOTE_OUT/st; rm -f $REMOTE_OUT/st/*.json; $BENCH --scheduler $SCHED --model yolov8n \
   --concurrency $CONC --duration 60 --policy round-robin --out $REMOTE_OUT/st \
   >$REMOTE_OUT/st/bench.log 2>&1" 2>/dev/null &
BJ=$!
sleep $WARM
ssh -o BatchMode=yes npuforge-k \
  "mkdir -p $REMOTE_OUT/strace; \
   timeout -s INT $STRACE_DUR strace -c -f -p $PID -o $REMOTE_OUT/strace/summary.txt >/dev/null 2>&1; exit 0" 2>/dev/null
wait $BJ 2>/dev/null
echo "  strace 중 throughput(참고 — 크게 낮아진다): $(read_bench "$REMOTE_OUT/st/*.json" throughput)"
# 노드가 ptrace 후에도 멀쩡한지 확인한다.
ssh -o BatchMode=yes npuforge-k "pgrep npuforge-node >/dev/null && echo '  노드 생존 확인' || echo '  !! 노드가 죽었다'" 2>/dev/null
sleep 10
fi

# ── 조건 4: local direct ──────────────────────────────────────────────
if want local; then
echo "--- [4/4] local direct 8thr (부하 ${LOAD_DUR}s / 수집 ${PROF_DUR}s @ t+${WARM}) ---"
off npuforge-k
sleep 5   # NPU 컨텍스트 정리
ssh -o BatchMode=yes npuforge-k "bash $LOCALW $RKNN_MODEL $LOAD_DUR 8 $WARM $PROF_DUR $REMOTE_OUT $COLLECTOR" 2>&1 | tee /tmp/s39b-local.txt | sed 's/^/  /'
fi

# local direct 의 처리량을 래퍼 출력에서 뽑아 남긴다. 못 뽑으면 비워 두고
# 분석기가 기준값(161.5)으로 넘어가되 그 사실을 표시한다.
if want local; then
    mkdir -p "$OUT/raw/local"
    grep -oE '[0-9]+\.[0-9]+ *(inf/s|infer/s|FPS|fps)' /tmp/s39b-local.txt 2>/dev/null         | grep -oE '^[0-9]+\.[0-9]+' | tail -1 > "$OUT/raw/local/throughput.txt" || true
    cp /tmp/s39b-local.txt "$OUT/raw/local/wrapper-output.txt" 2>/dev/null || true
fi

# ── 수거 ──────────────────────────────────────────────────────────────
echo "--- 수거 ---"
ssh -o BatchMode=yes npuforge-k "cd $REMOTE_OUT && tar cf - ." 2>/dev/null | tar xf - -C "$OUT/raw" 2>/dev/null

printf 'freeze_commit=%s\ncooling=fan\nnodes=1 (king)\nconc=%s\nconns_per_node=%s\nload_dur=%s\nprof_dur=%s\nstrace_dur=%s\n' \
  "$FREEZE" "$CONC" "$CONNS" "$LOAD_DUR" "$PROF_DUR" "$STRACE_DUR" > "$OUT/meta.txt"

# ── 원상복구 ──────────────────────────────────────────────────────────
echo "--- 클러스터 복구 (3노드 + 기본 스케줄러) ---"
on npuforge-k; on npuforge-q; on npuforge-j
ssh -o BatchMode=yes npuforge-server \
  "pkill -9 npuforge-schedu 2>/dev/null; sleep 2; \
   setsid nohup $SCHED_BIN --config /root/scheduler.toml >/tmp/sched.log 2>&1 </dev/null & disown; exit 0" 2>/dev/null
sleep 8

echo
PYTHONIOENCODING=utf-8 python scripts/analyze-node-residual.py "$OUT" 2>&1 || echo "(분석기 실패 — 원본은 $OUT/raw)"
echo "결과: $OUT/"
