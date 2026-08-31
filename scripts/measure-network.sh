#!/usr/bin/env bash
# 추론망 대역 실측. 링크 협상 속도 + 단일 + 3노드 aggregation.
#
# 왜: 케이블 불량으로 협상이 조용히 낮아지는 사고가 반복됐다. 10GBASE-T 는
# Cat6/6a 를 요구하고 Cat5e 면 2.5G/5G 로 떨어진다. 방치하면 NPU 가 아니라
# 케이블을 측정한다. adrs/014, infrastructure.md §4.
#
# 전제: ~/.ssh/config 에 npuforge-k/q/j/server 별칭.
set -u
SERVER=npuforge-server
SRV_IP=192.168.123.9
NODES="npuforge-k npuforge-q npuforge-j"

echo "== 링크 협상 =="
ssh -o BatchMode=yes "$SERVER" 'ethtool enp4s0 2>/dev/null | grep -E "Speed|Link detected"' | tr -d '\r'
for h in $NODES; do
  printf "%-12s " "$h"
  ssh -o BatchMode=yes "$h" 'printf "eth0=%s Mb/s\n" "$(cat /sys/class/net/eth0/speed)"' | tr -d '\r'
done

echo "== 3노드 동시 → server (nc, 각 ${1:-6}GB) =="
CNT=$((${1:-6} * 1000))
ssh -o BatchMode=yes "$SERVER" 'pkill ncat 2>/dev/null; firewall-cmd --add-port=5201-5203/tcp >/dev/null 2>&1; for p in 5201 5202 5203; do setsid nohup sh -c "/usr/bin/ncat -l $p | dd of=/dev/null bs=1M 2>/tmp/rx_$p" >/dev/null 2>&1 </dev/null & done; sleep 1' 2>&1 | tr -d '\r'
i=1
for h in $NODES; do
  p=$((5200 + i))
  ssh -o BatchMode=yes "$h" "dd if=/dev/zero bs=1M count=$CNT 2>/dev/null | nc -N $SRV_IP $p" &
  i=$((i + 1))
done
wait; sleep 2
ssh -o BatchMode=yes "$SERVER" 'for p in 5201 5202 5203; do echo -n "port $p: "; tail -1 /tmp/rx_$p; done; pkill ncat 2>/dev/null; rm -f /tmp/rx_520*; firewall-cmd --remove-port=5201-5203/tcp >/dev/null 2>&1' 2>&1 | tr -d '\r'
echo "(측정 리스너·방화벽 런타임 규칙 정리 완료)"
