#!/usr/bin/env python3
"""S3.5 transport profiling 원본 → 지표 표.

무엇을 답하려는 스크립트인가
    노드당 ~115 inf/s (로컬 direct ~160 대비 -30%) 를 막는 것이 무엇인가.
    `01-TECHSPEC.md` §15.4 가 요구하는 지표를 조건별로 뽑아 비교한다.

계산은 전부 여기서 한다. 보드에서는 /proc 원본만 떠 온다. 나중에 다른
각도로 다시 볼 수 있어야 하기 때문이다.

사용법
    python scripts/analyze-transport-profile.py [결과디렉터리]
"""

import json
import pathlib
import re
import sys

USER_HZ = 100          # aarch64 Linux 기본
NIC = "eth0"
LINK_BPS = 2.34e9      # king→server iperf3 실측 (docs/infrastructure.md §230)
# RK3576 = 4×A53 + 4×A72. governor=performance 라 고정 클럭으로 본다.
# cycles/req 는 PMU(perf) 가 없어 이 클럭으로 환산한 근사값이다.
CORE_MHZ = {0: 2016, 1: 2016, 2: 2016, 3: 2016,
            4: 2208, 5: 2208, 6: 2208, 7: 2208}

ROOT = pathlib.Path(sys.argv[1] if len(sys.argv) > 1
                    else "results/transport-profile-20260820/raw")


def read(cond, name):
    p = ROOT / cond / name
    return p.read_text(encoding="utf-8", errors="replace") if p.exists() else None


def window_seconds(cond):
    """수집 창의 실제 길이. /proc/uptime 차이가 가장 믿을 만하다."""
    b, a = read(cond, "uptime.before"), read(cond, "uptime.after")
    return float(a.split()[0]) - float(b.split()[0])


def proc_cpu_seconds(cond):
    """대상 프로세스가 쓴 CPU 초 (utime+stime).

    /proc/PID/stat 은 comm 이 괄호로 싸여 있고 그 안에 공백이 들 수 있어
    단순 split 이 어긋난다. 닫는 괄호 뒤부터 세는 것이 안전하다.
    """
    out = {}
    for when in ("before", "after"):
        s = read(cond, "pstat." + when)
        if s is None:
            return None
        tail = s[s.rindex(")") + 2:].split()
        # tail[0] 이 state. utime=14, stime=15 (1-based) → tail[11], tail[12]
        out[when] = (int(tail[11]) + int(tail[12])) / USER_HZ
    return out["after"] - out["before"]


def proc_syscalls(cond):
    out = {}
    for when in ("before", "after"):
        s = read(cond, "pio." + when)
        if s is None:
            return None
        d = dict(line.split(": ") for line in s.strip().splitlines())
        out[when] = (int(d["syscr"]), int(d["syscw"]))
    return (out["after"][0] - out["before"][0],
            out["after"][1] - out["before"][1])


def proc_ctxt(cond):
    """프로세스 전체 컨텍스트 스위치.

    /proc/PID/status 는 메인 스레드 값만 준다. task/* 를 전부 더해야 한다 —
    처음에 이걸 놓쳐 '평생 20회' 라는 값이 나왔다.
    """
    out = {}
    for when in ("before", "after"):
        s = read(cond, "ctxt." + when)
        if s is None:
            return None
        vol = nonvol = 0
        for line in s.splitlines():
            if ":" not in line:
                continue
            n = int(line.rsplit(":", 1)[1])
            if "nonvoluntary" in line:
                nonvol += n
            elif "voluntary" in line:
                vol += n
        out[when] = (vol, nonvol)
    return (out["after"][0] - out["before"][0],
            out["after"][1] - out["before"][1])


def netdev(cond):
    out = {}
    for when in ("before", "after"):
        for line in read(cond, "netdev." + when).splitlines():
            if line.strip().startswith(NIC + ":"):
                f = line.split(":", 1)[1].split()
                out[when] = dict(rx_bytes=int(f[0]), rx_pkts=int(f[1]),
                                 tx_bytes=int(f[8]), tx_pkts=int(f[9]))
    return {k: out["after"][k] - out["before"][k] for k in out["before"]}


def nic_irqs():
    """eth0 에 할당된 MSI IRQ 번호. 없으면 None (전체 IRQ 로 폴백)."""
    f = ROOT / "nic-topology.txt"
    if not f.exists():
        return None
    m = re.search(r"^msi_irqs=(.*)$", f.read_text(encoding="utf-8"), re.M)
    return set(m[1].split()) if m else None


NIC_IRQS = nic_irqs()


