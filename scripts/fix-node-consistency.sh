#!/usr/bin/env bash
#
# 세 노드의 상태를 일치시킨다.
#
# docs/board-worklog.md §2.5 에 기록된 불일치를 해소한다.
#   1. Ubuntu 패치 레벨 차이 (king 만 24.04.3)
#   2. 미적용 패키지 업데이트 279~374개
#   3. SSH 호스트 키가 3대 동일
#   4. 기본 패키지 미설치 (ethtool, curl, chrony, iperf3 등)
#   5. CPU Governor = ondemand
#
# ────────────────────────────────────────────────────────────────
#  ⚠️  커널을 절대 업그레이드하지 않는다.
#
#  커널 6.1.141 은 FriendlyElec BSP 이며 RKNPU 드라이버 v0.9.8 이
#  여기에 묶여 있다. 커널이 교체되면 NPU 가 동작하지 않을 수 있다.
#  이 스크립트는 apt 작업 전에 커널 패키지를 hold 한다.
# ────────────────────────────────────────────────────────────────
#
# 기본은 DRY RUN 이다. 실제 적용하려면 --apply 를 준다.
#
#   ./scripts/fix-node-consistency.sh                 # 무엇을 할지만 출력
#   ./scripts/fix-node-consistency.sh --apply         # 실제 실행
#   ./scripts/fix-node-consistency.sh --apply --only hostkeys,packages
#
# 단계별로 나눠 실행하는 것을 권한다. 특히 upgrade 는 마지막에 단독으로.

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

NODES=("npuforge-k:king" "npuforge-q:queen" "npuforge-j:jack")
SSH_OPTS=(-o BatchMode=yes -o ConnectTimeout=10)

# HARDWARE-SETUP §5.2 기본 패키지 + 프로파일링 도구
PACKAGES="build-essential pkg-config cmake git curl wget chrony iperf3 ethtool jq htop sysstat linux-tools-common"

APPLY=0
ONLY=""
for arg in "$@"; do
    case "$arg" in
        --apply) APPLY=1 ;;
        --only) shift; ONLY="${1:-}" ;;
        --only=*) ONLY="${arg#*=}" ;;
        -h|--help) sed -n '2,32p' "$0"; exit 0 ;;
    esac
done

want() {
    [ -z "$ONLY" ] && return 0
    case ",$ONLY," in *",$1,"*) return 0 ;; *) return 1 ;; esac
}

run_on() {
    local host="$1"; shift
    if [ "$APPLY" -eq 1 ]; then
        ssh "${SSH_OPTS[@]}" "$host" "bash -s" <<< "$*"
    else
        echo "    [DRY RUN] $host <<< $(echo "$*" | head -1 | cut -c1-90)..."
    fi
}

banner() {
    echo ""
    echo "════════════════════════════════════════════════════════════"
    echo " $1"
    echo "════════════════════════════════════════════════════════════"
}

if [ "$APPLY" -eq 0 ]; then
    echo "*** DRY RUN 모드입니다. 실제 변경은 일어나지 않습니다. ***"
    echo "*** 적용하려면 --apply 를 붙이세요. ***"
fi

# ───────────────────────────────────────────────────────────────
# 0. 사전 점검
# ───────────────────────────────────────────────────────────────
banner "0. 사전 점검"
for entry in "${NODES[@]}"; do
    host="${entry%%:*}"; name="${entry##*:}"
    printf "  %-12s " "$name"
    if ssh "${SSH_OPTS[@]}" "$host" 'true' 2>/dev/null; then
        info=$(ssh "${SSH_OPTS[@]}" "$host" \
            'echo "$(hostname) kernel=$(uname -r) $(grep -oP "(?<=^VERSION=\").*(?=\")" /etc/os-release)"' 2>/dev/null)
        echo "OK  $info"
    else
        echo "접속 실패 — 중단"
        exit 1
    fi
done

# ───────────────────────────────────────────────────────────────
# 1. 커널 hold  (반드시 apt 작업보다 먼저)
# ───────────────────────────────────────────────────────────────
if want kernelhold; then
banner "1. 커널 패키지 hold"
echo "  RKNPU 드라이버가 BSP 커널에 묶여 있으므로 커널 교체를 막는다."
for entry in "${NODES[@]}"; do
    host="${entry%%:*}"; name="${entry##*:}"
    echo "  --- $name ---"
    run_on "$host" '
