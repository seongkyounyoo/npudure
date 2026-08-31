#!/usr/bin/env bash
# S3.6 — HTTP/2 window × 노드당 커넥션 수 A/B.
#
# 무엇을 가르는 실험인가
#   S3.5 는 노드당 −30% 손실(116 vs 로컬 direct 161.5)이 스케줄러↔노드
#   HTTP/2 전송 경로에 있다는 데까지 좁혔다. 배제된 것:
#     대역폭(방향당 51%), 보드 CPU 총량(63% idle), CPU0 softirq(RPS A/B
#     −0.2%), 서버·스케줄러(3노드 3.00× 선형).
#   하지만 그 경로 안에서 무엇인지는 갈리지 않았다.
#
#     ① flow control   64 KB 기본 window 가 1.2 MB 메시지를 막는가
#     ② 커넥션/TCP     커넥션 상태 기계·소켓 하나가 직렬화 지점인가
#     ③ protobuf·복사  프레이밍과 encode/decode 비용인가
#
#   2×2 로 ①②를 가른다. ③은 배제법으로 남는다.
#
#     Test  커넥션/노드  H2 window   목적
#     A         1        default     baseline (= 현재 115)
#     B         1        확대        ① flow control 검증
#     C         4        default     ② 커넥션/TCP 검증
#     D         4        확대        결합
#
#   해석
#     B 만 상승        → 범인은 커넥션 수가 아니라 flow control
#     C 만 상승        → 범인은 단일 커넥션 / TCP 경로
#     B·C 둘 다 상승   → 둘 다 영향
#     D 까지 그대로    → ①② 아님 → ③ 으로 복귀. 이때 io_uring 이
#                        훨씬 강한 근거를 갖는다
#
#   HTTP/2 는 원래 커넥션 하나에서 스트림을 다중화하라고 만든 프로토콜이다.
#   "커넥션이 1개" 라는 사실만으로 병목이라 단정하면 안 된다. 그래서 B 가
#   중요하다.
#
#   window 는 최적값 탐색이 아니다. **64 KB 급 기본값이 막고 있었는지
#   여부만** 본다. 그래서 한 메시지(1.23 MB)가 통째로 들어가고도 남는
#   크기로 크게 잡는다.
#
# 무엇을 건드리는가
#   스케줄러·노드를 S3.6 빌드로 띄우고 설정에 [transport] 를 넣는다.
#   동결 바이너리는 `*.frozen-01f29a2` 로 남겨 두었다. 기본값은 기존 동작과
#   같으므로 **조건 A 가 ~115 를 재현하면 그 자체가 회귀 검사**다.
#
# 사용법
#   bash scripts/run-h2-channel-ab.sh [라운드수]     기본 5
set -u

ROUNDS=${1:-5}

DATE=$(date +%Y%m%d)
OUT="results/h2-channel-ab-${DATE}"
REMOTE=/tmp/h2ab

BENCH=/root/npuforge/target/release/npuforge-bench
SCHED_BIN=/root/npuforge/target/release/npuforge-scheduler
SCHED_CFG=/root/scheduler-s36.toml
SCHED_URL=http://127.0.0.1:50051
CONC=32
DUR=60
PROF=40          # 수집 창. DUR 보다 짧아야 램프가 안 섞인다
WARM=10          # 부하 시작 후 이만큼 지나 수집 시작

NODE_BIN=/home/pi/npuforge/npuforge-node.s36
NODE_CFG=/home/pi/npuforge/node-s36.toml
COLLECTOR=/home/pi/node-profile-collect.sh

# 한 메시지가 1.23 MB 다. stream 8 MB 면 메시지 하나가 WINDOW_UPDATE 없이
# 통째로 들어간다. connection 64 MB 면 c32 동시 요청이 전부 in-flight 가
# 될 수 있다(32 × 1.23 = 39 MB). window 는 전송 허가지 선할당이 아니므로
# 보드 메모리(2.7 GB 여유)에 부담이 없다.
WIN_STREAM=8388608      # 8 MiB
WIN_CONN=67108864       # 64 MiB

FREEZE=$(git rev-parse --short HEAD 2>/dev/null || echo unknown)

mkdir -p "$OUT/raw"

# ── 원격 제어 ─────────────────────────────────────────────────────────
# pkill 에 -f 를 쓰지 않는다. 패턴이 자기 ssh 셸에도 걸린다 (ADR-017).
# 프로세스명은 15자에서 잘리므로 스케줄러는 'npuforge-schedu' 로 잡는다.
sched_stop() { ssh -o BatchMode=yes npuforge-server 'pkill -9 npuforge-schedu 2>/dev/null; exit 0' 2>/dev/null; }
node_stop()  { ssh -o BatchMode=yes npuforge-k 'pkill -9 npuforge-node 2>/dev/null; exit 0' 2>/dev/null; }

