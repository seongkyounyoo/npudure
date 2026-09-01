#!/usr/bin/env bash
#
# 새로 설치한 노드를 NPUDure 클러스터에 편입시킨다.
#
# OS 재설치 직후 실행한다. 다음을 수행한다.
#   1. SSH 키 설치 (비밀번호 인증은 이후 불필요)
#   2. hostname 설정 및 /etc/hosts 정리
#   3. SSH 호스트 키 재생성 (이미지 복제 시 중복 방지)
#   4. 기본 패키지 설치
#   5. chrony 활성화
#   6. 기준 노드와 환경 비교
#
# 사용법:
#   ./scripts/setup-node.sh <ip> <hostname> [ssh_alias]
#
# 예:
#   ./scripts/setup-node.sh 192.168.123.12 king npuforge-k
#
# 전제: 대상 노드가 FriendlyElec 기본 이미지 상태 (계정 pi/pi)

set -uo pipefail

IP="${1:-}"
NEWHOST="${2:-}"
ALIAS="${3:-npuforge-${NEWHOST}}"

# 기준 노드. 이 노드와 환경을 비교한다.
REFERENCE="${REFERENCE:-npuforge-q}"

DEFAULT_USER="${NPUFORGE_NODE_USER:-pi}"
DEFAULT_PASS="${NPUFORGE_NODE_PASS:-${NPUFORGE_SUDO_PASS:-}}"
KEY="$HOME/.ssh/id_ed25519_npuforge"

PACKAGES="build-essential pkg-config cmake git curl wget chrony iperf3 ethtool jq htop sysstat"

if [ -z "$IP" ] || [ -z "$NEWHOST" ]; then
    sed -n '2,20p' "$0"
    exit 2
fi

say() { printf '\n\033[1m== %s ==\033[0m\n' "$1"; }
ssh_key() { ssh -o BatchMode=yes -o ConnectTimeout=10 "$@"; }

# ---------------------------------------------------------------
# 0. 사전 확인
# ---------------------------------------------------------------
say "0. 사전 확인"

if [ ! -f "$KEY" ]; then
    echo "SSH 키가 없습니다: $KEY"
    echo "다음으로 생성하세요:"
    echo "  ssh-keygen -t ed25519 -f $KEY -N '' -C npuforge-automation"
    exit 1
fi

if ! timeout 5 bash -c "echo > /dev/tcp/$IP/22" 2>/dev/null; then
    echo "$IP 의 tcp/22 에 접속할 수 없습니다. 부팅과 네트워크를 확인하세요."
    exit 1
fi
echo "  $IP tcp/22 응답 확인"

# 이전 호스트 키 제거. 재설치했으므로 반드시 바뀌어 있다.
ssh-keygen -R "$IP" >/dev/null 2>&1 || true
ssh-keygen -R "$ALIAS" >/dev/null 2>&1 || true
echo "  known_hosts 에서 이전 항목 제거"

# ---------------------------------------------------------------
# 1. SSH 키 설치
# ---------------------------------------------------------------
say "1. SSH 키 설치"

if ssh_key -i "$KEY" "${DEFAULT_USER}@${IP}" true 2>/dev/null; then
    echo "  이미 키 인증이 동작합니다"
else
    # SSH 는 비밀번호를 stdin 이 아니라 제어 터미널에서 읽는다.
    # TTY 가 없는 자동화에서는 SSH_ASKPASS_REQUIRE=force 로 우회한다.
    ASKPASS=$(mktemp)
    chmod 700 "$ASKPASS"
    cat > "$ASKPASS" <<HELPER
#!/bin/sh
cat <<'PWEOF'
${DEFAULT_PASS}
PWEOF
HELPER
    trap 'rm -f "$ASKPASS"' EXIT

    SSH_ASKPASS="$ASKPASS" SSH_ASKPASS_REQUIRE=force DISPLAY=dummy \
        ssh-copy-id -o StrictHostKeyChecking=accept-new -o NumberOfPasswordPrompts=1 \
                    -i "${KEY}.pub" "${DEFAULT_USER}@${IP}" >/dev/null 2>&1

    rm -f "$ASKPASS"
    trap - EXIT

    if ssh_key -i "$KEY" "${DEFAULT_USER}@${IP}" true 2>/dev/null; then
        echo "  키 설치 완료"
    else
        echo "  키 설치 실패. 계정/비밀번호를 확인하세요 (${DEFAULT_USER}/${DEFAULT_PASS})"
        exit 1
    fi
fi

# ---------------------------------------------------------------
# 2. ~/.ssh/config 별칭 등록
# ---------------------------------------------------------------
say "2. SSH 별칭 등록"

if grep -q "^Host ${ALIAS}\$" "$HOME/.ssh/config" 2>/dev/null; then
    echo "  ${ALIAS} 항목이 이미 있습니다 (HostName 확인 필요)"
else
    cat >> "$HOME/.ssh/config" <<EOF

Host ${ALIAS}
    HostName ${IP}
    User ${DEFAULT_USER}
    IdentityFile ${KEY}
    StrictHostKeyChecking accept-new
EOF
    echo "  ${ALIAS} -> ${DEFAULT_USER}@${IP} 등록"
fi

# 이후 모든 작업은 별칭으로 수행한다.
R() { ssh_key "$ALIAS" "bash -s"; }

# ---------------------------------------------------------------
# 3. 원격 설정
# ---------------------------------------------------------------
say "3. hostname, 호스트 키, 패키지, chrony"

R <<REMOTE
set -uo pipefail
S() { printf '%s\n' '${DEFAULT_PASS}' | sudo -S -p "" "\$@" 2>/dev/null; }