def counters(cond, fname, pattern, only=None):
    """/proc/interrupts, /proc/softirqs 처럼 '라벨 + 코어별 숫자' 형식."""
    out = {}
    for when in ("before", "after"):
        tot, percore = 0, {}
        for line in read(cond, fname + "." + when).splitlines():
            head = line.strip().split(":", 1)[0]
            if not re.match(pattern, line.strip()):
                continue
            if only is not None and head not in only:
                continue
            nums = [int(x) for x in line.split()[1:] if x.isdigit()]
            for i, n in enumerate(nums[:8]):
                percore[i] = percore.get(i, 0) + n
            tot += sum(nums[:8])
        out[when] = (tot, percore)
    return (out["after"][0] - out["before"][0],
            {i: out["after"][1].get(i, 0) - out["before"][1].get(i, 0)
             for i in range(8)})


def mpstat_avg(cond):
    """mpstat 의 Average: 블록 → {코어: {필드: %}}."""
    rows = {}
    for line in read(cond, "mpstat.txt").splitlines():
        if not line.startswith("Average:"):
            continue
        f = line.split()
        if f[1] == "CPU":
            continue
        rows[f[1]] = dict(usr=float(f[2]), sys=float(f[4]),
                          irq=float(f[6]), soft=float(f[7]),
                          idle=float(f[11]))
    return rows


def throughput(cond):
    if cond == "idle":
        return 0.0
    if cond == "cluster":
        p = ROOT / "cluster" / "bench.json"
        return json.loads(p.read_text(encoding="utf-8"))["summary"]["throughput"]
    # local: direct.log 의 10초 누적 표에서 수집 창에 해당하는 구간 기울기.
    # 요약값(158.8)은 초반 램프를 포함하므로 창 안의 실제값보다 낮다.
    pts = [(int(m[1]), int(m[2])) for m in
           re.finditer(r"^\s*(\d+)s\s+(\d+)\s+[\d.]+\s+\d+\s*$",
                       read(cond, "direct.log"), re.M)]
    lo = [p for p in pts if p[0] <= 20][-1]
    hi = [p for p in pts if p[0] <= 70][-1]
    return (hi[1] - lo[1]) / (hi[0] - lo[0])


def temp(cond, when):
    s = read(cond, "temp." + when)
    return int(s.strip()) / 1000 if s else float("nan")


CONDS = [c for c in ("idle", "cluster", "local") if (ROOT / c).exists()]
R = {}

for c in CONDS:
    secs = window_seconds(c)
    tp = throughput(c)
    mp = mpstat_avg(c)
    irq_tot, irq_core = counters(c, "interrupts", r"^\d+:", only=NIC_IRQS)
    nrx_tot, _ = counters(c, "softirqs", r"^NET_RX:")
    ntx_tot, _ = counters(c, "softirqs", r"^NET_TX:")
    # 코어별 busy 시간을 클럭으로 가중해 대략적인 사이클 수를 만든다.
    busy_cycles = sum((100 - mp[str(i)]["idle"]) / 100 * secs
                      * CORE_MHZ[i] * 1e6 for i in range(8))
    R[c] = dict(
        secs=secs, tp=tp, reqs=tp * secs, mp=mp, nd=netdev(c),
        irq_tot=irq_tot, nrx=nrx_tot, ntx=ntx_tot,
        cpu_s=proc_cpu_seconds(c), sysc=proc_syscalls(c), ctx=proc_ctxt(c),
        busy_cycles=busy_cycles,
        nthreads=int(read(c, "nthreads.after").strip()),
        t0=temp(c, "before"), t1=temp(c, "after"),
    )


def fmt(v, spec):
    return "—" if v is None else format(v, spec)


W = 26


def row(label, fn, spec=".1f"):
    print(f"{label:<{W}}" + "".join(f"{fmt(fn(R[c]), spec):>16}" for c in CONDS))


def per_req(r, val):
    return (val / r["reqs"]) if r["reqs"] else None


print("=" * 78)
print("S3.5  transport cost profiling — king (RK3576, 8core, 팬, performance)")
print("=" * 78)
print(f"{'':<{W}}" + "".join(f"{c:>16}" for c in CONDS))
print("-" * 78)
row("수집 창 (s)", lambda r: r["secs"])
row("throughput (inf/s)", lambda r: r["tp"])
row("추론 건수", lambda r: r["reqs"], ".0f")
row("스레드 수", lambda r: r["nthreads"], "d")
row("온도 after (C)", lambda r: r["t1"])

print()
print("── CPU 점유 (mpstat 평균, 8코어 전체) " + "─" * 33)
row("%usr (all)", lambda r: r["mp"]["all"]["usr"])
row("%sys (all)", lambda r: r["mp"]["all"]["sys"])
row("%soft (all)", lambda r: r["mp"]["all"]["soft"])
row("%idle (all)", lambda r: r["mp"]["all"]["idle"])

