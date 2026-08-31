#!/usr/bin/env bash
# S3.5b — RPS A/B. S3.5 프로파일링이 지목한 1순위 가설을 검증한다.
#
# 가설
#   노드당 상한(~116 inf/s)을 누르는 것은 대역폭도, 보드 전체 CPU 도 아니다.
#   S3.5 실측:
#     - eth0 RX/TX 각각 1.20 Gbps = 링크 실측 상한(2.34)의 51%  → 대역폭 아님
#     - 보드 전체 63.1% idle                                    → 총 CPU 아님
#     - CPU0 busy 69.7%, 그중 %soft 51.5%. 나머지 코어는 27~38% → CPU0 편중
#   eth0 는 RX 큐가 1개이고 RPS 가 꺼져 있다(rps_cpus=00). 그래서 NET_RX
#   softirq 가 전부 CPU0 한 곳에서 직렬 처리된다.
#
#   → RPS 를 켜서 softirq 를 다른 코어로 분산하면 처리량이 오르는가?
#
# 이걸 먼저 하는 이유
#   맞으면 코드 0 줄로 얻는다. io_uring(S4)은 같은 비용을 훨씬 큰 구현으로
#   공격하는 것이므로, 이 결과가 S4 의 범위와 기대치를 정한다.
#
# 무엇을 건드리는가
#   king 의 /sys/class/net/eth0/queues/rx-0/rps_cpus 만. 런타임 값이라
#   재부팅하면 사라지고, 스크립트 끝에서 원래 값(00)으로 되돌린다.
#   동결 대상(노드/스케줄러 바이너리·설정·모델)은 건드리지 않는다.
#
# 사용법
#   bash scripts/run-rps-ab.sh
set -u

. "$(dirname "${BASH_SOURCE[0]}")/lib/remote.sh"

DATE=$(date +%Y%m%d)
OUT="results/rps-ab-${DATE}"
REMOTE=/tmp/rpsab
BENCH=/root/npuforge/target/release/npuforge-bench
SCHED=http://127.0.0.1:50051
CONC=32
DUR=60
ROUNDS=3

RPS_OFF=00     # 현재값. CPU0 만 처리
RPS_ON=fe      # 코어 1~7. IRQ 가 붙는 CPU0 은 뺀다
RPS_PATH=/sys/class/net/eth0/queues/rx-0/rps_cpus

NODE_BIN=/home/pi/npuforge/npuforge-node
NODE_CFG=/home/pi/npuforge/node.toml
FREEZE=$(git rev-parse --short HEAD 2>/dev/null || echo unknown)

on()  { ssh -o BatchMode=yes "$1" "pgrep npuforge-node >/dev/null 2>&1 || { setsid nohup $NODE_BIN --config $NODE_CFG >/tmp/node.log 2>&1 </dev/null & disown; }; exit 0" 2>/dev/null; }
off() { ssh -o BatchMode=yes "$1" 'pkill -9 npuforge-node 2>/dev/null; exit 0' 2>/dev/null; }

set_rps() {
    npuforge_ssh_sudo npuforge-k "sh -c 'echo $1 > $RPS_PATH'" >/dev/null
    local got
    got=$(ssh -o BatchMode=yes npuforge-k "cat $RPS_PATH" 2>/dev/null | tr -d '\r ')
    if [ "$got" != "$1" ]; then
        echo "  !! rps_cpus 설정 실패 (원했다: $1, 실제: $got)"
        return 1
    fi
    return 0
}

mkdir -p "$OUT/raw"
echo "=== S3.5b RPS A/B (freeze $FREEZE) ==="

# S3 ceiling 과 같은 1노드 구성
echo "--- 1노드 구성 (king only) ---"
off npuforge-q
off npuforge-j
on  npuforge-k
sleep 12

ORIG=$(ssh -o BatchMode=yes npuforge-k "cat $RPS_PATH" 2>/dev/null | tr -d '\r ')
echo "rps_cpus 원래값: $ORIG"
ssh -o BatchMode=yes npuforge-k "mkdir -p $REMOTE" 2>/dev/null
ssh -o BatchMode=yes npuforge-server "mkdir -p $REMOTE" 2>/dev/null

