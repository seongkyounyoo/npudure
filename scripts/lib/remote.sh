#!/usr/bin/env bash
# 노드 원격 실행 공용 헬퍼.
#
#   . "$(dirname "${BASH_SOURCE[0]}")/lib/remote.sh"
#
# 여기 모아 둔 것들은 전부 실측에서 한 번씩 당한 함정을 피하기 위한 것이다.
# 각 함수의 주석에 근거를 적어 두었다.

# ─────────────────────────────────────────────────────────────────────
# sudo 비밀번호
#
# 저장소에 비밀번호를 넣지 않는다. 이 프로젝트는 공개 예정이고, 커밋
# 히스토리에 한 번 들어가면 지우는 데 히스토리 재작성이 필요하다.
#
# 우선순위
#   1. NPUFORGE_SUDO_PASS 환경변수
#   2. ~/.npuforge/sudo-pass 파일 (권한 600)
#   3. 없으면 안내하고 실패
#
# 권장: 노드에 NOPASSWD sudoers 규칙을 두고 이 경로를 아예 쓰지 않는 것.
#       `npuforge_sudo_hint` 가 그 방법을 안내한다.
# ─────────────────────────────────────────────────────────────────────

npuforge_sudo_pass() {
    if [ -n "${NPUFORGE_SUDO_PASS:-}" ]; then
        printf '%s' "$NPUFORGE_SUDO_PASS"
        return 0
    fi
    local f="${HOME}/.npuforge/sudo-pass"
    if [ -r "$f" ]; then
        head -1 "$f"
        return 0
    fi
    return 1
}

npuforge_sudo_hint() {
    cat >&2 <<'HINT'
노드 sudo 비밀번호가 설정되지 않았습니다. 다음 중 하나를 선택하세요.

  1) 환경변수 (이번 셸에서만)
       export NPUFORGE_SUDO_PASS='...'

  2) 파일 (반복 사용)
       mkdir -p ~/.npuforge
       printf '%s\n' '...' > ~/.npuforge/sudo-pass
       chmod 600 ~/.npuforge/sudo-pass

  3) 권장 — 노드에 NOPASSWD 규칙을 두고 비밀번호를 아예 쓰지 않기
       ssh npuforge-k
       sudo visudo -f /etc/sudoers.d/npuforge
         pi ALL=(root) NOPASSWD: /usr/bin/tee, /usr/bin/install, \
                                 /bin/systemctl, /usr/bin/cat
     이 경우 NPUFORGE_SUDO_PASS 를 비워 두면 sudo 를 그대로 호출합니다.
HINT
}

# 원격에서 sudo 명령을 실행한다.
#
#   npuforge_ssh_sudo npuforge-k cat /sys/kernel/debug/rknpu/version
#
# 비밀번호가 없으면 sudo 를 그냥 호출한다(NOPASSWD 설정된 경우 동작).
npuforge_ssh_sudo() {
    local host="$1"; shift
    local pass
    if pass="$(npuforge_sudo_pass)"; then
        # `sudo -S` 는 stdin 첫 줄을 비밀번호로 소비한다. 따라서 명령에
        # 파이프로 데이터를 넘길 수 없다. 파일이 필요하면 scp 로 보낸 뒤
        # 원격에서 이동시킨다. board-worklog.md §2.4 참조.
        printf '%s\n' "$pass" | ssh -o BatchMode=yes "$host" "sudo -S -p '' $*" 2>/dev/null
    else
        ssh -o BatchMode=yes "$host" "sudo -n $*" 2>/dev/null
    fi
}

# 파일을 원격의 root 소유 경로에 설치한다.
#
#   npuforge_install_root npuforge-k ./unit.service /etc/systemd/system/x.service 644
#
# ssh 안에서 heredoc 과 sudo 를 중첩하면 따옴표가 조용히 깨져 **파일이
# 아예 생기지 않는데 종료 코드는 0** 이다. 실제로 그렇게 실패했다.
# board-worklog.md §2.21 참조. 그래서 scp 로 보낸 뒤 install 한다.
npuforge_install_root() {
    local host="$1" src="$2" dst="$3" mode="${4:-644}"
    local tmp="/tmp/npuforge-install-$$"

    scp -q "$src" "$host:$tmp" || return 1
    npuforge_ssh_sudo "$host" "install -m $mode -o root -g root $tmp $dst" || {
        ssh -o BatchMode=yes "$host" "rm -f $tmp" 2>/dev/null
        return 1
    }
    ssh -o BatchMode=yes "$host" "rm -f $tmp" 2>/dev/null
    # 실제로 생겼는지 확인한다. 확인하지 않으면 실패를 성공으로 넘긴다.
    ssh -o BatchMode=yes "$host" "test -f $dst" 2>/dev/null
}

# 원격에서 특정 실행 파일이 몇 개 돌고 있는지 센다.
#
#   npuforge_count_running npuforge-k sustained_load_test thread_safety_test
#
# `pgrep -f` 를 쓰지 않는다. 명령줄을 매칭하므로 **자기 자신을 실행한
# 래퍼 셸까지 센다.** 실제로 양방향으로 틀렸다.
#   부하 실행 중 → 0 (놓침) / 부하 없음 → 2 (자기 셸을 셈)
# `/proc/PID/exe` 는 실제 실행 파일을 가리키므로 셸이 끼어들지 않는다.
# board-worklog.md §2.21 참조.
npuforge_count_running() {
    local host="$1"; shift
    local pattern=""
    local name
    for name in "$@"; do
        pattern="${pattern}${pattern:+|}*${name}"
    done
    [ -z "$pattern" ] && { echo 0; return 0; }

    ssh -o BatchMode=yes "$host" "
        n=0
        for p in /proc/[0-9]*; do
            case \"\$(readlink \"\$p/exe\" 2>/dev/null)\" in
                $pattern) n=\$((n+1)) ;;
            esac
        done
        echo \$n
    " 2>/dev/null || echo 0
}

