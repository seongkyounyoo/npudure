#!/usr/bin/env python3
"""S3.9a — scale-out efficiency loss profiling 분석.

질문: optimized 3N 에서 사라진 약 4.7% efficiency 가 shared path 의 어디서
생기는가. 노드당 부하가 셋 다 동일(c12/node)하므로, **서버 쪽 비용이 노드
수와 함께 어떻게 증가하는지**만 본다.

서버에 sysstat 이 없어 mpstat 을 못 쓴다. /proc/stat 델타로 코어별 busy 를
직접 계산한다 — mpstat 이 하는 일과 동일하다.

사용법
    python scripts/analyze-scaleout-profile.py <결과디렉터리>
"""

import csv
import pathlib
import re
import statistics as st
import sys

ROOT = pathlib.Path(sys.argv[1] if len(sys.argv) > 1
                    else "results/scaleout-profile-20260821")
RAW = ROOT / "raw"
NIC = "enp4s0"
USER_HZ = 100
NL = chr(10)


def read(d, name):
    p = d / name
    return p.read_text(encoding="utf-8", errors="replace") if p.exists() else None


def secs(d):
    b, a = read(d, "uptime.before"), read(d, "uptime.after")
    return float(a.split()[0]) - float(b.split()[0]) if b and a else None


def cpu_delta(d):
    """/proc/stat 델타 → (전체 busy 코어수, softirq 코어수, 코어별 busy%)."""
    out = {}
    for when in ("before", "after"):
        t = read(d, "stat." + when)
        if not t:
            return None
        cores = {}
        for line in t.splitlines():
            m = re.match(r"^cpu(\d+)\s+(.*)$", line)
            if m:
                v = [int(x) for x in m[2].split()]
                cores[int(m[1])] = v
        out[when] = cores
    res = {}
    for i in out["before"]:
        b, a = out["before"][i], out["after"][i]
        dv = [x - y for x, y in zip(a, b)]
        total = sum(dv)
        if total <= 0:
            continue
        idle = dv[3] + dv[4]            # idle + iowait
        soft = dv[6] if len(dv) > 6 else 0
        res[i] = dict(busy=(total - idle) / total * 100,
                      soft=soft / total * 100)
    return res


def netdev(d):
    out = {}
    for when in ("before", "after"):
        t = read(d, "netdev." + when)
        if not t:
            return None
        for line in t.splitlines():
            if line.strip().startswith(NIC + ":"):
                f = line.split(":", 1)[1].split()
                out[when] = dict(rx_b=int(f[0]), rx_p=int(f[1]), rx_drop=int(f[3]),
                                 tx_b=int(f[8]), tx_p=int(f[9]), tx_drop=int(f[11]))
    if len(out) < 2:
        return None
    return {k: out["after"][k] - out["before"][k] for k in out["before"]}


def proc_cpu(d):
    out = {}
    for when in ("before", "after"):
        t = read(d, "pstat." + when)
        if not t:
            return None
        tail = t[t.rindex(")") + 2:].split()
        out[when] = (int(tail[11]) + int(tail[12])) / USER_HZ
    return out["after"] - out["before"]


def thread_cpu(d):
    """스레드별 CPU 초 델타. 직렬화 지점이 있으면 특정 스레드만 튄다."""
    out = {}
    for when in ("before", "after"):
        t = read(d, "tstat." + when)
        if not t:
            return None
        per = {}
        for line in t.splitlines():
            try:
                tid = line.split()[0]
                rest = line[line.rindex(")") + 2:].split()
                per[tid] = (int(rest[11]) + int(rest[12])) / USER_HZ
            except Exception:
                continue
        out[when] = per
    return {k: out["after"][k] - out["before"].get(k, 0)
            for k in out["after"] if out["after"][k] - out["before"].get(k, 0) >= 0}


def syscalls(d):
    out = {}
    for when in ("before", "after"):
        t = read(d, "pio." + when)
        if not t:
            return None
        kv = dict(x.split(": ") for x in t.strip().splitlines())
        out[when] = int(kv["syscr"]) + int(kv["syscw"])
    return out["after"] - out["before"]


def nodeconn(d, when="mid"):
    t = read(d, "nodeconn." + when) or read(d, "nodeconn.after")
    try:
        return int(t.strip())
    except Exception:
        return None


