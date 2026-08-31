#!/usr/bin/env python3
"""S3.6 A/B 결과 요약.

조건별 평균 ± SD 와 baseline(A) 대비 변화를 낸다. 판정 문구까지 찍는다 —
표만 보고 사람이 매번 해석 규칙을 다시 떠올리게 하면 실수가 난다.

사용법
    python scripts/analyze-h2-channel-ab.py <results.csv>
"""

import csv
import statistics
import sys

NUM = ("throughput", "e2e_p50_ms", "e2e_p95_ms",
       "net_to_node_ms", "net_to_client_ms", "node_queue_ms")
ORDER = ["A_1ch_default", "B_1ch_bigwin", "C_4ch_default", "D_4ch_bigwin"]
LABEL = {
    "A_1ch_default": "A  1ch default",
    "B_1ch_bigwin": "B  1ch bigwin ",
    "C_4ch_default": "C  4ch default",
    "D_4ch_bigwin": "D  4ch bigwin ",
}

path = sys.argv[1] if len(sys.argv) > 1 else "results.csv"
rows = [r for r in csv.DictReader(open(path, encoding="utf-8")) if r.get("throughput")]

g = {}
for r in rows:
    g.setdefault(r["cond"], []).append(r)

conds = [c for c in ORDER if c in g]
if not conds:
    print("유효한 run 이 없다.")
    sys.exit(1)


def stat(cond, field):
    v = [float(r[field]) for r in g[cond] if r.get(field)]
    if not v:
        return None, None
    return statistics.mean(v), (statistics.stdev(v) if len(v) > 1 else 0.0)


print("=" * 92)
print("S3.6  HTTP/2 window × 노드당 커넥션  A/B")
print("=" * 92)
hdr = f"{'조건':<16}{'n':>3}{'TCP':>5}{'throughput':>16}{'E2E p50':>11}{'E2E p95':>10}"
hdr += f"{'→node':>9}{'→client':>10}{'nodeq':>8}"
print(hdr)
print("-" * 92)
print("  (지연은 run-level percentile 의 run 간 평균 — pooled 아님)")

base, _ = stat(conds[0], "throughput")
for c in conds:
    tp, sd = stat(c, "throughput")
    p50, _ = stat(c, "e2e_p50_ms")
    p95, _ = stat(c, "e2e_p95_ms")
    n2n, _ = stat(c, "net_to_node_ms")
    n2c, _ = stat(c, "net_to_client_ms")
    nq, _ = stat(c, "node_queue_ms")
    tcp = statistics.mode([r["tcp_conns"] for r in g[c] if r.get("tcp_conns")]) \
        if any(r.get("tcp_conns") for r in g[c]) else "?"
    delta = f"({tp / base - 1:+5.1%})" if base else ""
    print(f"{LABEL.get(c, c):<16}{len(g[c]):>3}{tcp:>5}"
          f"{tp:>8.1f}±{sd:<4.1f}{delta:>9}"
          f"{p50:>10.1f}{p95:>10.1f}{n2n:>9.1f}{n2c:>10.1f}{nq:>8.2f}")

err = sum(float(r["error_rate"]) for r in rows if r.get("error_rate"))
print(f"\n  총 오류율 합계: {err:.4f}   (0 이 아니면 결과를 신뢰하지 않는다)")

# ── 판정 ──────────────────────────────────────────────────────────
print("\n" + "─" * 92)
THRESH = 0.05   # baseline 대비 5% 이상이면 '상승' 으로 본다


def rise(c):
    tp, _ = stat(c, "throughput")
    return tp is not None and base and (tp / base - 1) >= THRESH


b = rise("B_1ch_bigwin") if "B_1ch_bigwin" in g else False
c4 = rise("C_4ch_default") if "C_4ch_default" in g else False
d = rise("D_4ch_bigwin") if "D_4ch_bigwin" in g else False

if b and not c4:
    v = ["① flow control 이 주요 제약 요인이다. 커넥션 수가 아니라 window 를 바꿨을 때 올랐다.",
         "  → S4 는 io_uring 이 아니라 gRPC/HTTP2 튜닝 방향이다."]
elif c4 and not b:
    v = ["② 노드당 단일 커넥션 구조가 주요 제약 요인이다.",
         "   window 가 아니라 커넥션 수를 바꿨을 때 올랐다.",
         "   다만 그 구조 안에서 TCP per-flow / H2 multiplexing·락 /",
         "   flow control 상호작용 중 무엇인지는 아직 분리되지 않았다.",
         "  → 다음은 커넥션 수 sweep(처리량과 tail 을 함께 본다).",
         "     io_uring 은 여전히 근거 부족."]
