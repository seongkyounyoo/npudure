#!/usr/bin/env bash
# S3.5 — transport cost profiling. io_uring(S4) 착수 전 사전 측정.
#
# 왜 S4 앞에 이것이 있는가
#   `01-TECHSPEC.md` §15.1 의 순서는 2.CPU profile → 3.syscall·복사 비용
#   → 4.버퍼 풀 → 5.io_uring 이다. 지금 2~4 가 비어 있다. 그리고 §15.4 가
#   요구하는 지표(syscalls/req, ctx switches/req, cycles/req)는 S4 의
#   "before" 기준값으로 어차피 필요한데 저장소에 하나도 없다.
#   먼저 만들고 나서 재면 개선을 무엇에 귀속시킬 근거가 없다.
#
# 답할 질문
#   노드당 ~115 inf/s (로컬 direct 161.5 대비 -30%) 를 막는 것이 무엇인가.
#
#   대역폭은 아니다. 2.5GbE 는 full-duplex 이고 방향당 실측 상한 2.34 Gbps
#   대비 RX 1.13 / TX 1.12 Gbps → 약 48% 만 쓴다. 3노드에서 3.00× 선형
#   확장이 나온 것도 공유 링크가 병목이 아님을 뒷받침한다.
#   → 남는 후보는 보드 로컬 자원, 그중 CPU 다. 이 스크립트가 그것을 잰다.
#
# 조건 3개를 같은 보드(king)에서 잰다. 비교 가능해야 하므로 순서·냉각·
# governor 를 S2·S3 와 동일하게 둔다.
#
#   idle     부하 없음.              계측기 자체의 바닥값
#   cluster  1노드 클러스터 c32.     S3 ceiling 조건 (115.2 inf/s)
#   local    로컬 direct 8스레드.    네트워크 경로가 통째로 빠진 조건 (161.5)
#
# cluster 와 local 의 차이가 곧 transport 가 보드에서 쓰는 비용이다.
#
# 사용법
#   bash scripts/run-transport-profile.sh              세 조건 모두
#   bash scripts/run-transport-profile.sh --only local 한 조건만 다시
#
# 결과
#   results/transport-profile-<날짜>/raw/{idle,cluster,local}/
set -u

DATE=$(date +%Y%m%d)
LOCAL_OUT="results/transport-profile-${DATE}/raw"
REMOTE_OUT=/tmp/tprofile
BENCH=/root/npuforge/target/release/npuforge-bench
SCHED=http://127.0.0.1:50051

# 부하는 넉넉히 돌리고 그 안쪽만 잰다. 시작·종료 구간의 램프와 warmup 이
# 평균을 오염시키지 않게 한다.
LOAD_DUR=80
WARM=20      # 부하 시작 후 이만큼 지나서 수집을 시작한다
PROF_DUR=45  # 수집 창 (WARM+PROF_DUR < LOAD_DUR 이어야 한다)
CONC=32      # S3 에서 1노드 ceiling 이 나온 동시성

NODE_BIN=/home/pi/npuforge/npuforge-node
NODE_CFG=/home/pi/npuforge/node.toml
RKNN_DIR=/home/pi/npuforge-rknn-test
RKNN_MODEL=yolov8n-int8.rknn
COLLECTOR=/home/pi/node-profile-collect.sh
LOCALW=/home/pi/node-profile-local.sh

# 조건 하나만 다시 재고 싶을 때가 있다. 한 번 실패한 조건 때문에 정상인
# 두 조건까지 다시 도는 것은 보드를 40분 더 달구는 일이다.
ONLY=all
[ "${1:-}" = "--only" ] && ONLY=${2:?--only 뒤에 idle|cluster|local}
want() { [ "$ONLY" = all ] || [ "$ONLY" = "$1" ]; }

FREEZE=$(git rev-parse --short HEAD 2>/dev/null || echo unknown)

on()  { ssh -o BatchMode=yes "$1" "pgrep npuforge-node >/dev/null 2>&1 || { setsid nohup $NODE_BIN --config $NODE_CFG >/tmp/node.log 2>&1 </dev/null & disown; }; exit 0" 2>/dev/null; }
# -f 를 쓰면 패턴이 자기 셸에도 걸려 ssh 세션을 죽인다 (ADR-017).
off() { ssh -o BatchMode=yes "$1" 'pkill -9 npuforge-node 2>/dev/null; exit 0' 2>/dev/null; }

kpid() { ssh -o BatchMode=yes npuforge-k 'pgrep npuforge-node | head -1' 2>/dev/null | tr -d '\r'; }

echo "=== S3.5 transport profiling (freeze $FREEZE) ==="

# 수집기를 보드에 올린다.
scp -q -o BatchMode=yes scripts/node-profile-collect.sh npuforge-k:$COLLECTOR || {
    echo "수집기 전송 실패"; exit 1; }
scp -q -o BatchMode=yes scripts/node-profile-local.sh npuforge-k:$LOCALW || {
    echo "local 래퍼 전송 실패"; exit 1; }
# 다시 재는 조건의 디렉터리만 지운다. 통째로 지우면 --only 로 한 조건만
# 다시 잴 때 나머지 원본이 날아간다.
ssh -o BatchMode=yes npuforge-k "chmod +x $COLLECTOR $LOCALW; mkdir -p $REMOTE_OUT;     [ '$ONLY' = all ] && rm -rf $REMOTE_OUT/idle $REMOTE_OUT/cluster $REMOTE_OUT/local;     rm -rf $REMOTE_OUT/$ONLY; exit 0" 2>/dev/null