run_one() {
    local cond=$1 rps=$2 round=$3
    set_rps "$rps" || return 1
    # softirq 분산 상태를 같이 남긴다. 처리량만 보면 왜 그런지 모른다.
    ssh -o BatchMode=yes npuforge-k \
        "mpstat -P ALL 1 40 > $REMOTE/mpstat_${cond}_r${round}.txt 2>&1" &
    local MP=$!
    # bench 는 자기가 파일명을 정하므로 스테이징 디렉터리(rt)에 받고 꺼내 온다.
    # 예전엔 출력 디렉터리를 통째로 비워 앞선 run 의 원본까지 지웠다.
    # results.csv 만 보고 있으면 눈치채지 못한다.
    ssh -o BatchMode=yes npuforge-server \
        "mkdir -p $REMOTE/rt; rm -f $REMOTE/rt/*.json; $BENCH --scheduler $SCHED --model yolov8n \
         --concurrency $CONC --duration $DUR --policy round-robin \
         --out $REMOTE/rt >/dev/null 2>&1; \
         f=\$(ls $REMOTE/rt/*.json 2>/dev/null | head -1); \
         [ -n \"\$f\" ] && mv \"\$f\" $REMOTE/${cond}_r${round}.json" 2>/dev/null
    wait $MP 2>/dev/null
    local tp
    tp=$(ssh -o BatchMode=yes npuforge-server \
        "python3 -c 'import json;print(round(json.load(open(\"$REMOTE/${cond}_r${round}.json\"))[\"summary\"][\"throughput\"],1))'" 2>/dev/null | tr -d '\r')
    local soft0
    soft0=$(ssh -o BatchMode=yes npuforge-k \
        "awk '/^Average:  *0 /{print \$8}' $REMOTE/mpstat_${cond}_r${round}.txt" 2>/dev/null | tr -d '\r')
    printf '  round %s  %-8s rps=%s  →  %6s inf/s   (CPU0 %%soft %s)\n' \
        "$round" "$cond" "$rps" "${tp:-?}" "${soft0:-?}"
    echo "$cond,$round,$rps,${tp:-},${soft0:-}" >> "$OUT/raw/results.csv"
    sleep 8
}

echo "cond,round,rps_cpus,throughput,cpu0_soft" > "$OUT/raw/results.csv"

for r in $(seq 1 $ROUNDS); do
    # 순서를 라운드마다 뒤집는다. 시간 경과·온도 상승이 한 조건에만
    # 몰리면 A/B 가 아니라 A-먼저/B-나중 비교가 된다.
    if [ $((r % 2)) -eq 1 ]; then
        run_one off "$RPS_OFF" "$r"; run_one on "$RPS_ON" "$r"
    else
        run_one on "$RPS_ON" "$r"; run_one off "$RPS_OFF" "$r"
    fi
done

echo "--- 원상복구 ---"
set_rps "$ORIG" && echo "rps_cpus 복구: $(ssh -o BatchMode=yes npuforge-k "cat $RPS_PATH" 2>/dev/null | tr -d '\r')"
on npuforge-q
sleep 3

scp -q -r -o BatchMode=yes "npuforge-k:$REMOTE/mpstat_*" "$OUT/raw/" 2>/dev/null
ssh -o BatchMode=yes npuforge-server "cd $REMOTE && tar cf - *.json" 2>/dev/null \
    | tar xf - -C "$OUT/raw" 2>/dev/null

printf 'freeze_commit=%s\nconcurrency=%s\nduration=%s\nrounds=%s\nrps_on=%s\nrps_off=%s\n' \
    "$FREEZE" "$CONC" "$DUR" "$ROUNDS" "$RPS_ON" "$RPS_OFF" > "$OUT/meta.txt"

echo
echo "=== 요약 ==="
python - "$OUT/raw/results.csv" <<'PYEOF'
import csv, statistics, sys
rows = list(csv.DictReader(open(sys.argv[1], encoding="utf-8")))
g = {}
for r in rows:
    if r["throughput"]:
        g.setdefault(r["cond"], []).append(float(r["throughput"]))
for c in ("off", "on"):
    if c in g:
        v = g[c]
        sd = statistics.stdev(v) if len(v) > 1 else 0.0
        print(f"  rps {c:3s} : {statistics.mean(v):6.1f} +- {sd:.1f} inf/s   {[round(x,1) for x in v]}")
if "off" in g and "on" in g:
    a, b = statistics.mean(g["off"]), statistics.mean(g["on"])
    print(f"  차이     : {b - a:+.1f} inf/s ({(b / a - 1) * 100:+.1f}%)")
PYEOF
echo "결과: $OUT/"
