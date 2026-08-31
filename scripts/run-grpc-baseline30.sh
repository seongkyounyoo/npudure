#!/usr/bin/env bash
# gRPC baseline 30회 반복 측정. 1차 결과를 "재현된 결과"로 승격시키는 단계.
#
# 원칙 (측정 중 코드·설정 동결):
#   - 조건 순서를 섞는다(rotate). 시간 경과·온도 변동을 한 조건에 몰지 않는다.
#   - 각 run 은 bench --out 으로 JSON 저장. throughput / latency(p50/95/99) /
#     node_inference / TimingBreakdown / per_node / nodes_before·after(temp·
#     voltage) / verdict / run_id 가 전부 들어간다.
#   - 이상 run 도 삭제하지 않는다. bench 가 verdict.valid=false + reason 을 남긴다.
#   - 파일명은 n{노드}_r{라운드}.json (run_id 는 같은 조건이면 겹치므로 rename).
#
# 노드 제어는 개발 PC(이 스크립트)에서 ssh 로, bench 는 server 에서 실행한다.
set -u

OUT=/tmp/baseline30
BENCH=/root/npuforge/target/release/npuforge-bench
SCHED=http://127.0.0.1:50051
DUR=60
ROUNDS=10

FREEZE=$(git rev-parse --short HEAD 2>/dev/null || echo unknown)

on()  { ssh -o BatchMode=yes "$1" 'pgrep npuforge-node >/dev/null 2>&1 || { setsid nohup /home/pi/npuforge/npuforge-node --config /home/pi/npuforge/node.toml >/tmp/node.log 2>&1 </dev/null & disown; }; exit 0' 2>/dev/null; }
off() { ssh -o BatchMode=yes "$1" 'pkill -9 npuforge-node 2>/dev/null; exit 0' 2>/dev/null; }

set_nodes() {
  local n=$1
  on npuforge-k
  if [ "$n" -ge 2 ]; then on npuforge-q; else off npuforge-q; fi
  if [ "$n" -ge 3 ]; then on npuforge-j; else off npuforge-j; fi
  sleep 12   # 등록/dereg 안정화 + 노드 warmup + NPU 컨텍스트 정리
}

ssh -o BatchMode=yes npuforge-server "mkdir -p $OUT/rt; printf 'freeze_commit=%s\nbench=timing-expanded\nstarted=%s\n' '$FREEZE' \"\$(date -u +%FT%TZ)\" > $OUT/meta.txt" 2>/dev/null

echo "=== gRPC baseline 30회 시작 (freeze $FREEZE) ==="
for round in $(seq 1 "$ROUNDS"); do
  case $(( (round - 1) % 3 )) in
    0) order=(1 2 3);;
    1) order=(2 3 1);;
    2) order=(3 1 2);;
  esac
  for n in "${order[@]}"; do
    conc=$(( n * 8 ))
    set_nodes "$n"
    ssh -o BatchMode=yes npuforge-server \
      "rm -f $OUT/rt/*.json 2>/dev/null; $BENCH --scheduler $SCHED --model yolov8n --concurrency $conc --duration $DUR --policy round-robin --out $OUT/rt >/dev/null 2>&1; f=\$(ls $OUT/rt/*.json 2>/dev/null | head -1); [ -n \"\$f\" ] && mv \"\$f\" $OUT/n${n}_r${round}.json" 2>/dev/null
    tp=$(ssh -o BatchMode=yes npuforge-server "python3 -c 'import json; print(round(json.load(open(\"$OUT/n${n}_r${round}.json\"))[\"summary\"][\"throughput\"],1))' 2>/dev/null" 2>/dev/null | tr -d '\r')
    echo "round $round  ${n}N c$conc  →  ${tp:-?} inf/s"
    sleep 8   # cooldown
  done
done
echo "=== 30 run 완료. 결과: server:$OUT/ ==="
