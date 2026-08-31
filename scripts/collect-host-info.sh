#!/usr/bin/env bash
# 스케줄러 호스트의 하드웨어 인벤토리를 마크다운으로 찍는다.
#
# 왜 있는가
#   2026-08-26 서버 교체 때 구서버(Xeon E5-2630L ×2)의 메인보드·RAM 종류·
#   디스크 모델·PCIe 정보가 **어디에도 기록돼 있지 않았다.** 측정 421건이
#   나온 장비인데 CPU/RAM 용량/NIC 이름만 남아 있었다. 장비가 손을 떠난
#   뒤에는 채울 수 없다.
#
#   보드에는 collect-node-info.sh 가 있었지만 호스트에는 없었다.
#   server-profile-collect.sh 는 성능 프로파일러(S3.9a)이지 인벤토리가 아니다.
#
# 원칙
#   **시리얼·자산번호·UUID 는 수집하지 않는다.** 이 출력은 공개 문서로
#   들어간다. 재현에 필요한 것은 모델명과 규격이지 개체 식별자가 아니다.
#
# 사용법
#   원격 (권장) — 대상에 스크립트를 두지 않는다
#     ssh npuforge-server 'bash -s' < scripts/collect-host-info.sh > /tmp/host.md
#   로컬
#     sudo bash scripts/collect-host-info.sh > host.md
#
#   dmidecode 는 root 가 필요하다. 없으면 해당 절이 비고 나머지는 나온다.
set -u

h() { printf '\n## %s\n\n' "$1"; }
kv() { printf '| %s | %s |\n' "$1" "${2:-—}"; }
thead() { printf '| 항목 | 값 |\n|---|---|\n'; }
dmi() { command -v dmidecode >/dev/null 2>&1 && dmidecode -s "$1" 2>/dev/null | grep -v '^#' | head -1; }

printf '# 호스트 인벤토리 — %s\n\n' "$(hostname)"
printf -- '- 수집: %s\n' "$(date '+%Y-%m-%d %H:%M:%S %Z')"
printf -- '- 수집기: `scripts/collect-host-info.sh`\n'

h "시스템"
thead
kv "hostname"   "$(hostname)"
kv "제조사/모델" "$(dmi system-manufacturer) $(dmi system-product-name)"
kv "베이스보드"  "$(dmi baseboard-product-name)"
kv "BIOS"       "$(dmi bios-version) ($(dmi bios-release-date))"
kv "배포판"      "$(. /etc/os-release 2>/dev/null && echo "$PRETTY_NAME")"
kv "커널"        "$(uname -r)"
kv "아키텍처"    "$(uname -m)"
kv "glibc"      "$(ldd --version 2>/dev/null | head -1 | grep -oE '[0-9]+\.[0-9]+$')"
kv "SELinux"    "$(getenforce 2>/dev/null)"

h "CPU"
thead
lscpu 2>/dev/null | grep -E '^(Model name|Socket|Core\(s\) per socket|Thread\(s\) per core|CPU\(s\)|CPU max MHz|CPU min MHz|L3 cache)' \
  | sed 's/  */ /g' | while IFS=: read -r k v; do kv "$k" "$(echo "$v" | sed 's/^ *//')"; done

h "메모리"
thead
kv "총량" "$(free -g 2>/dev/null | awk '/^Mem:/{print $2" GB"}')"
if command -v dmidecode >/dev/null 2>&1; then
  printf '\n| 슬롯 | 용량 | 종류 | 속도 | 제조사 |\n|---|---|---|---|---|\n'
  dmidecode -t memory 2>/dev/null | awk '
    /Memory Device/{loc="";sz="";ty="";sp="";mf=""}
    /^\tLocator:/{sub(/^\tLocator: /,"");loc=$0}
    /^\tSize:/{sub(/^\tSize: /,"");sz=$0}
    /^\tType:/{sub(/^\tType: /,"");ty=$0}
    /^\tSpeed:/{sub(/^\tSpeed: /,"");sp=$0}
    /^\tManufacturer:/{sub(/^\tManufacturer: /,"");mf=$0;
      if(sz!="No Module Installed"&&sz!="")printf("| %s | %s | %s | %s | %s |\n",loc,sz,ty,sp,mf)}'
