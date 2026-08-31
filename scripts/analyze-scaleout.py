#!/usr/bin/env python3
"""S3.8 — optimized gRPC scale-out 요약.

각 노드 수의 **자기 운영점**에서 비교한다. 고정 concurrency 로 비교하면
configuration effect 가 아니라 overload behavior 를 보게 된다(S3.7 §4.3).

    operating concurrency = peak 처리량의 98% 이상을 내는 가장 낮은 concurrency
    Scaling(N)    = throughput_N / throughput_1
    Efficiency(N) = throughput_N / (N x throughput_1)

percentile 은 run-level 값을 run 사이에서 평균한 것이다(S2 §7.4.1).
pooled 아니므로 절대값 인용은 안 된다.

사용법
    python scripts/analyze-scaleout.py <results.csv>
"""

import csv
import statistics as st
import sys

CEIL_FRAC = 0.98

path = sys.argv[1] if len(sys.argv) > 1 else "results.csv"
rows = [r for r in csv.DictReader(open(path, encoding="utf-8")) if r.get("throughput")]
if not rows:
    print("유효한 run 이 없다.")
    sys.exit(1)

g = {}
for r in rows:
    g.setdefault((int(r["nodes"]), int(r["concurrency"])), []).append(r)

nodes = sorted({k[0] for k in g})


def m(key, f):
    v = [float(r[f]) for r in g[key] if r.get(f)]
    return st.mean(v) if v else float("nan")


def sd(key, f):
    v = [float(r[f]) for r in g[key] if r.get(f)]
    return st.stdev(v) if len(v) > 1 else 0.0


W = 100
NL = chr(10)
print("=" * W)
print("S3.8  optimized gRPC scale-out — 각 노드 수의 운영점에서 비교")
print("=" * W)
print("  (지연은 run-level percentile 의 run 간 평균 — pooled 아님)")

# ── 노드 수별 곡선 ────────────────────────────────────────────────
for n in nodes:
    ks = sorted([k for k in g if k[0] == n], key=lambda k: k[1])
    tot = int(g[ks[0]][0]["conns_total"])
    print(f"{NL}── {n}노드 (커넥션 {g[ks[0]][0]['conns_per_node']}/node = {tot} total) "
          + "─" * (W - 42))
    print(f"{'conc':>6}{'n':>3}{'throughput':>16}{'p50':>9}{'p95':>9}{'p99':>9}"
          f"{'max':>10}{'balance':>9}{'err':>8}")
    print("-" * W)
    for k in ks:
        print(f"{k[1]:>6}{len(g[k]):>3}{m(k, 'throughput'):>9.1f}±{sd(k, 'throughput'):<5.1f}"
              f"{m(k, 'p50_ms'):>9.1f}{m(k, 'p95_ms'):>9.1f}{m(k, 'p99_ms'):>9.1f}"
              f"{m(k, 'max_ms'):>10.1f}{m(k, 'balance_pp'):>8.1f}%p{m(k, 'error_rate'):>8.4f}")

# ── 운영점 ────────────────────────────────────────────────────────
oppt = {}
for n in nodes:
    ks = sorted([k for k in g if k[0] == n], key=lambda k: k[1])
    peak_k = max(ks, key=lambda k: m(k, "throughput"))
    peak = m(peak_k, "throughput")
    oppt[n] = (next((k for k in ks if m(k, "throughput") >= CEIL_FRAC * peak), peak_k),
               peak, peak_k)

print(NL + "─" * W)
print(f"운영점 (peak 의 {CEIL_FRAC:.0%} 이상을 내는 가장 낮은 concurrency)")
print("─" * W)
print(f"{'Nodes':>6}{'Tot.conn':>10}{'Op.conc':>9}{'Throughput':>13}{'p95':>9}{'p99':>9}"
      f"{'Scaling':>10}{'Efficiency':>12}")
print("-" * W)

base = m(oppt[nodes[0]][0], "throughput") if nodes else None
table = []
for n in nodes:
    k, peak, peak_k = oppt[n]
    tp = m(k, "throughput")
    scal = tp / base if base else float("nan")
    eff = scal / n if base else float("nan")
    tot = int(g[k][0]["conns_total"])
    print(f"{n:>6}{tot:>10}{k[1]:>9}{tp:>13.1f}{m(k, 'p95_ms'):>9.1f}"
          f"{m(k, 'p99_ms'):>9.1f}{scal:>9.2f}x{eff:>11.1%}")
    table.append(dict(n=n, k=k, tp=tp, scal=scal, eff=eff, peak=peak, peak_k=peak_k))

err = sum(float(r["error_rate"]) for r in rows if r.get("error_rate"))
print(f"{NL}  총 오류율 합계: {err:.4f}   (0 이 아니면 결과를 신뢰하지 않는다)")

# ── 포화 경고 ─────────────────────────────────────────────────────
print(NL + "─" * W)
print("포화 판정")
print("─" * W)
for t in table:
    ks = sorted([k for k in g if k[0] == t["n"]], key=lambda k: k[1])
    lo, hi = ks[0][1], ks[-1][1]
    pc = t["peak_k"][1]
    if pc == hi:
        print(f"  {t['n']}N: peak 이 sweep 상단(c{hi}) — **포화 미확인**, 더 높은 부하를 봐야 한다.")
    elif t["k"][1] == lo:
        print(f"  {t['n']}N: 운영점이 sweep 하단(c{lo}) — 더 낮은 부하에서도 ceiling 유지 가능.")
    else:
        print(f"  {t['n']}N: peak c{pc}, 운영점 c{t['k'][1]} → 범위 안에서 포화 확인.")

# ── 해석 ──────────────────────────────────────────────────────────
if len(table) >= 2:
    print(NL + "─" * W)
    print("해석")
    print("─" * W)
    last = table[-1]
    print(f"  {last['n']}노드에서 scaling {last['scal']:.2f}x, efficiency {last['eff']:.1%}")
    ideal = base * last["n"]
    print(f"  이상적 {last['n']}N = {ideal:.1f} inf/s, 실제 {last['tp']:.1f} "
          f"(차이 {last['tp'] - ideal:+.1f})")
    if last["eff"] >= 0.95:
        print("  → near-linear 유지. 노드당 전송 최적화가 scale-out 을 해치지 않았다.")
    elif last["eff"] >= 0.85:
        print("  → 선형에서 벗어났다. 노드당 커넥션이 N배로 늘면서 서버 쪽에")
        print("     새 병목이 생겼을 수 있다 — 서버 CPU·NIC·스케줄러를 프로파일한다.")
    else:
        print("  → 크게 꺾였다. **새 병목을 발견한 것**이다. 서버 쪽을 먼저 본다")
        print("     (커넥션 수 N배, 전송량 N배, 스케줄러 팬아웃).")
    bal = max(m(t["k"], "balance_pp") for t in table)
    print(f"{NL}  최대 노드 간 분배 편차: {bal:.1f}%p (0 에 가까울수록 균등)")
