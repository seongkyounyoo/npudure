#!/usr/bin/env bash
# 벤치마크 직전 하드 실패 검사.
#
# 이 스크립트는 "측정을 시작해도 되는가"만 판정한다. 고치지는 않는다.
# 고치는 것은 `fix-node-consistency.sh` 의 몫이다.
#
# 왜 필요한가
#   지금까지 측정을 망친 원인은 대부분 측정 자체가 아니라 전제였다.
#
#     - 문서의 IP 가 낡아 노드를 사망으로 오판했다 (worklog §2.20)
#     - 부하 프로파일이 다른 두 측정을 비교해 19°C 격차로 오해했다 (§2.19)
#     - 어댑터 용량 부족으로 리셋된 보드의 처리량을 성능으로 읽을 뻔했다 (§2.17.2)
#     - 컨텍스트 공유가 오류 없이 100% 틀린 답을 냈다 (discuss.md §9)
#
#   마지막 항목이 특히 중요하다. **성능 측정 전에 정확도부터 확인한다.**
#   틀린 답을 빨리 내는 구성이 벤치마크에서 이기면 안 된다.
#
# 사용법
#   ./scripts/preflight-check.sh                    기본 검사
#   ./scripts/preflight-check.sh --with-inference   추론 정확도까지 (권장)
#   ./scripts/preflight-check.sh --scheduler URL    클러스터 등록 상태까지
#   ./scripts/preflight-check.sh --json out.json    결과를 파일로
#
# 종료 코드
#   0  통과 (경고는 있을 수 있다)
#   1  하드 실패. 이 상태로 측정하면 결과가 무효다
#   2  스크립트 사용 오류

set -uo pipefail

# ── 노드 sudo 비밀번호 ────────────────────────────────────────────────
# 저장소에 비밀번호를 넣지 않는다. 이 프로젝트는 공개 예정이고, 커밋
# 히스토리에 들어가면 지우는 데 히스토리 재작성이 필요하다.
#
#   export NPUFORGE_SUDO_PASS='...'
# 또는
#   printf '%s\n' '...' > ~/.npuforge/sudo-pass && chmod 600 ~/.npuforge/sudo-pass
#
# 노드에 NOPASSWD sudoers 규칙을 두면 비밀번호 없이 동작한다(권장).
SUDO_PASS="${NPUFORGE_SUDO_PASS:-}"
if [ -z "$SUDO_PASS" ] && [ -r "$HOME/.npuforge/sudo-pass" ]; then
    SUDO_PASS="$(head -1 "$HOME/.npuforge/sudo-pass")"
fi

HOSTS=(npuforge-k npuforge-q npuforge-j)
EXPECTED=(king queen jack)

MODEL_DIR=/home/pi/npuforge-rknn-test
MODEL=yolov8n-fp16.rknn
DUMP_TOOL=$MODEL_DIR/dump_output_test

# 유휴 온도 상한. 이전 run 의 열이 남아 있으면 다음 run 의 상승 곡선이
# 달라진다. 실측 유휴는 35~40°C 였다.
IDLE_TEMP_MAX=50
# 입력 전압 하한. 4.983V 로 리셋된 적이 있다. 5V 4A 어댑터에서 5.05V 이상.
VOLT_MIN=5.00

WITH_INFERENCE=0
SCHEDULER=""
JSON_OUT=""

while [ $# -gt 0 ]; do
    case "$1" in
        --with-inference) WITH_INFERENCE=1; shift ;;
        --scheduler) SCHEDULER="${2:-}"; shift 2 ;;
        --json) JSON_OUT="${2:-}"; shift 2 ;;
        --idle-temp-max) IDLE_TEMP_MAX="${2:-}"; shift 2 ;;
        -h|--help) sed -n '2,30p' "$0"; exit 0 ;;
        *) echo "알 수 없는 옵션: $1" >&2; exit 2 ;;
    esac
done

FAILS=0
WARNS=0
declare -a RESULTS