sched_start() {
    ssh -o BatchMode=yes npuforge-server \
        "setsid nohup $SCHED_BIN --config $SCHED_CFG >/tmp/sched-s36.log 2>&1 </dev/null & disown; exit 0" 2>/dev/null
}
node_start() {
    ssh -o BatchMode=yes npuforge-k \
        "setsid nohup $NODE_BIN --config $NODE_CFG >/tmp/node-s36.log 2>&1 </dev/null & disown; exit 0" 2>/dev/null
}

# 조건에 맞는 설정을 만들고 두 프로세스를 다시 띄운다.
apply() {
    local conns=$1 win=$2

    sched_stop; node_stop
    sleep 4      # NPU 컨텍스트 정리 + 포트 반납

    # [transport] 블록을 ssh 표준입력으로 흘려 붙인다.
    # 예전엔 원격 printf 에 이스케이프한 개행을 넘겼는데 따옴표가 3중으로
    # 겹치며 조용히 깨졌다. 여기서 진짜 개행을 만들어 보내면 원격은 그냥
    # 받아 적기만 하면 된다.
    {
        echo
        echo "[transport]"
        echo "node_connections = $conns"
        if [ "$win" = "big" ]; then
            echo "h2_stream_window_bytes = $WIN_STREAM"
            echo "h2_connection_window_bytes = $WIN_CONN"
        fi
    } | ssh -o BatchMode=yes npuforge-server \
        "cp /root/scheduler-base.toml $SCHED_CFG && cat >> $SCHED_CFG" 2>/dev/null

    # 노드는 서버라 커넥션 수를 정하지 않는다. window 만 광고한다.
    # default 조건에서는 [transport] 자체를 안 붙인다 — 빈 섹션을 붙이면
    # "설정했는데 기본값" 인지 "안 붙었는지" 구분이 안 된다.
    if [ "$win" = "big" ]; then
        {
            echo
            echo "[transport]"
            echo "h2_stream_window_bytes = $WIN_STREAM"
            echo "h2_connection_window_bytes = $WIN_CONN"
        } | ssh -o BatchMode=yes npuforge-k \
            "cp /home/pi/npuforge/node.toml $NODE_CFG && cat >> $NODE_CFG" 2>/dev/null
    else
        ssh -o BatchMode=yes npuforge-k "cp /home/pi/npuforge/node.toml $NODE_CFG" 2>/dev/null
    fi

    # 스케줄러를 먼저 띄운다. 노드는 등록을 재시도하지만 순서를 지키면
    # 첫 등록이 바로 붙어 대기가 짧다.
    sched_start
    sleep 3
    node_start
    sleep 12     # 등록 + warmup

    # 정말 그 설정으로 떴는지 확인한다. 설정이 조용히 무시되면 A/B 가
    # 아니라 같은 조건 4번이 된다. 두 프로세스가 로그에 찍는 "전송 설정"
    # 줄을 근거로 삼는다.
    if ! ssh -o BatchMode=yes npuforge-server "grep -q node_connections /tmp/sched-s36.log" 2>/dev/null; then
        echo "  !! 스케줄러가 전송 설정을 읽지 못했다"
        ssh -o BatchMode=yes npuforge-server 'tail -5 /tmp/sched-s36.log' 2>/dev/null
        return 1
    fi
    if ! ssh -o BatchMode=yes npuforge-k "pgrep npuforge-node >/dev/null" 2>/dev/null; then
        echo "  !! 노드가 뜨지 않았다"
        ssh -o BatchMode=yes npuforge-k 'tail -5 /tmp/node-s36.log' 2>/dev/null
        return 1
    fi
    return 0
}

run_one() {
    local cond=$1 conns=$2 win=$3 round=$4
    if ! apply "$conns" "$win"; then
        echo "$cond,$round,$conns,$win,,,,,,,," >> "$OUT/raw/results.csv"
        return 1
    fi

    local pid
    pid=$(ssh -o BatchMode=yes npuforge-k 'pgrep npuforge-node | head -1' 2>/dev/null | tr -d '\r')

    ssh -o BatchMode=yes npuforge-server \
        "mkdir -p $REMOTE/rt; rm -f $REMOTE/rt/*.json; \
         $BENCH --scheduler $SCHED_URL --model yolov8n --concurrency $CONC \
         --duration $DUR --policy round-robin --out $REMOTE/rt >/dev/null 2>&1; \
         f=\$(ls $REMOTE/rt/*.json 2>/dev/null | head -1); \
         [ -n \"\$f\" ] && mv \"\$f\" $REMOTE/${cond}_r${round}.json" 2>/dev/null &
    local BJ=$!

    sleep "$WARM"
    # 노드가 실제로 몇 개의 커넥션을 받고 있는지 — 설정이 먹었다는 물증.
    # 이게 conns 와 다르면 그 run 은 의도한 조건이 아니다.
    local nconn
    nconn=$(ssh -o BatchMode=yes npuforge-k \
        "ss -tn state established '( sport = :51001 )' 2>/dev/null | tail -n +2 | wc -l" 2>/dev/null | tr -d '\r')
    ssh -o BatchMode=yes npuforge-k "bash $COLLECTOR ${cond}_r${round} $pid $PROF $REMOTE" >/dev/null 2>&1
    wait $BJ 2>/dev/null

    local m
    m=$(ssh -o BatchMode=yes npuforge-server \
        "python3 -c \"
import json
d=json.load(open('$REMOTE/${cond}_r${round}.json'))
s=d['summary']; sb=s['stage_breakdown']
print('%.1f,%.2f,%.2f,%.2f,%.2f,%.3f,%.4f' % (
  s['throughput'], s['latency']['p50']/1000, s['latency']['p95']/1000,
  sb['network_to_node']['p50']/1000, sb['network_to_client']['p50']/1000,
  sb['node_queue']['p50']/1000, s['error_rate']))
\"" 2>/dev/null | tr -d '\r')

    echo "$cond,$round,$conns,$win,$nconn,$m" >> "$OUT/raw/results.csv"
    printf '  round %s  %-14s conn=%s win=%-7s tcp=%-2s -> %s\n' \
        "$round" "$cond" "$conns" "$win" "${nconn:-?}" "${m:-실패}"
    sleep 8
}

