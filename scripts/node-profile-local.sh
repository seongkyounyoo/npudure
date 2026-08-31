#!/usr/bin/env bash
# local-direct 조건을 보드에서 한 세션으로 실행한다. S3.5 전용.
#
# 왜 한 세션인가
#   처음에는 개발 PC 에서 `setsid nohup ... &` 로 띄우고 잠시 뒤 pgrep 으로
#   PID 를 찾았다. 두 가지가 겹쳐 실패했다.
#     1. setsid 가 한 번 더 fork 하므로 `$!` 가 실제 프로세스가 아니다.
#     2. 기동과 탐색이 경쟁한다. 부하는 정상 완료되는데 PID 만 못 찾아
#        프로파일이 통째로 비었다 — 조용한 실패라 더 나쁘다.
#   같은 셸에서 띄우면 `$!` 가 정확하고 경쟁도 없다.
#
# 사용법
#   node-profile-local.sh <model> <load_dur> <threads> <warm> <prof_dur> <outdir> <collector>
set -u

MODEL=${1:?model}
LOAD_DUR=${2:?load_dur}
THREADS=${3:?threads}
WARM=${4:?warm}
PROF=${5:?prof_dur}
OUT=${6:?outdir}
COLLECTOR=${7:?collector}

cd /home/pi/npuforge-rknn-test || exit 1

./sustained_load_test "$MODEL" "$LOAD_DUR" "$THREADS" > /tmp/local-direct.log 2>&1 &
LPID=$!

# 프로세스가 실제로 살아 있는지 확인한 뒤에 진행한다.
sleep 2
if ! kill -0 "$LPID" 2>/dev/null; then
    echo "local-direct 기동 실패 (pid=$LPID)"
    tail -5 /tmp/local-direct.log
    exit 1
fi
echo "local-direct pid=$LPID"

sleep $(( WARM - 2 ))
bash "$COLLECTOR" local "$LPID" "$PROF" "$OUT"

wait "$LPID" 2>/dev/null
echo "--- local-direct 결과 ---"
tail -8 /tmp/local-direct.log