pass() { printf '  \033[32m✓\033[0m %s\n' "$1"; RESULTS+=("PASS|$1|"); }
warn() { printf '  \033[33m!\033[0m %s\n     %s\n' "$1" "${2:-}"; WARNS=$((WARNS+1)); RESULTS+=("WARN|$1|${2:-}"); }
fail() { printf '  \033[31m✗\033[0m %s\n     %s\n' "$1" "${2:-}"; FAILS=$((FAILS+1)); RESULTS+=("FAIL|$1|${2:-}"); }
section() { printf '\n\033[1m%s\033[0m\n' "$1"; }

# 세 노드에서 같은 명령을 돌려 값을 모은다. 실패하면 빈 문자열.
# 출력이 여러 줄이면 첫 줄만 쓴다.
collect() {
    local cmd="$1"
    local -n _out=$2
    _out=()
    local h
    for h in "${HOSTS[@]}"; do
        _out+=("$(ssh -o ConnectTimeout=8 -o BatchMode=yes "$h" "$cmd" 2>/dev/null | head -1)")
    done
}

# 세 값이 모두 같고, 읽기에 성공한 값인지 판정한다.
#
# "못 읽음"을 세 노드에서 똑같이 받고 "일치"로 판정하면 안 된다.
# 그것이야말로 discuss.md §10 의 A 유형(지표가 무엇을 세는지 확인하지
# 않음) 실수다. 값이 비었거나 자리표시자면 실패로 본다.
same_across_nodes() {
    local label="$1"; shift
    local -a v=("$@")
    local i
    for i in 0 1 2; do
        case "${v[$i]}" in
            ""|unknown|Unknown|UNKNOWN|"Permission denied"|N/A)
                fail "$label" "${EXPECTED[$i]} 에서 값을 얻지 못했다 ('${v[$i]}')"
                return 1
                ;;
        esac
    done
    if [ "${v[0]}" = "${v[1]}" ] && [ "${v[1]}" = "${v[2]}" ]; then
        pass "$label — ${v[0]}"
        return 0
    fi
    fail "$label" "king=${v[0]} queen=${v[1]} jack=${v[2]}"
    return 1
}

echo "════════════════════════════════════════════════════════════"
echo " NPUDure preflight — $(date '+%Y-%m-%d %H:%M:%S %Z')"
echo "════════════════════════════════════════════════════════════"

# ── 1. 접속과 정체성 ────────────────────────────────────────────────
# 별칭이 엉뚱한 보드를 가리키면 결과가 잘못된 노드에 귀속된다.
# 연결 실패보다 이쪽이 훨씬 위험하다. worklog §2.20
section "1. 접속과 정체성"

REACHABLE=1
for i in 0 1 2; do
    h="${HOSTS[$i]}"; want="${EXPECTED[$i]}"
    got="$(ssh -o ConnectTimeout=8 -o BatchMode=yes "$h" hostname 2>/dev/null | head -1)"
    if [ -z "$got" ]; then
        fail "$h 접속" "응답 없음. ~/.ssh/config 의 HostName 을 확인하라 (문서의 IP 는 낡을 수 있다)"
        REACHABLE=0
    elif [ "$got" != "$want" ]; then
        fail "$h → hostname" "'$want' 를 기대했는데 '$got' 이다. 별칭이 다른 보드를 가리킨다"
        REACHABLE=0
    else
        pass "$h → $got"
    fi
done

if [ "$REACHABLE" -eq 0 ]; then
    echo ""
    echo "접속 단계에서 실패했다. 이후 검사는 의미가 없다."
    exit 1
fi

# boot_id 는 벤치 도구가 run 유효성 판정에 쓴다. 여기서 기준값을 남긴다.
collect 'cat /proc/sys/kernel/random/boot_id' BOOT_IDS
for i in 0 1 2; do
    printf '     %-6s boot_id %s\n' "${EXPECTED[$i]}" "${BOOT_IDS[$i]:0:8}"
done

# ── 2. 소프트웨어 동일성 ────────────────────────────────────────────
# 하나라도 다르면 노드 간 성능 차이가 하드웨어 차이인지 소프트웨어 차이인지
# 구분할 수 없다.
section "2. 소프트웨어 동일성"

