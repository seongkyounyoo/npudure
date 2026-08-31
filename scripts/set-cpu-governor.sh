#!/usr/bin/env bash
# 세 노드의 CPU governor 를 설정하고 재부팅에도 유지되게 한다.
#
# 왜 필요한가
#   `ondemand` 는 부하에 따라 주파수를 바꾼다. 벤치마크에서는 두 가지가
#   문제다.
#
#     1. 같은 조건을 반복해도 주파수 궤적이 달라 재현성이 떨어진다
#     2. 부하 시작 직후 주파수가 올라가는 구간이 지연에 섞인다
#
#   `performance` 는 항상 최대 주파수를 유지한다. 대가로 유휴 소비전력과
#   발열이 늘지만, 팬리스 열 특성(S0)도 그 조건에서 측정해야 한다.
#
# 재부팅 유지가 왜 중요한가
#   보드는 실제로 리셋된 적이 있다(worklog §2.17.2). governor 가 원래대로
#   돌아간 채 다음 run 이 돌면 조건이 달라진다. preflight 가 잡아 주지만,
#   야간 무인 실행에서는 그 시점에 이미 몇 시간을 버린 뒤다.
#
# 사용법
#   ./scripts/set-cpu-governor.sh                 현재 상태만 표시
#   ./scripts/set-cpu-governor.sh --apply         performance 로 설정 + 영구화
#   ./scripts/set-cpu-governor.sh --apply ondemand   원래대로 되돌리기

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
GOV="${2:-performance}"
APPLY=0
[ "${1:-}" = "--apply" ] && APPLY=1

UNIT=/etc/systemd/system/npuforge-cpu-governor.service
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TMP_UNIT="$(mktemp)"
trap 'rm -f "$TMP_UNIT"' EXIT

show() {
    local h="$1"
    ssh -o BatchMode=yes "$h" '
        printf "  %-7s " "$(hostname)"
        for p in /sys/devices/system/cpu/cpufreq/policy*; do
            printf "%s=%s(%sMHz) " "$(basename $p)" \
                "$(cat $p/scaling_governor)" "$(( $(cat $p/scaling_cur_freq)/1000 ))"
        done
        if systemctl is-enabled npuforge-cpu-governor.service >/dev/null 2>&1; then
            printf " [영구화됨]"
        else
            printf " [영구화 안 됨]"
        fi
        printf "\n"
    ' 2>/dev/null
}

echo "════════════════════════════════════════════════════════════"
echo " CPU governor — $(date '+%Y-%m-%d %H:%M:%S %Z')"
echo "════════════════════════════════════════════════════════════"

echo ""
echo "현재 상태"
for h in "${HOSTS[@]}"; do show "$h"; done

if [ "$APPLY" -eq 0 ]; then
    echo ""
    echo "적용하려면 --apply 를 붙이세요."
    exit 0
fi

case "$GOV" in
    performance|ondemand|schedutil|powersave|conservative) ;;
    *) echo "지원하지 않는 governor: $GOV" >&2; exit 2 ;;
esac

echo ""
echo "'$GOV' 로 설정하고 영구화합니다."

for h in "${HOSTS[@]}"; do
    name=$(ssh -o BatchMode=yes "$h" hostname 2>/dev/null)
    echo ""
    echo "  --- $name ---"

    # systemd 유닛으로 영구화한다. cpufrequtils 를 설치하면 세 노드의
    # 패키지 목록이 달라지므로 쓰지 않는다 — 환경 일치가 깨진다.
    #
    # 유닛 파일은 로컬에서 만들어 전송한다. ssh 안에서 heredoc 과 sudo 를
    # 중첩하면 따옴표가 조용히 깨져 파일이 아예 생기지 않는다.
    # (실제로 그렇게 실패했다. 종료 코드는 0 이었다)
    sed "s|@GOVERNOR@|$GOV|" "$SCRIPT_DIR/npuforge-cpu-governor.service.in" > "$TMP_UNIT"
    scp -q "$TMP_UNIT" "$h:/tmp/npuforge-cpu-governor.service" || {
        echo "    !! 유닛 파일 전송 실패"
        continue
    }
    ssh -o BatchMode=yes "$h" "
        printf '%s
' \"\$SUDO_PASS\" | sudo -S -p '' install -m 644 -o root -g root             /tmp/npuforge-cpu-governor.service $UNIT
        printf '%s
' \"\$SUDO_PASS\" | sudo -S -p '' systemctl daemon-reload
        printf '%s
' \"\$SUDO_PASS\" | sudo -S -p '' systemctl enable --now npuforge-cpu-governor.service
        rm -f /tmp/npuforge-cpu-governor.service
    " >/dev/null 2>&1

    # 유닛이 실제로 적용했는지 확인한다. enable 성공과 값 적용은 다르다.
    got=$(ssh -o BatchMode=yes "$h" 'cat /sys/devices/system/cpu/cpufreq/policy0/scaling_governor' 2>/dev/null)
    got4=$(ssh -o BatchMode=yes "$h" 'cat /sys/devices/system/cpu/cpufreq/policy4/scaling_governor' 2>/dev/null)
    if [ "$got" = "$GOV" ] && [ "$got4" = "$GOV" ]; then
        echo "    적용됨 (policy0, policy4 모두 $GOV)"
    else
        echo "    !! 적용 실패: policy0=$got policy4=$got4"
    fi
done

echo ""
echo "변경 후 상태"
for h in "${HOSTS[@]}"; do show "$h"; done

echo ""
echo "재부팅 후에도 유지되는지는 실제로 재부팅해 확인해야 한다."
echo "preflight-check.sh 가 매 측정 전에 값을 확인한다."