elif b and c4:
    v = ["①② 둘 다 영향이 있다. D 가 가장 높으면 둘은 독립적으로 더해진다.",
         "  → 두 설정을 함께 잡고 재측정한 뒤 io_uring 필요성을 다시 본다."]
elif d:
    v = ["B·C 단독으로는 안 올랐는데 D 에서 올랐다.",
         "   두 제약이 동시에 걸려 있어 하나만 풀면 다른 하나가 막는다.",
         "  → 두 설정을 함께 적용하는 방향."]
else:
    v = ["①② 모두 아니다. window 와 커넥션을 다 풀어도 상한이 그대로다.",
         "  → 남는 것은 ③ protobuf·복사·syscall 경로다.",
         "     **여기서 io_uring 이 비로소 정당화된다** — 대역폭도, 스케줄러도,",
         "     CPU 배치도, flow control 도, 커넥션도 아니었다는 배제 근거를 갖는다."]

print("판정: " + (chr(10).join(v) if isinstance(v, list) else v))
print("─" * 92)
print(f"  (기준: baseline {conds[0]} 대비 {THRESH:.0%} 이상 상승을 '상승' 으로 본다)")


# ─────────────────────────────────────────────────────────────────
# 보드 프로파일 (있으면). run 마다 node-profile-collect.sh 가 떠 온
# /proc 원본에서 CPU·syscall 을 뽑아 조건별로 평균한다.
# ─────────────────────────────────────────────────────────────────
import pathlib

PROF = pathlib.Path(path).parent / "profile" / "h2ab"


def _read(d, name):
    f = d / name
    return f.read_text(encoding="utf-8", errors="replace") if f.exists() else None


def _mpstat_all(d):
    for line in (_read(d, "mpstat.txt") or "").splitlines():
        f = line.split()
        if line.startswith("Average:") and len(f) > 11 and f[1] == "all":
            return dict(usr=float(f[2]), sys=float(f[4]),
                        soft=float(f[7]), idle=float(f[11]))
    return None


def _cpu0_busy(d):
    for line in (_read(d, "mpstat.txt") or "").splitlines():
        f = line.split()
        if line.startswith("Average:") and len(f) > 11 and f[1] == "0":
            return 100 - float(f[11]), float(f[7])
    return None, None


def _syscalls(d):
    out = {}
    for w in ("before", "after"):
        t = _read(d, "pio." + w)
        if not t:
            return None
        kv = dict(x.split(": ") for x in t.strip().splitlines())
        out[w] = int(kv["syscr"]) + int(kv["syscw"])
    return out["after"] - out["before"]


def _secs(d):
    b, a = _read(d, "uptime.before"), _read(d, "uptime.after")
    return float(a.split()[0]) - float(b.split()[0]) if b and a else None


if PROF.exists():
    print("\n" + "=" * 92)
    print("보드 프로파일 (king, 조건별 5 run 평균)")
    print("=" * 92)
    print(f"{'조건':<16}{'%usr':>8}{'%sys':>8}{'%soft':>8}{'%idle':>8}"
          f"{'CPU0busy':>10}{'CPU0soft':>10}{'syscall/req':>13}")
    print("-" * 92)
    for c in conds:
        accs = {k: [] for k in ("usr", "sys", "soft", "idle", "c0", "c0s", "sc")}
        for r in g[c]:
            d = PROF / f"{c}_r{r['round']}"
            if not d.exists():
                continue
            m = _mpstat_all(d)
            if m:
                for k in ("usr", "sys", "soft", "idle"):
                    accs[k].append(m[k])
            b0, s0 = _cpu0_busy(d)
            if b0 is not None:
                accs["c0"].append(b0)
                accs["c0s"].append(s0)
            sc, sec = _syscalls(d), _secs(d)
            if sc and sec and r.get("throughput"):
                accs["sc"].append(sc / (float(r["throughput"]) * sec))
        if not accs["usr"]:
            continue
        a = {k: (statistics.mean(v) if v else float("nan")) for k, v in accs.items()}
        print(f"{LABEL.get(c, c):<16}{a['usr']:>8.1f}{a['sys']:>8.1f}{a['soft']:>8.1f}"
              f"{a['idle']:>8.1f}{a['c0']:>10.1f}{a['c0s']:>10.1f}{a['sc']:>13.1f}")