collect 'uname -r' KERNELS
same_across_nodes "커널 버전" "${KERNELS[@]}"

collect 'sha256sum /usr/lib/librknnrt.so 2>/dev/null | cut -c1-16' RKNN_LIBS
same_across_nodes "librknnrt.so 해시" "${RKNN_LIBS[@]}"

# debugfs 는 root 만 읽을 수 있다. 자리표시자로 덮으면 세 노드가
# 똑같이 못 읽은 것을 "일치"로 오판한다.
collect "printf '%s\n' '$SUDO_PASS' | sudo -S -p '' cat /sys/kernel/debug/rknpu/version 2>/dev/null" DRIVERS
same_across_nodes "RKNPU 드라이버" "${DRIVERS[@]}"

collect "sha256sum $MODEL_DIR/$MODEL 2>/dev/null | cut -c1-16" MODELS
same_across_nodes "모델 해시 ($MODEL)" "${MODELS[@]}"

# ── 3. 측정 조건 ────────────────────────────────────────────────────
section "3. 측정 조건"

# CPU governor. ondemand 면 주파수가 부하에 따라 흔들려 재현성이 떨어진다.
# governor 는 "세 노드가 같은가"와 "값이 performance 인가"를 함께 본다.
# ondemand 면 주파수가 부하에 따라 흔들려 재현성이 떨어진다.
collect 'cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null' GOVS
if [ -z "${GOVS[0]}" ] || [ -z "${GOVS[1]}" ] || [ -z "${GOVS[2]}" ]; then
    fail "CPU governor" "값을 읽지 못한 노드가 있다"
elif [ "${GOVS[0]}" != "${GOVS[1]}" ] || [ "${GOVS[1]}" != "${GOVS[2]}" ]; then
    fail "CPU governor" "노드마다 다르다: king=${GOVS[0]} queen=${GOVS[1]} jack=${GOVS[2]}"
elif [ "${GOVS[0]}" != "performance" ]; then
    fail "CPU governor" "세 노드 모두 '${GOVS[0]}' 이다. 측정 전에 performance 로 바꾸라"
else
    pass "CPU governor — performance"
fi

# 유휴 온도. 직전 run 의 열이 남아 있으면 상승 곡선이 달라진다.
collect 'awk "{printf \"%.1f\", \$1/1000}" /sys/class/thermal/thermal_zone4/temp' TEMPS
HOT=""
for i in 0 1 2; do
    t="${TEMPS[$i]}"
    if [ -z "$t" ]; then
        warn "${EXPECTED[$i]} NPU 온도" "읽지 못했다"
    elif awk -v a="$t" -v b="$IDLE_TEMP_MAX" 'BEGIN{exit !(a>b)}'; then
        HOT="$HOT ${EXPECTED[$i]}=${t}°C"
    fi
done
if [ -n "$HOT" ]; then
    fail "유휴 온도" "상한 ${IDLE_TEMP_MAX}°C 초과:$HOT — 식을 때까지 기다려라"
else
    pass "유휴 온도 — king=${TEMPS[0]} queen=${TEMPS[1]} jack=${TEMPS[2]} (°C)"
fi

# 입력 전압. 강하는 리셋의 선행 지표다. worklog §2.17.2
collect 'awk "{printf \"%.3f\", \$1/1000000}" /sys/class/power_supply/simple-vin/voltage_now 2>/dev/null' VOLTS
LOW=""
for i in 0 1 2; do
    v="${VOLTS[$i]}"
    if [ -z "$v" ]; then
        warn "${EXPECTED[$i]} 입력 전압" "센서를 읽지 못했다"
    elif awk -v a="$v" -v b="$VOLT_MIN" 'BEGIN{exit !(a<b)}'; then
        LOW="$LOW ${EXPECTED[$i]}=${v}V"
    fi
done
if [ -n "$LOW" ]; then
    fail "입력 전압" "하한 ${VOLT_MIN}V 미만:$LOW — 어댑터 용량을 확인하라"