# ── 1노드 구성 ────────────────────────────────────────────────────────
# S3 ceiling 과 같은 조건을 만든다. queen·jack 이 떠 있으면 round-robin 이
# 요청을 나눠 가져 king 의 부하가 달라진다.
echo "--- 1노드 구성 (king only) ---"
off npuforge-q
off npuforge-j
on  npuforge-k
sleep 12   # 등록/해제 안정화 + NPU 컨텍스트 정리

PID=$(kpid)
if [ -z "$PID" ]; then echo "king 노드가 뜨지 않았다"; exit 1; fi
echo "king npuforge-node pid=$PID"

# ── 조건 1: idle ──────────────────────────────────────────────────────
if want idle; then
echo "--- [1/3] idle (${PROF_DUR}s) ---"
ssh -o BatchMode=yes npuforge-k "bash $COLLECTOR idle $PID $PROF_DUR $REMOTE_OUT" 2>&1 | tail -1
fi

# ── 조건 2: cluster 1노드 c32 ─────────────────────────────────────────
if want cluster; then
echo "--- [2/3] cluster 1N c$CONC (부하 ${LOAD_DUR}s / 수집 ${PROF_DUR}s @ t+${WARM}) ---"
ssh -o BatchMode=yes npuforge-server \
  "rm -rf $REMOTE_OUT; mkdir -p $REMOTE_OUT; $BENCH --scheduler $SCHED --model yolov8n \
   --concurrency $CONC --duration $LOAD_DUR --policy round-robin --out $REMOTE_OUT \
   >$REMOTE_OUT/bench.log 2>&1" 2>/dev/null &
BENCH_JOB=$!

sleep $WARM
ssh -o BatchMode=yes npuforge-k "bash $COLLECTOR cluster $PID $PROF_DUR $REMOTE_OUT" 2>&1 | tail -1
wait $BENCH_JOB 2>/dev/null
echo -n "cluster throughput: "
ssh -o BatchMode=yes npuforge-server \
  "python3 -c 'import json,glob; f=sorted(glob.glob(\"$REMOTE_OUT/*.json\"))[0]; d=json.load(open(f)); print(round(d[\"summary\"][\"throughput\"],1))'" 2>/dev/null | tr -d '\r'

fi

sleep 10   # cooldown. 다음 조건의 상승 곡선을 오염시키지 않는다

# ── 조건 3: local direct ──────────────────────────────────────────────
# 네트워크 경로가 통째로 빠진다. 노드를 내려야 NPU 컨텍스트가 풀린다.
#
# 기동과 수집을 보드의 한 세션 안에서 한다. 개발 PC 에서 띄우고 pgrep 으로
# 찾는 방식은 setsid 의 이중 fork 와 경쟁 때문에 조용히 실패했다
# (`node-profile-local.sh` 주석 참조).
if want local; then
echo "--- [3/3] local direct 8thr (부하 ${LOAD_DUR}s / 수집 ${PROF_DUR}s @ t+${WARM}) ---"
off npuforge-k
sleep 5    # NPU 컨텍스트 정리 대기

ssh -o BatchMode=yes npuforge-k   "bash $LOCALW $RKNN_MODEL $LOAD_DUR 8 $WARM $PROF_DUR $REMOTE_OUT $COLLECTOR" 2>&1   | sed 's/^/  /'
fi

# ── 원상복구 ──────────────────────────────────────────────────────────
# 이 스크립트가 껐던 것만 되돌린다. jack 은 시작 전부터 죽어 있었으므로
# 여기서 손대지 않는다 — 원인(eth0 링크 플랩)이 따로 있고, 조용히 살려
# 두면 다음 사람이 그 사실을 모른다.
echo "--- 노드 복구 (king, queen) ---"
on npuforge-k
on npuforge-q
sleep 5
for h in npuforge-k npuforge-q npuforge-j; do
    printf '  %-16s ' "$h"
    ssh -o BatchMode=yes "$h" 'pgrep npuforge-node >/dev/null && echo running || echo DOWN' 2>/dev/null | tr -d '
'
done

# ── 수거 ──────────────────────────────────────────────────────────────
mkdir -p "$LOCAL_OUT"
scp -q -r -o BatchMode=yes "npuforge-k:$REMOTE_OUT/idle"    "$LOCAL_OUT/" 2>/dev/null
scp -q -r -o BatchMode=yes "npuforge-k:$REMOTE_OUT/cluster" "$LOCAL_OUT/" 2>/dev/null
scp -q -r -o BatchMode=yes "npuforge-k:$REMOTE_OUT/local"   "$LOCAL_OUT/" 2>/dev/null
scp -q -o BatchMode=yes "npuforge-k:/tmp/local-direct.log"  "$LOCAL_OUT/local/direct.log" 2>/dev/null
ssh -o BatchMode=yes npuforge-server "ls $REMOTE_OUT/*.json 2>/dev/null | head -1" 2>/dev/null | tr -d '\r' | while read -r f; do
    [ -n "$f" ] && scp -q -o BatchMode=yes "npuforge-server:$f" "$LOCAL_OUT/cluster/bench.json" 2>/dev/null
done

printf 'freeze_commit=%s\nconcurrency=%s\nload_dur=%s\nprof_dur=%s\nwarm=%s\ncollected=%s\n' \
  "$FREEZE" "$CONC" "$LOAD_DUR" "$PROF_DUR" "$WARM" "$(date -u +%FT%TZ)" > "$LOCAL_OUT/meta.txt"

echo "=== 완료 → $LOCAL_OUT ==="
find "$LOCAL_OUT" -type f | sort | sed 's/^/  /'