echo "=== S3.6 H2/channel A/B (freeze $FREEZE, ${ROUNDS}라운드 × 4조건) ==="

# 1노드 구성. queen/jack 이 떠 있으면 round-robin 이 부하를 나눈다.
ssh -o BatchMode=yes npuforge-q 'pkill -9 npuforge-node 2>/dev/null; exit 0' 2>/dev/null
ssh -o BatchMode=yes npuforge-j 'pkill -9 npuforge-node 2>/dev/null; exit 0' 2>/dev/null
scp -q -o BatchMode=yes scripts/node-profile-collect.sh npuforge-k:$COLLECTOR
ssh -o BatchMode=yes npuforge-k "chmod +x $COLLECTOR; mkdir -p $REMOTE" 2>/dev/null
ssh -o BatchMode=yes npuforge-server "mkdir -p $REMOTE" 2>/dev/null

echo "cond,round,conns,win,tcp_conns,throughput,e2e_p50_ms,e2e_p95_ms,net_to_node_ms,net_to_client_ms,node_queue_ms,error_rate" \
    > "$OUT/raw/results.csv"

for r in $(seq 1 "$ROUNDS"); do
    # 라운드마다 순서를 돌린다. 온도 상승·시간 경과가 한 조건에 몰리면
    # A/B 가 아니라 먼저/나중 비교가 된다.
    case $(( (r - 1) % 4 )) in
        0) order="A B C D";;
        1) order="B C D A";;
        2) order="C D A B";;
        3) order="D A B C";;
    esac
    for c in $order; do
        case $c in
            A) run_one A_1ch_default 1 default "$r";;
            B) run_one B_1ch_bigwin  1 big     "$r";;
            C) run_one C_4ch_default 4 default "$r";;
            D) run_one D_4ch_bigwin  4 big     "$r";;
        esac
    done
done

# ── 원상복구 ──────────────────────────────────────────────────────────
# 기본 설정(= 동결과 동일 동작)으로 되돌려 둔다. 다음 사람이 그대로
# 이어받을 수 있어야 한다. jack 은 S3.6 전부터 죽어 있었으므로 건드리지
# 않는다 — 조용히 살려 두면 그 사실이 묻힌다.
echo "--- 원상복구 (기본 설정으로 재기동) ---"
apply 1 default >/dev/null 2>&1
ssh -o BatchMode=yes npuforge-q \
    "setsid nohup /home/pi/npuforge/npuforge-node --config /home/pi/npuforge/node.toml >/tmp/node.log 2>&1 </dev/null & disown; exit 0" 2>/dev/null
sleep 3
for h in npuforge-k npuforge-q npuforge-j; do
    printf '  %-16s ' "$h"
    ssh -o BatchMode=yes "$h" 'pgrep npuforge-node >/dev/null && echo running || echo DOWN' 2>/dev/null | tr -d '\r'
done

scp -q -r -o BatchMode=yes "npuforge-k:$REMOTE" "$OUT/raw/profile" 2>/dev/null
ssh -o BatchMode=yes npuforge-server "cd $REMOTE && tar cf - *.json" 2>/dev/null \
    | tar xf - -C "$OUT/raw" 2>/dev/null

printf 'freeze_commit=%s\nconcurrency=%s\nduration=%s\nrounds=%s\nwin_stream=%s\nwin_conn=%s\n' \
    "$FREEZE" "$CONC" "$DUR" "$ROUNDS" "$WIN_STREAM" "$WIN_CONN" > "$OUT/meta.txt"

echo
PYTHONIOENCODING=utf-8 python scripts/analyze-h2-channel-ab.py "$OUT/raw/results.csv" 2>&1 \
    || cat "$OUT/raw/results.csv"
echo "결과: $OUT/"