# 원격에서 프로그램을 백그라운드로 띄우고 실제로 떴는지 확인한다.
#
#   npuforge_spawn npuforge-k /abs/path/prog "arg1 arg2" /tmp/out.log
#
# `cd DIR && setsid nohup ./prog ... &` 형태를 쓰지 않는다. `&` 가 리스트
# 전체에 걸리는데 ssh 가 즉시 끊기면 서브셸이 setsid 에 닿기 전에 죽는다.
# **실패해도 종료 코드 0, stderr 비어 있음** — 아무 신호가 없다.
# 절대경로로 띄우고, 띄운 뒤 확인한다. board-worklog.md §2.21 참조.
npuforge_spawn() {
    local host="$1" bin="$2" args="$3" logfile="${4:-/dev/null}"

    case "$bin" in
        /*) ;;
        *) echo "npuforge_spawn: 절대경로가 필요하다: $bin" >&2; return 2 ;;
    esac

    ssh -n -o BatchMode=yes "$host" \
        "setsid nohup $bin $args > $logfile 2>&1 < /dev/null &" || return 1

    sleep 3
    local name running
    name="$(basename "$bin")"
    running="$(npuforge_count_running "$host" "$name")"
    if [ "${running:-0}" -eq 0 ]; then
        echo "npuforge_spawn: $host 에서 $name 이 뜨지 않았다 (로그: $logfile)" >&2
        return 1
    fi
    return 0
}

# ─────────────────────────────────────────────────────────────────────
# 클러스터 복구
#
# 측정 스크립트는 1노드 구성을 만들려고 queen·jack 의 노드를 죽인다.
# 그런데 복구 블록은 오랫동안 queen 만 되살렸다. 그래서 jack 이 한 번
# 내려가면 이후 모든 run 이 조용히 jack 을 죽인 채로 지나갔다
# (2026-08-20 에 실제로 그렇게 됐다).
#
# 기동 시 **로그 리다이렉트를 반드시 붙인다.** 핸드오프 절차에 이게 빠져
# 있어서 jack 이 왜 죽었는지 사후에 알 수 없었다. 원인 규명을 막는 것은
# 프로세스가 죽는 것보다 나쁘다.
#
#   npuforge_restore_cluster            세 노드 전부
#   npuforge_restore_cluster k q        지정한 노드만
# ─────────────────────────────────────────────────────────────────────
npuforge_node_start() {
    local host=$1
    ssh -o BatchMode=yes "$host" \
        'pgrep npuforge-node >/dev/null 2>&1 || {
             setsid nohup /home/pi/npuforge/npuforge-node \
                 --config /home/pi/npuforge/node.toml \
                 >>/home/pi/npuforge/node.log 2>&1 </dev/null & disown;
         }; exit 0' 2>/dev/null
}

npuforge_restore_cluster() {
    local hosts="${*:-k q j}"
    local h
    for h in $hosts; do
        npuforge_node_start "npuforge-$h"
    done
    sleep 5
    for h in $hosts; do
        printf '  %-16s ' "npuforge-$h"
        ssh -o BatchMode=yes "npuforge-$h" \
            'pgrep npuforge-node >/dev/null && echo running || echo DOWN' \
            2>/dev/null | tr -d '\r'
    done
}

# ─────────────────────────────────────────────────────────────────────
# 클러스터 점유 확인 — 두 하네스가 같은 클러스터를 때리는 것을 막는다
#
# 2026-08-21 사고. 정책 A/B 하네스를 중단한 줄 알고 capacity 교정을
# 띄웠는데 중단이 실패해 **두 하네스가 각각 c36 으로 같은 3노드를
# 때렸다**(합 72 동시성).
#
#   교정 기준선   197.4 inf/s   (정상 391.2)   노드 편차 2.73x
#   다음 run      오류율 82.4%
#
# "king·jack 만 느리다" 로 보여 클러스터 고장으로 오진하기 직전이었다.
# 실제로는 멀쩡했다 — 정리 후 391.2 inf/s / 오류 0 / 편차 1.02x.
#
# 왜 로컬 프로세스 확인으로는 부족한가
#   Windows(git-bash)에서 `TaskStop` 도 `pkill -f` 도 하네스를 못 잡았고
#   `ps -ef` 에도 보이지 않았다. PowerShell `Get-CimInstance Win32_Process`
#   로만 보였다. 로컬 관측은 플랫폼에 따라 거짓말을 한다.
#   그래서 **공유 자원 쪽(서버)에서 직접 확인한다.**
# ─────────────────────────────────────────────────────────────────────

npuforge_assert_cluster_free() {
    local who="${1:-이 하네스}" out
    out=$(ssh -o BatchMode=yes npuforge-server         'ps -eo pid,etimes,args | grep npuforge-bench | grep -v grep' 2>/dev/null | tr -d '
')
    if [ -n "$out" ]; then
        {
            echo
            echo "!! 서버에서 npuforge-bench 가 이미 돌고 있다. ${who}를 시작하지 않는다."
            echo "   두 하네스가 같은 클러스터를 때리면 측정이 조용히 오염된다."
            echo
            echo "$out" | sed 's/^/     /'
            echo
            echo "   확인: 다른 하네스가 살아 있는지 본다."
            echo "     Windows  powershell -c \"Get-CimInstance Win32_Process -Filter \\\"Name='bash.exe'\\\" | ? { \$_.CommandLine -match 'scripts/run-' } | select ProcessId,CommandLine\""
            echo "     정리     ssh npuforge-server 'pkill -9 npuforge-bench'"
        } >&2
        return 1
    fi
    return 0
}