elif [ -n "${VOLTS[0]}" ]; then
    pass "입력 전압 — king=${VOLTS[0]} queen=${VOLTS[1]} jack=${VOLTS[2]} (V)"
fi

# 남은 부하 프로세스. 이전 실험이 아직 돌고 있으면 측정이 오염된다.
#
# `pgrep -f` 는 쓰지 않는다. 명령줄을 매칭하므로 **자기 자신을 실행한
# 래퍼 셸까지 매치한다.** ssh 가 보내는 `bash -c "...pgrep -f X..."` 의
# 명령줄에 패턴 문자열이 들어 있기 때문이다. 대괄호 트릭
# (`[s]ustained`)도 같은 명령줄에 다른 형태가 섞이면 무력해진다.
#
# 실제로 양방향으로 틀렸다.
#   - 부하가 도는데 0 을 보고 (false negative)
#   - 부하가 없는데 2 를 보고 (false positive, 자기 셸을 셈)
#
# `/proc/PID/exe` 의 심볼릭 링크를 본다. 이것은 실제 실행 파일을
# 가리키므로 셸이 끼어들 여지가 없다.
#
# `npuforge-node` 는 여기서 제외한다. M3 부터 노드는 상시 실행되는
# 서비스라 프로세스가 떠 있는 것이 정상이다. 노드가 이전 측정의 부하를
# 아직 처리 중인지는 프로세스 존재가 아니라 **유휴 온도**로 판정한다
# (idle 이면 위 3번 검사에서 35~40°C). 측정 C 도구(sustained 등)만
# 잔존 부하로 본다.
collect 'n=0; for p in /proc/[0-9]*; do
  case "$(readlink -f $p/exe 2>/dev/null)" in
    *sustained_load_test|*thread_safety_test|*npu_occupancy_test|*zerocopy_test|*shared_context_test)
      n=$((n+1)) ;;
  esac
done; echo $n' LOADS
LEFTOVER=""
for i in 0 1 2; do
    [ "${LOADS[$i]:-0}" != "0" ] && LEFTOVER="$LEFTOVER ${EXPECTED[$i]}(${LOADS[$i]}개)"
done
if [ -n "$LEFTOVER" ]; then
    fail "남은 부하 프로세스" "$LEFTOVER — 종료 후 온도가 식기를 기다려라"
else
    pass "남은 부하 프로세스 없음"
fi

# 시각 동기화. 노드 간 절대 시각을 직접 비교하지는 않지만, 로그 대조에 필요하다.
collect 'timedatectl show -p NTPSynchronized --value 2>/dev/null' NTP
NOSYNC=""
for i in 0 1 2; do
    [ "${NTP[$i]}" != "yes" ] && NOSYNC="$NOSYNC ${EXPECTED[$i]}"
done
if [ -n "$NOSYNC" ]; then
    warn "NTP 동기화" "미동기:$NOSYNC — 로그 시각 대조가 어긋날 수 있다"
else
    pass "NTP 동기화"
fi

# 데스크톱 세션. 한 대에만 GUI 가 떠 있으면 그 보드만 CPU·발열이 다르다.
collect 'who | wc -l' SESSIONS
if [ "${SESSIONS[0]}" = "${SESSIONS[1]}" ] && [ "${SESSIONS[1]}" = "${SESSIONS[2]}" ]; then
    pass "세션 수 동일 (${SESSIONS[0]})"
else
    warn "세션 수 불일치" "king=${SESSIONS[0]} queen=${SESSIONS[1]} jack=${SESSIONS[2]} — GUI 작업 중인 보드가 있으면 측정 오염원이다"
fi

