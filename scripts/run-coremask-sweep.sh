#!/usr/bin/env bash
#
# core_mask 분배 방식 비교 실험.
#
# NPU 2코어를 어떻게 배정하느냐에 따라 처리량이 달라지는지 확인한다.
# docs/discuss.md 「아직 하지 않은 검증」 1번 항목.
#
# 보드에서 직접 실행한다:
#   ./run-coremask-sweep.sh [iterations]
#
# 결과는 /tmp/coremask_result.txt 에 쓰고, 완료 시 /tmp/coremask.done 을 만든다.

set -uo pipefail

ITERS="${1:-200}"
PASS="${NPUFORGE_SUDO_PASS:-${NPUFORGE_NODE_PASS:-}}"
DIR="$HOME/npuforge-rknn-test"
OUT=/tmp/coremask_result.txt
MODEL=yolov8n-fp16.rknn

S() { printf '%s\n' "$PASS" | sudo -S -p "" "$@" 2>/dev/null; }

cd "$DIR" || exit 1

# NPU load 는 delayms 창으로 평균된다. 재부팅하면 3000 으로 돌아간다.
printf '%s\n' "$PASS" | sudo -S -p "" sh -c 'echo 100 > /sys/kernel/debug/rknpu/delayms' 2>/dev/null
DELAY=$(S cat /sys/kernel/debug/rknpu/delayms)

{
    echo "# core_mask sweep"
    echo "# host=$(hostname) model=$MODEL iterations=$ITERS delayms=$DELAY"
    echo "# $(date -u +%FT%TZ)"
    printf '%-3s %-4s %-12s %-9s %-9s %-9s %-9s %-11s %-11s\n' \
        "thr" "mode" "name" "tput/s" "run_us" "in_us" "out_us" "Core0 a/max" "Core1 a/max"
} > "$OUT"

for T in 1 2 4 8; do
    for M in 0 1 2 3; do
        ./npu_occupancy_test "$MODEL" "$ITERS" "$T" "$M" > /tmp/cm_o.txt 2>/dev/null &
        PID=$!

        # 워밍업 구간을 평균에서 제외한다
        sleep 4
        for _ in $(seq 1 20); do
            S cat /sys/kernel/debug/rknpu/load
            sleep 0.3
        done > /tmp/cm_l.txt
        wait $PID

        NAME=$(grep -oE 'core_mode=[0-9]\(([A-Z_0-9]+)\)' /tmp/cm_o.txt | sed 's/.*(\(.*\))/\1/')
        TP=$(grep -oE 'throughput_ips=[0-9.]+' /tmp/cm_o.txt | cut -d= -f2)
        RUN=$(grep -oE ' run=[0-9]+' /tmp/cm_o.txt | tr -d ' ' | cut -d= -f2)
        IN=$(grep -oE 'inputs_set=[0-9]+' /tmp/cm_o.txt | cut -d= -f2)
        OUTV=$(grep -oE 'outputs_get=[0-9]+' /tmp/cm_o.txt | cut -d= -f2)
        C0=$(grep -oE 'Core0: *[0-9]+%' /tmp/cm_l.txt | grep -oE '[0-9]+' \
             | awk '{s+=$1;n++; if($1>m)m=$1} END{if(n)printf "%.0f/%d", s/n, m; else printf "-"}')
        C1=$(grep -oE 'Core1: *[0-9]+%' /tmp/cm_l.txt | grep -oE '[0-9]+' \
             | awk '{s+=$1;n++; if($1>m)m=$1} END{if(n)printf "%.0f/%d", s/n, m; else printf "-"}')

        printf '%-3s %-4s %-12s %-9s %-9s %-9s %-9s %-11s %-11s\n' \
            "$T" "$M" "$NAME" "$TP" "$RUN" "$IN" "$OUTV" "$C0" "$C1" >> "$OUT"
        sync
    done
done

echo "DONE=$?" > /tmp/coremask.done
sync