print()
print("── CPU0 — NIC IRQ 가 전부 여기 붙는다 (단일 큐) " + "─" * 24)
row("CPU0 %soft", lambda r: r["mp"]["0"]["soft"])
row("CPU0 %sys", lambda r: r["mp"]["0"]["sys"])
row("CPU0 %usr", lambda r: r["mp"]["0"]["usr"])
row("CPU0 busy (100-idle)", lambda r: 100 - r["mp"]["0"]["idle"])

print()
print("── 네트워크 (" + NIC + ") " + "─" * 45)
row("RX (Gbps)", lambda r: r["nd"]["rx_bytes"] * 8 / r["secs"] / 1e9, ".3f")
row("TX (Gbps)", lambda r: r["nd"]["tx_bytes"] * 8 / r["secs"] / 1e9, ".3f")
row("링크 대비 RX (%)", lambda r: r["nd"]["rx_bytes"] * 8 / r["secs"] / LINK_BPS * 100)
row("링크 대비 TX (%)", lambda r: r["nd"]["tx_bytes"] * 8 / r["secs"] / LINK_BPS * 100)
row("RX 패킷/s", lambda r: r["nd"]["rx_pkts"] / r["secs"], ".0f")
row("NIC IRQ/s (eth0)", lambda r: r["irq_tot"] / r["secs"], ".0f")
row("NET_RX softirq/s", lambda r: r["nrx"] / r["secs"], ".0f")

print()
print("── 요청당 비용 (TECHSPEC §15.4 가 요구하는 지표) " + "─" * 22)
row("syscalls/req", lambda r: per_req(r, sum(r["sysc"])) if r["sysc"] else None)
row("  read/req", lambda r: per_req(r, r["sysc"][0]) if r["sysc"] else None)
row("  write/req", lambda r: per_req(r, r["sysc"][1]) if r["sysc"] else None)
row("ctx switch/req (vol)", lambda r: per_req(r, r["ctx"][0]) if r["ctx"] else None)
row("ctx switch/req (nonvol)", lambda r: per_req(r, r["ctx"][1]) if r["ctx"] else None)
row("프로세스 CPU-ms/req",
    lambda r: per_req(r, r["cpu_s"] * 1000) if r["cpu_s"] is not None else None)
row("보드 전체 CPU-ms/req",
    lambda r: per_req(r, (100 - r["mp"]["all"]["idle"]) / 100 * 8 * r["secs"] * 1000))
row("≈ Mcycles/req", lambda r: per_req(r, r["busy_cycles"] / 1e6))
row("RX 패킷/req", lambda r: per_req(r, r["nd"]["rx_pkts"]))

if "cluster" in R and "local" in R:
    c, l = R["cluster"], R["local"]
    print()
    print("── cluster vs local — transport 가 보드에서 실제로 쓰는 비용 " + "─" * 10)
    cb, lb = 100 - c["mp"]["all"]["idle"], 100 - l["mp"]["all"]["idle"]
    print(f"  처리량             {l['tp']:.1f} → {c['tp']:.1f} inf/s "
          f"({(c['tp'] / l['tp'] - 1) * 100:+.1f}%)")
    print(f"  보드 전체 busy     {lb:.1f}% → {cb:.1f}%  ({cb - lb:+.1f}%p)")
    print(f"  CPU0 busy          {100 - l['mp']['0']['idle']:.1f}% → "
          f"{100 - c['mp']['0']['idle']:.1f}%  "
          f"({(l['mp']['0']['idle'] - c['mp']['0']['idle']):+.1f}%p)")
    print(f"  CPU0 %soft         {l['mp']['0']['soft']:.1f}% → {c['mp']['0']['soft']:.1f}%")
    print()
    busiest = max(range(8), key=lambda i: 100 - c["mp"][str(i)]["idle"])
    others = max(100 - c["mp"][str(i)]["idle"] for i in range(8) if i != busiest)
    print(f"  cluster 최번 코어   CPU{busiest} busy "
          f"{100 - c['mp'][str(busiest)]['idle']:.1f}%   "
          f"(나머지 코어 최대 {others:.1f}%)")
    print("  코어별 busy% cluster:  " +
          "  ".join(f"c{i}={100 - c['mp'][str(i)]['idle']:.0f}" for i in range(8)))
    print("  코어별 busy% local  :  " +
          "  ".join(f"c{i}={100 - l['mp'][str(i)]['idle']:.0f}" for i in range(8)))