# ── 4. 추론 정확도 (성능보다 먼저) ──────────────────────────────────
# discuss.md §9: 컨텍스트 공유가 API 오류 0건으로 100% 틀린 답을 냈다.
# 틀린 답을 빨리 내는 구성이 벤치마크에서 이기면 안 된다.
if [ "$WITH_INFERENCE" -eq 1 ]; then
    section "4. 추론 정확도 (성능보다 먼저 본다)"

    HAVE_TOOL=1
    for i in 0 1 2; do
        if ! ssh -o BatchMode=yes "${HOSTS[$i]}" "test -x $DUMP_TOOL" 2>/dev/null; then
            warn "${EXPECTED[$i]} dump_output_test" "없다. crates/npuforge-rknn/native/ 에서 빌드해 배포하라"
            HAVE_TOOL=0
        fi
    done

    if [ "$HAVE_TOOL" -eq 1 ]; then
        # 같은 입력을 세 보드에 넣고 출력 해시를 비교한다.
        # 하나라도 다르면 모델·드라이버·양자화 중 무언가가 다르다.
        HASHES=()
        for i in 0 1 2; do
            h="${HOSTS[$i]}"
            out=$(ssh -o BatchMode=yes "$h" "
                set -e
                cd $MODEL_DIR
                # 결정적 입력. 모델 입력 크기와 정확히 같아야 한다.
                python3 -c \"
import sys
sys.stdout.buffer.write(bytes((i*7+13)%251 for i in range(640*640*3)))
\" > /tmp/preflight-in.bin
                $DUMP_TOOL $MODEL /tmp/preflight-in.bin /tmp/preflight-a.bin 1 >/dev/null 2>&1
                $DUMP_TOOL $MODEL /tmp/preflight-in.bin /tmp/preflight-b.bin 1 >/dev/null 2>&1
                # 같은 입력 두 번 → 같은 출력이어야 한다 (결정성)
                if ! cmp -s /tmp/preflight-a.bin /tmp/preflight-b.bin; then
                    echo NONDETERMINISTIC
                    exit 0
                fi
                sha256sum /tmp/preflight-a.bin | cut -c1-16
            " 2>/dev/null | tail -1)
            HASHES+=("$out")
        done

        NONDET=""
        for i in 0 1 2; do
            [ "${HASHES[$i]}" = "NONDETERMINISTIC" ] && NONDET="$NONDET ${EXPECTED[$i]}"
        done

        if [ -n "$NONDET" ]; then
            fail "추론 결정성" "같은 입력이 다른 출력을 냈다:$NONDET"
        elif same_across_nodes "추론 출력 일치 (3노드 동일 입력)" "${HASHES[@]}"; then
            :
        else
            printf '     세 보드가 같은 입력에 다른 답을 낸다. 모델·드라이버·양자화를 확인하라.\n'
        fi
    fi
else
    section "4. 추론 정확도 — 건너뜀"
    echo "     --with-inference 로 실행하면 세 보드가 같은 입력에 같은 답을"
    echo "     내는지 확인한다. discuss.md §9 참조. 정식 측정 전에는 반드시 하라."
fi

# ── 5. 네트워크 실측 기록 (M3 전제) ─────────────────────────────────
# 계산만 믿지 않는다. 입력만 계산하고 출력을 안 본 것이 이미 한 번
# 오류였다. 02-HARDWARE-SETUP.md §3.3.3
section "5. 네트워크 링크 속도"

# 스케줄러 쪽 링크. 세 노드 트래픽이 합류하는 지점이라 여기가 먼저 막힌다.
if [ -n "${NPUFORGE_SCHED_HOST:-}" ]; then
    sched_if=$(ssh -o BatchMode=yes "$NPUFORGE_SCHED_HOST"         "ip -o -4 route show default | awk '{print \$5}' | head -1" 2>/dev/null)
    sched_speed=$(ssh -o BatchMode=yes "$NPUFORGE_SCHED_HOST"         "cat /sys/class/net/$sched_if/speed 2>/dev/null" 2>/dev/null)
    if [ -z "$sched_speed" ] || [ "$sched_speed" -le 0 ] 2>/dev/null; then
        warn "스케줄러 링크 속도" "$NPUFORGE_SCHED_HOST:$sched_if 에서 읽지 못했다"
    elif [ "$sched_speed" -lt 10000 ]; then
        # INT8 3노드 입력만 4.64 Gbps, 출력은 want_float 에 따라 최대 18.4 Gbps.
        fail "스케줄러 링크 속도"             "$sched_if ${sched_speed}Mb/s. 3노드 포화에 10G 가 필요하다 (§3.3.2)"
    else
        pass "스케줄러 링크 — $sched_if ${sched_speed}Mb/s"
    fi
else
    echo "     NPUFORGE_SCHED_HOST 를 지정하면 스케줄러 링크 속도를 확인한다."
fi

# 노드 쪽 링크. 2.5G 로 붙어야 한다.
collect 'ip -o -4 route show default | awk "{print \$5}" | head -1' NODE_IFS
SLOW=""
for i in 0 1 2; do
    ifc="${NODE_IFS[$i]}"
    [ -z "$ifc" ] && continue
    sp=$(ssh -o BatchMode=yes "${HOSTS[$i]}" "cat /sys/class/net/$ifc/speed 2>/dev/null" 2>/dev/null)
    if [ -z "$sp" ] || [ "$sp" -le 0 ] 2>/dev/null; then
        warn "${EXPECTED[$i]} 링크 속도" "$ifc 에서 읽지 못했다"
    elif [ "$sp" -lt 2500 ]; then
        SLOW="$SLOW ${EXPECTED[$i]}($ifc ${sp}Mb/s)"
    fi
done
if [ -n "$SLOW" ]; then
    warn "노드 링크 속도" "2.5G 미만:$SLOW — 추론망에 연결했는지 확인하라"
else
    pass "노드 링크 속도 — 3노드 모두 2.5G 이상"
fi

# ── 6. 클러스터 등록 상태 ───────────────────────────────────────────
if [ -n "$SCHEDULER" ]; then
    section "6. 클러스터 등록 상태"
    if command -v grpcurl >/dev/null 2>&1; then
        n=$(grpcurl -plaintext "${SCHEDULER#http://}" npuforge.v1.SchedulerService/ListNodes 2>/dev/null \
            | grep -c '"nodeId"' || echo 0)
        if [ "$n" -eq 3 ]; then
            pass "스케줄러에 3노드 등록됨"
        else
            fail "스케줄러 등록" "${n}개만 등록되어 있다 (3개 기대)"
        fi
    else
        warn "클러스터 확인" "grpcurl 이 없다. npuforge-bench 가 시작 시 다시 확인한다"
    fi
fi

# ── 결과 ────────────────────────────────────────────────────────────
echo ""
echo "════════════════════════════════════════════════════════════"
if [ "$FAILS" -gt 0 ]; then
    printf ' \033[31m실패 %d건\033[0m, 경고 %d건 — \033[1m측정을 시작하지 마라\033[0m\n' "$FAILS" "$WARNS"
else
    printf ' \033[32m통과\033[0m, 경고 %d건\n' "$WARNS"
fi
echo "════════════════════════════════════════════════════════════"

if [ -n "$JSON_OUT" ]; then
    {
        printf '{\n'
        printf '  "checked_at": "%s",\n' "$(date -Iseconds)"
        printf '  "failures": %d,\n' "$FAILS"
        printf '  "warnings": %d,\n' "$WARNS"
        printf '  "boot_ids": {"king": "%s", "queen": "%s", "jack": "%s"},\n' \
            "${BOOT_IDS[0]}" "${BOOT_IDS[1]}" "${BOOT_IDS[2]}"
        printf '  "checks": [\n'
        first=1
        for r in "${RESULTS[@]}"; do
            IFS='|' read -r status label detail <<< "$r"
            [ $first -eq 0 ] && printf ',\n'
            first=0
            printf '    {"status": "%s", "check": "%s", "detail": "%s"}' \
                "$status" "${label//\"/\\\"}" "${detail//\"/\\\"}"
        done
        printf '\n  ]\n}\n'
    } > "$JSON_OUT"
    echo "결과 저장: $JSON_OUT"
fi

[ "$FAILS" -eq 0 ] || exit 1
exit 0