def ss_stats(d, when="mid"):
    """ss -tin 에서 노드행 커넥션의 rtt/retrans 를 뽑는다."""
    t = read(d, "ss." + when) or read(d, "ss.after")
    if not t:
        return None
    rtts, retr = [], 0
    lines = t.splitlines()
    for i, line in enumerate(lines):
        if re.search(r"192\.168\.123\.[345]:51001", line):
            detail = lines[i + 1] if i + 1 < len(lines) else ""
            m = re.search(r"rtt:([\d.]+)", detail)
            if m:
                rtts.append(float(m[1]))
            r = re.search(r"retrans:\d+/(\d+)", detail)
            if r:
                retr += int(r[1])
    return dict(rtt=st.mean(rtts) if rtts else float("nan"),
                n=len(rtts), retrans=retr)


# ── bench 결과 ────────────────────────────────────────────────────
rows = [r for r in csv.DictReader(open(RAW / "results.csv", encoding="utf-8"))
        if r.get("throughput")]
if not rows:
    print("유효한 run 이 없다.")
    sys.exit(1)
g = {}
for r in rows:
    g.setdefault(int(r["nodes"]), []).append(r)
nodes = sorted(g)


def bm(n, f):
    v = [float(r[f]) for r in g[n] if r.get(f)]
    return st.mean(v) if v else float("nan")


W = 100
print("=" * W)
print("S3.9a  scale-out efficiency loss profiling — 서버 쪽 비용이 노드 수와 함께 어떻게 느는가")
print("=" * W)
print("  노드당 부하는 셋 다 동일(커넥션 2/node, c12/node). 달라지는 것은 노드 수뿐이다.")

base = bm(nodes[0], "throughput")
print(f"{NL}{'Nodes':>6}{'conc':>6}{'throughput':>12}{'Scaling':>9}{'Eff':>8}"
      f"{'p50':>8}{'p95':>8}{'p99':>8}{'schedQ':>9}{'route':>8}{'nodeQ':>8}{'bal':>7}")
print("-" * W)
for n in nodes:
    tp = bm(n, "throughput")
    print(f"{n:>6}{int(bm(n, 'concurrency')):>6}{tp:>12.1f}{tp / base:>8.2f}x"
          f"{tp / base / n:>7.1%}{bm(n, 'p50_ms'):>8.1f}{bm(n, 'p95_ms'):>8.1f}"
          f"{bm(n, 'p99_ms'):>8.1f}{bm(n, 'sched_queue_ms'):>9.3f}"
          f"{bm(n, 'sched_route_ms'):>8.3f}{bm(n, 'node_queue_ms'):>8.3f}"
          f"{bm(n, 'balance_pp'):>6.1f}%p")

err = sum(float(r["error_rate"]) for r in rows if r.get("error_rate"))
print(f"{NL}  총 오류율 합계: {err:.4f}")

# ── 서버 프로파일 ─────────────────────────────────────────────────
SRV = RAW / "server"
if not SRV.exists():
    print(f"{NL}서버 프로파일 없음 ({SRV})")
    sys.exit(0)

print(NL + "=" * W)
print("서버 (24코어, 10GbE, RX 큐 24개)")
print("=" * W)
print(f"{'Nodes':>6}{'busy코어':>10}{'최번코어':>10}{'softirq코어':>12}"
      f"{'RX Gbps':>10}{'TX Gbps':>10}{'10G대비':>9}{'drop':>7}"
      f"{'schedCPU':>10}{'sysc/req':>10}")
print("-" * W)

srv = {}
for n in nodes:
    ds = [SRV / f"n{n}_r{r['rep']}" for r in g[n]]
    ds = [d for d in ds if d.exists()]
    if not ds:
        continue
    acc = {k: [] for k in ("busy", "top", "soft", "rx", "tx", "drop", "scpu", "sysc")}
    for d in ds:
        s = secs(d)
        c = cpu_delta(d)
        nd = netdev(d)
        if not (s and c and nd):
            continue
        reqs = bm(n, "throughput") * s
        acc["busy"].append(sum(v["busy"] for v in c.values()) / 100)
        acc["top"].append(max(v["busy"] for v in c.values()))
        acc["soft"].append(sum(v["soft"] for v in c.values()) / 100)
        acc["rx"].append(nd["rx_b"] * 8 / s / 1e9)
        acc["tx"].append(nd["tx_b"] * 8 / s / 1e9)
        acc["drop"].append(nd["rx_drop"] + nd["tx_drop"])
        pc = proc_cpu(d)
        if pc is not None:
            acc["scpu"].append(pc / 100 * 100 / s * 100 / 100)  # 코어 환산
        sc = syscalls(d)
        if sc and reqs:
            acc["sysc"].append(sc / reqs)
    if not acc["busy"]:
        continue
    a = {k: (st.mean(v) if v else float("nan")) for k, v in acc.items()}
    srv[n] = a
    util = max(a["rx"], a["tx"]) / 10.0 * 100
    print(f"{n:>6}{a['busy']:>10.2f}{a['top']:>9.1f}%{a['soft']:>12.2f}"
          f"{a['rx']:>10.3f}{a['tx']:>10.3f}{util:>8.1f}%{a['drop']:>7.0f}"
          f"{a['scpu']:>10.2f}{a['sysc']:>10.1f}")