echo "--- hostname ---"
OLD=\$(hostname)
S hostnamectl set-hostname '${NEWHOST}'
# /etc/hosts 를 임시 파일 경유로 쓴다.
# sudo -S 가 stdin 첫 줄을 소비하므로 파이프로 파일 내용을 넘길 수 없다.
cat > /tmp/npf_hosts <<HOSTS
127.0.0.1	localhost
::1		localhost ip6-localhost ip6-loopback
ff02::1		ip6-allnodes
ff02::2		ip6-allrouters

127.0.1.1	${NEWHOST}
HOSTS
S cp /tmp/npf_hosts /etc/hosts
rm -f /tmp/npf_hosts
echo "  \$OLD -> \$(hostname)"

echo "--- SSH 호스트 키 재생성 ---"
# 이미지를 복제하면 호스트 키가 중복되어 노드를 구분할 수 없다.
BEFORE=\$(ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub 2>/dev/null | awk '{print \$2}')
S rm -f /etc/ssh/ssh_host_*_key /etc/ssh/ssh_host_*_key.pub
S ssh-keygen -A >/dev/null 2>&1
S systemctl restart ssh
AFTER=\$(ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub 2>/dev/null | awk '{print \$2}')
echo "  before: \$BEFORE"
echo "  after : \$AFTER"

echo "--- 커널 hold (RKNPU 드라이버 보호) ---"
PKGS=\$(dpkg -l | awk '/^ii/ && (\$2 ~ /^linux-(image|headers|modules|dtb)/) {print \$2}')
if [ -n "\$PKGS" ]; then
    S apt-mark hold \$PKGS >/dev/null
    echo "  hold: \$(echo \$PKGS | tr '\n' ' ')"
else
    echo "  dpkg 관리 커널 패키지 없음 (BSP 커널)"
fi

echo "--- 기본 패키지 ---"
export DEBIAN_FRONTEND=noninteractive
S apt-get update -qq
S apt-get install -y -qq ${PACKAGES} 2>&1 | tail -2
echo "  설치 확인: \$(for c in curl ethtool iperf3 chronyc jq gcc; do command -v \$c >/dev/null && printf '%s ' \$c; done)"

echo "--- chrony ---"
S systemctl enable --now chrony >/dev/null 2>&1
sleep 2
echo "  \$(chronyc tracking 2>/dev/null | grep -E 'Leap status' || echo '동기화 확인 필요')"
REMOTE

# 호스트 키를 바꿨으므로 로컬 known_hosts 를 다시 정리한다.
ssh-keygen -R "$IP" >/dev/null 2>&1 || true
ssh-keygen -R "$ALIAS" >/dev/null 2>&1 || true
ssh_key -o StrictHostKeyChecking=accept-new "$ALIAS" true 2>/dev/null || true

# ---------------------------------------------------------------
# 4. 기준 노드와 비교
# ---------------------------------------------------------------
say "4. 기준 노드(${REFERENCE})와 환경 비교"

compare() {
    local label="$1" cmd="$2"
    local a b
    a=$(ssh_key "$ALIAS" "$cmd" 2>/dev/null | tr -d '\r')
    b=$(ssh_key "$REFERENCE" "$cmd" 2>/dev/null | tr -d '\r')
    if [ "$a" = "$b" ]; then
        printf '  \033[32m일치\033[0m  %-16s %s\n' "$label" "$a"
    else
        printf '  \033[31m불일치\033[0m %-16s\n' "$label"
        printf '           %-8s %s\n' "$NEWHOST" "$a"
        printf '           %-8s %s\n' "$REFERENCE" "$b"
    fi
}

compare "rom-version"  'cat /etc/rom-version 2>/dev/null'
compare "fwver"        'grep -oE "androidboot\.fwver=[^ ]*" /proc/cmdline | cut -d= -f2'
compare "kernel"       'uname -r'
compare "os"           'grep -oP "(?<=^VERSION=\").*(?=\")" /etc/os-release'
compare "glibc"        'ldd --version | head -1 | grep -oE "[0-9]+\.[0-9]+$"'
compare "rknn_runtime" 'strings /usr/lib/librknnrt.so 2>/dev/null | grep -m1 -oE "librknnrt version: [0-9.]+"'
compare "librknnrt_sha" 'sha256sum /usr/lib/librknnrt.so 2>/dev/null | cut -c1-16'
compare "npu_cores"    'ls /sys/class/devfreq/ | grep -c npu'
compare "ram_mb"       'free -m | awk "/Mem:/{print \$2}"'

echo ""
echo "  호스트 키가 기준 노드와 달라야 정상입니다:"
printf '    %-8s %s\n' "$NEWHOST"   "$(ssh_key "$ALIAS" 'ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub | awk "{print \$2}"' 2>/dev/null)"
printf '    %-8s %s\n' "$REFERENCE" "$(ssh_key "$REFERENCE" 'ssh-keygen -lf /etc/ssh/ssh_host_ed25519_key.pub | awk "{print \$2}"' 2>/dev/null)"

# ---------------------------------------------------------------
say "5. 다음 단계"
cat <<NEXT
  1) 실측 정보 수집
       ssh $ALIAS 'bash -s' < scripts/collect-node-info.sh > benchmarks/node-info/${NEWHOST}.txt

  2) 펌웨어가 일치하면 안정성 재검증
       scp crates/npuforge-rknn/native/thread_safety_test.c $ALIAS:~/npuforge-rknn-test/
       ssh $ALIAS 'cd ~/npuforge-rknn-test && gcc -O2 -Wall -o thread_safety_test thread_safety_test.c -lrknnrt -lpthread'
       ssh $ALIAS 'cd ~/npuforge-rknn-test && ./thread_safety_test yolov8n-fp16.rknn 20 5 8'

  3) docs/environment-matrix.md 와 docs/board-worklog.md 갱신
NEXT