S() { printf '%s\n' "$SUDO_PASS" | sudo -S -p "" "$@" 2>/dev/null; }
PKGS=$(dpkg -l | awk "/^ii/ && (\$2 ~ /^linux-(image|headers|modules|dtb)/) {print \$2}")
if [ -n "$PKGS" ]; then
    S apt-mark hold $PKGS >/dev/null
    echo "    hold: $(echo $PKGS | tr "\n" " ")"
else
    echo "    dpkg 관리 커널 패키지 없음 (BSP 커널이 파일로 설치된 형태)"
fi
echo "    현재 커널: $(uname -r)"
'
done
fi

# ───────────────────────────────────────────────────────────────
# 2. SSH 호스트 키 재생성
# ───────────────────────────────────────────────────────────────
if want hostkeys; then
banner "2. SSH 호스트 키 재생성"
echo "  현재 3대가 동일한 호스트 키를 쓰고 있어 노드 식별이 불가능하다."
echo "  재생성 후 PC 의 known_hosts 에서 기존 항목을 제거해야 한다."
for entry in "${NODES[@]}"; do
    host="${entry%%:*}"; name="${entry##*:}"
    echo "  --- $name ---"
    run_on "$host" '
S() { printf '%s\n' "$SUDO_PASS" | sudo -S -p "" "$@" 2>/dev/null; }
OLD=$(ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub | awk "{print \$2}")
S rm -f /etc/ssh/ssh_host_*_key /etc/ssh/ssh_host_*_key.pub
S ssh-keygen -A >/dev/null 2>&1
S systemctl restart ssh
NEW=$(ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub | awk "{print \$2}")
echo "    old=$OLD"
echo "    new=$NEW"
'
done
if [ "$APPLY" -eq 1 ]; then
    echo "  --- PC known_hosts 정리 ---"
    for ip in 192.168.123.12 192.168.123.16 192.168.123.33; do
        ssh-keygen -R "$ip" >/dev/null 2>&1 && echo "    제거: $ip"
    done
    for h in npuforge-k npuforge-q npuforge-j; do
        ssh-keygen -R "$h" >/dev/null 2>&1
    done
    echo "    다음 접속 시 새 호스트 키를 accept-new 로 등록한다."
fi
fi

# ───────────────────────────────────────────────────────────────
# 3. 기본 패키지 설치
# ───────────────────────────────────────────────────────────────
if want packages; then
banner "3. 기본 패키지 설치"
echo "  대상: $PACKAGES"
for entry in "${NODES[@]}"; do
    host="${entry%%:*}"; name="${entry##*:}"
    echo "  --- $name ---"
    run_on "$host" "