print(f"{NL}  busy코어/softirq코어 = 코어 환산 사용량 (24 가 전체 포화)")
print("  schedCPU = 스케줄러 프로세스가 쓴 코어 수")

# ── 스케줄러 스레드 편중 ──────────────────────────────────────────
print(NL + "─" * W)
print("스케줄러 스레드 편중 — 직렬화 지점이 있으면 특정 스레드만 튄다")
print("─" * W)
for n in nodes:
    ds = [SRV / f"n{n}_r{r['rep']}" for r in g[n]]
    ds = [d for d in ds if d.exists()]
    if not ds:
        continue
    d = ds[0]
    tc = thread_cpu(d)
    s = secs(d)
    if not tc or not s:
        continue
    vals = sorted(tc.values(), reverse=True)
    top = [v / s * 100 for v in vals[:5]]
    nthr = read(d, "nthreads.after")
    print(f"  {n}N  스레드 {nthr.strip() if nthr else '?'}개  "
          f"상위5 코어점유%: " + "  ".join(f"{x:.0f}" for x in top)
          + f"   (합 {sum(v / s * 100 for v in vals):.0f}%)")

# ── 커넥션 상태 ───────────────────────────────────────────────────
print(NL + "─" * W)
print("스케줄러→노드 커넥션 상태 (부하 중간 시점)")
print("─" * W)
for n in nodes:
    ds = [SRV / f"n{n}_r{r['rep']}" for r in g[n]]
    ds = [d for d in ds if d.exists()]
    if not ds:
        continue
    ss_ = ss_stats(ds[0])
    nc = nodeconn(ds[0])
    exp = n * 2
    flag = "" if nc == exp else f"  !! 기대 {exp}"
    if ss_:
        print(f"  {n}N  커넥션 {nc}개{flag}   rtt {ss_['rtt']:.2f} ms   "
              f"retrans {ss_['retrans']}")
    else:
        print(f"  {n}N  커넥션 {nc}개{flag}")

# ── 해석 ──────────────────────────────────────────────────────────
if len(srv) >= 2 and nodes[-1] in srv and nodes[0] in srv:
    a1, aN = srv[nodes[0]], srv[nodes[-1]]
    N = nodes[-1]
    print(NL + "─" * W)
    print("해석")
    print("─" * W)
    tpN, tp1 = bm(N, "throughput"), bm(nodes[0], "throughput")
    print(f"  처리량 {tp1:.1f} → {tpN:.1f}  ({tpN / tp1:.2f}×, eff {tpN / tp1 / N:.1%})")
    print(f"  서버 busy 코어 {a1['busy']:.2f} → {aN['busy']:.2f} "
          f"({aN['busy'] / a1['busy']:.2f}× / 이상 {N}×)")
    util = max(aN["rx"], aN["tx"]) / 10.0 * 100
    print(f"  10G 사용률 {util:.1f}%,  최번 코어 {aN['top']:.1f}%,  "
          f"drop {aN['drop']:.0f}")
    print()
    if util > 85:
        print("  → 10G 링크가 유력하다. 사용률이 85% 를 넘었다.")
    elif aN["top"] > 90:
        print("  → 특정 코어가 포화했다. 단일 코어 직렬화 지점을 의심한다.")
    elif aN["drop"] > 0:
        print("  → 패킷 drop 이 있다. 큐/버퍼 설정을 본다.")
    elif aN["busy"] / a1["busy"] > N * 1.15:
        print("  → 서버 CPU 가 노드 수보다 빠르게 늘었다. 팬아웃 비용이 초선형이다.")
    else:
        print("  → 서버 쪽 자원에서 뚜렷한 포화가 안 보인다.")
        print("     efficiency 손실이 서버 자원 포화가 아니라면 남는 후보는")
        print("     노드 쪽 또는 경로 지연(팬아웃 왕복)이다. 노드 프로파일과")
        print("     p95 증가분을 함께 본다.")
