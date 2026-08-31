#!/usr/bin/env bash
# S3 — saturation sweep. 각 노드 수의 진짜 처리량 ceiling 을 찾는다.
#
# S2(scaling)와 다른 실험이다:
#   S2 = 동일 노드당 부하(c=8N)에서 선형성
#   S3 = 각 구성의 최대 처리량 (concurrency 를 올려 포화점 탐색)
# 두 실험을 섞지 않는다.
#
# 동결 유지(bench 254d560, S2 와 동일 코드·조건). duration 30초.
# 각 (nodes, concurrency) 3회, 조건 순서 rotate.
set -u

OUT=/tmp/sat30
BENCH=/root/npuforge/target/release/npuforge-bench
SCHED=http://127.0.0.1:50051
DUR=30
ROUNDS=3

FREEZE=$(git rev-parse --short HEAD 2>/dev/null || echo unknown)

# 노드 수별 concurrency sweep
declare -A SWEEP=([1]="4 8 16 32 48" [2]="8 16 24 32 48" [3]="12 24 32 48 64")

on()  { ssh -o BatchMode=yes "$1" 'pgrep npuforge-node >/dev/null 2>&1 || { setsid nohup /home/pi/npuforge/npuforge-node --config /home/pi/npuforge/node.toml >/tmp/node.log 2>&1 </dev/null & disown; }; exit 0' 2>/dev/null; }
off() { ssh -o BatchMode=yes "$1" 'pkill -9 npuforge-node 2>/dev/null; exit 0' 2>/dev/null; }

set_nodes() {
  local n=$1
  on npuforge-k
  if [ "$n" -ge 2 ]; then on npuforge-q; else off npuforge-q; fi
  if [ "$n" -ge 3 ]; then on npuforge-j; else off npuforge-j; fi
  sleep 12
}

ssh -o BatchMode=yes npuforge-server "mkdir -p $OUT/rt; printf 'freeze_commit=%s\nexperiment=S3_saturation\nstarted=%s\n' '$FREEZE' \"\$(date -u +%FT%TZ)\" > $OUT/meta.txt" 2>/dev/null

echo "=== S3 saturation sweep 시작 (freeze $FREEZE) ==="
for round in $(seq 1 "$ROUNDS"); do
  case $(( (round - 1) % 3 )) in
    0) order=(1 2 3);;
    1) order=(2 3 1);;
    2) order=(3 1 2);;
  esac
  for n in "${order[@]}"; do
    set_nodes "$n"
    for conc in ${SWEEP[$n]}; do
      ssh -o BatchMode=yes npuforge-server \
        "rm -f $OUT/rt/*.json 2>/dev/null; $BENCH --scheduler $SCHED --model yolov8n --concurrency $conc --duration $DUR --policy round-robin --out $OUT/rt >/dev/null 2>&1; f=\$(ls $OUT/rt/*.json 2>/dev/null | head -1); [ -n \"\$f\" ] && mv \"\$f\" $OUT/sat_n${n}_c${conc}_r${round}.json" 2>/dev/null
      tp=$(ssh -o BatchMode=yes npuforge-server "python3 -c 'import json; print(round(json.load(open(\"$OUT/sat_n${n}_c${conc}_r${round}.json\"))[\"summary\"][\"throughput\"],1))' 2>/dev/null" 2>/dev/null | tr -d '\r')
      echo "round $round  ${n}N c$conc  →  ${tp:-?} inf/s"
      sleep 5
    done
  done
done
echo "=== S3 완료 ($(( ROUNDS * 15 )) run). 결과: server:$OUT/ ==="