fi

h "저장장치"
printf '```text\n'; lsblk -d -o NAME,SIZE,MODEL,ROTA 2>/dev/null; echo; df -h / 2>/dev/null; printf '```\n'

h "네트워크"
printf '```text\n'; ip -br a 2>/dev/null | grep -v '^lo'; printf '```\n\n'
printf '| 인터페이스 | 속도 | 드라이버 | PCI | PCIe 링크 |\n|---|---|---|---|---|\n'
for i in /sys/class/net/*; do
  n=$(basename "$i"); [ "$n" = lo ] && continue
  sp=$(ethtool "$n" 2>/dev/null | awk -F': ' '/Speed:/{print $2}')
  dv=$(ethtool -i "$n" 2>/dev/null | awk -F': ' '/^driver:/{print $2}')
  bd=$(ethtool -i "$n" 2>/dev/null | awk -F': ' '/^bus-info:/{print $2}')
  lnk=""
  case "$bd" in
    *:*.*) lnk=$(lspci -vv -s "$bd" 2>/dev/null | awk '/LnkSta:/{sub(/^[ \t]*LnkSta:[ \t]*/,"");print;exit}') ;;
  esac
  printf '| `%s` | %s | %s | %s | %s |\n' "$n" "${sp:-—}" "${dv:-—}" "${bd:-—}" "${lnk:-—}"
done

h "PCIe 슬롯"
if command -v dmidecode >/dev/null 2>&1; then
  printf '| 슬롯 | 규격 | 사용 |\n|---|---|---|\n'
  dmidecode -t slot 2>/dev/null | awk '
    /System Slot Information/{d="";t="";u=""}
    /^\tDesignation:/{sub(/^\tDesignation: /,"");d=$0}
    /^\tType:/{sub(/^\tType: /,"");t=$0}
    /^\tCurrent Usage:/{sub(/^\tCurrent Usage: /,"");u=$0;
      if(d!="")printf("| %s | %s | %s |\n",d,t,u)}'
fi
printf '\n**루트 포트 능력** — 카드가 느리면 슬롯 탓인지 카드 탓인지 여기서 갈린다.\n\n```text\n'
for rp in $(lspci 2>/dev/null | awk '/PCI bridge/{print $1}'); do
  cap=$(lspci -vv -s "$rp" 2>/dev/null | awk '/LnkCap:/{sub(/^[ \t]*/,"");print;exit}')
  sta=$(lspci -vv -s "$rp" 2>/dev/null | awk '/LnkSta:/{sub(/^[ \t]*/,"");print;exit}')
  [ -n "$cap" ] && printf '%s\n  %s\n  %s\n' "$rp" "$cap" "$sta"
done
printf '```\n'

h "PCI 장치"
printf '```text\n'; lspci 2>/dev/null; printf '```\n'

h "서비스"
thead
kv "firewalld"      "$(systemctl is-active firewalld 2>/dev/null) / $(systemctl is-enabled firewalld 2>/dev/null)"
kv "열린 포트(영구)"  "$(firewall-cmd --permanent --list-ports 2>/dev/null)"
kv "chronyd"        "$(systemctl is-active chronyd 2>/dev/null) / $(systemctl is-enabled chronyd 2>/dev/null)"
kv "시각 동기"       "$(timedatectl 2>/dev/null | awk -F': ' '/synchronized/{print $2}')"

printf '\n> 시리얼·자산번호·UUID 는 수집하지 않는다. 재현에 필요한 것은\n'
printf '> 모델명과 규격이지 개체 식별자가 아니다.\n'