S() { printf '%s\n' \"\$SUDO_PASS\" | sudo -S -p '' \"\$@\" 2>/dev/null; }
export DEBIAN_FRONTEND=noninteractive
S apt-get update -qq
S apt-get install -y -qq $PACKAGES 2>&1 | tail -2
echo \"    설치 확인: \$(for c in curl ethtool iperf3 chronyc jq; do command -v \$c >/dev/null && printf '%s ' \$c; done)\"
"
done
fi

# ───────────────────────────────────────────────────────────────
# 4. 시간 동기화
# ───────────────────────────────────────────────────────────────
if want chrony; then
banner "4. chrony 활성화"
echo "  구조화 로그의 사건 순서를 맞추기 위한 것이다."
echo "  (서로 다른 장치의 monotonic clock 을 직접 비교하지는 않는다)"
for entry in "${NODES[@]}"; do
    host="${entry%%:*}"; name="${entry##*:}"
    echo "  --- $name ---"
    run_on "$host" '
S() { printf '%s\n' "$SUDO_PASS" | sudo -S -p "" "$@" 2>/dev/null; }
S systemctl enable --now chrony >/dev/null 2>&1
sleep 1
echo "    $(chronyc tracking 2>/dev/null | grep -E "Reference ID|System time" | tr "\n" " ")"
'
done
fi

# ───────────────────────────────────────────────────────────────
# 5. 패키지 업그레이드   ⚠️ 되돌리기 어려움
# ───────────────────────────────────────────────────────────────
if want upgrade; then
banner "5. 패키지 업그레이드  ⚠️  주의"
echo "  king 만 24.04.3 이고 나머지는 24.04.4 다. 세 노드를 맞춘다."
echo "  커널은 1단계에서 hold 되어 있어야 한다."
echo ""
if [ "$APPLY" -eq 1 ]; then
    echo "  hold 상태 확인 중..."
    for entry in "${NODES[@]}"; do
        host="${entry%%:*}"; name="${entry##*:}"
        held=$(ssh "${SSH_OPTS[@]}" "$host" 'apt-mark showhold 2>/dev/null | wc -l')
        kern=$(ssh "${SSH_OPTS[@]}" "$host" 'dpkg -l 2>/dev/null | grep -c "^ii.*linux-image"')
        echo "    $name: hold=$held개, dpkg커널패키지=$kern개"
        if [ "$kern" -gt 0 ] && [ "$held" -eq 0 ]; then
            echo "    ✗ 커널이 hold 되지 않았다. 1단계를 먼저 실행하라. 중단."
            exit 1
        fi
    done
fi
for entry in "${NODES[@]}"; do
    host="${entry%%:*}"; name="${entry##*:}"
    echo "  --- $name ---"
    run_on "$host" '
S() { printf '%s\n' "$SUDO_PASS" | sudo -S -p "" "$@" 2>/dev/null; }
export DEBIAN_FRONTEND=noninteractive
BEFORE=$(grep -oP "(?<=^VERSION=\").*(?=\")" /etc/os-release)
S apt-get update -qq
S apt-get upgrade -y -qq 2>&1 | tail -3
AFTER=$(grep -oP "(?<=^VERSION=\").*(?=\")" /etc/os-release)
echo "    $BEFORE -> $AFTER"
echo "    커널: $(uname -r) (재부팅 필요 여부: $([ -f /var/run/reboot-required ] && echo YES || echo no))"
'
done
echo ""
echo "  업그레이드 후 반드시 확인할 것:"
echo "    - uname -r 이 6.1.141 그대로인가"
echo "    - /sys/kernel/debug/rknpu/version 이 여전히 읽히는가"
echo "    - librknnrt.so SHA-256 이 변하지 않았는가"
fi

# ───────────────────────────────────────────────────────────────
# 6. CPU Governor
# ───────────────────────────────────────────────────────────────
if want governor; then
banner "6. CPU Governor -> performance"
echo "  ondemand 는 부하에 따라 주파수가 변해 측정 재현성을 떨어뜨린다."
echo "  주의: 소비전력과 발열이 올라간다. 팬리스 보드이므로 S0 열 특성"
echo "        측정 결과를 확인한 뒤 적용하는 것을 권한다."
for entry in "${NODES[@]}"; do
    host="${entry%%:*}"; name="${entry##*:}"
    echo "  --- $name ---"
    run_on "$host" '
S() { printf '%s\n' "$SUDO_PASS" | sudo -S -p "" "$@" 2>/dev/null; }
for g in /sys/devices/system/cpu/cpufreq/policy*/scaling_governor; do
    printf "performance\n" | S tee "$g" >/dev/null 2>&1
done
echo "    governor: $(cat /sys/devices/system/cpu/cpufreq/policy*/scaling_governor | tr "\n" " ")"
echo "    freq:     $(cat /sys/devices/system/cpu/cpufreq/policy*/scaling_cur_freq | tr "\n" " ")"
'
done
echo "  (재부팅 시 초기화된다. 영구 적용은 systemd 서비스나 cpufrequtils 로 별도 구성)"
fi

# ───────────────────────────────────────────────────────────────
# 7. 검증
# ───────────────────────────────────────────────────────────────
banner "7. 최종 검증"
if [ "$APPLY" -eq 1 ]; then
    echo "  scripts/collect-node-info.sh 로 재수집해 diff 하십시오:"
    echo ""
    echo "    for p in k:npuforge-k q:npuforge-q j:npuforge-j; do"
    echo "      n=\${p%%:*}; h=\${p##*:}"
    echo "      ssh \$h 'bash -s' < scripts/collect-node-info.sh > benchmarks/node-info/\$n.after.txt"
    echo "    done"
    echo ""
    echo "  RKNN 이 살아 있는지 반드시 확인:"
    echo "    ssh npuforge-k 'printf \"\$NPUFORGE_SUDO_PASS\\n\" | sudo -S cat /sys/kernel/debug/rknpu/version'"
else
    echo "  (DRY RUN 이므로 검증 단계 생략)"
    echo ""
    echo "  권장 실행 순서:"
    echo "    1) ./scripts/fix-node-consistency.sh --apply --only kernelhold"
    echo "    2) ./scripts/fix-node-consistency.sh --apply --only hostkeys"
    echo "    3) ./scripts/fix-node-consistency.sh --apply --only packages,chrony"
    echo "    4) ./scripts/fix-node-consistency.sh --apply --only upgrade      # 단독 실행"
    echo "    5) (S0 열 특성 측정 후) --only governor"
fi
echo ""
