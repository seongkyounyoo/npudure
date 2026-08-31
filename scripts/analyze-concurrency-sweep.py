#!/usr/bin/env python3
"""S3.7b — 후보별 concurrency sweep 요약.

S3.7a 는 c32 고정이라 각 설정의 ceiling 이 아니었다. 여기서 부하를 흔들어
후보마다 **실제 ceiling 과 그때의 tail** 을 찾고 운영점을 확정한다.

**운영 concurrency 의 정의를 코드에 못 박는다.**

    operating concurrency = peak 처리량의 98% 이상을 내는 가장 낮은 concurrency

98% 인 이유: 관측된 run 간 SD 가 ±1 inf/s 수준이라 99% 로 잡으면 임계가
측정 noise 와 겹친다. 이 정의가 없으면 132.8 / 134.1 / 134.3 같은 결과에서
"어디가 knee 냐" 가 매번 사람 판단으로 들어간다.

percentile 은 run-level 값을 run 사이에서 평균한 것이다. pooled 아니다
(S2 §7.4.1). 조건 간 비교엔 유효하나 절대값 인용은 안 된다.

사용법
    python scripts/analyze-concurrency-sweep.py <results.csv>
"""

import csv
import statistics as st
import sys

path = sys.argv[1] if len(sys.argv) > 1 else "results.csv"
rows = [r for r in csv.DictReader(open(path, encoding="utf-8")) if r.get("throughput")]
if not rows:
    print("유효한 run 이 없다.")
    sys.exit(1)

g = {}
for r in rows:
    g.setdefault((int(r["conns"]), int(r["concurrency"])), []).append(r)

conns = sorted({k[0] for k in g})
concs = sorted({k[1] for k in g})


def m(key, f):
    v = [float(r[f]) for r in g[key] if r.get(f)]
    return st.mean(v) if v else float("nan")


def sd(key, f):
    v = [float(r[f]) for r in g[key] if r.get(f)]
    return st.stdev(v) if len(v) > 1 else 0.0


NLC = chr(10)
W = 96
print("=" * W)
print("S3.7b  concurrency sweep — 후보별 실제 ceiling 과 그때의 tail")
print("=" * W)
print("  (지연은 run-level percentile 의 run 간 평균 — pooled 아님)")

for n in conns:
    print(f"\n── 커넥션 {n} 개 " + "─" * (W - 16))
    print(f"{'conc':>6}{'n':>3}{'throughput':>16}{'p50':>9}{'p95':>9}{'p99':>9}"
          f"{'max':>10}{'→node':>9}{'nodeq':>8}")
    print("-" * W)
    for c in concs:
        k = (n, c)
        if k not in g:
            continue
        print(f"{c:>6}{len(g[k]):>3}{m(k, 'throughput'):>9.1f}±{sd(k, 'throughput'):<5.1f}"
              f"{m(k, 'p50_ms'):>9.1f}{m(k, 'p95_ms'):>9.1f}{m(k, 'p99_ms'):>9.1f}"
              f"{m(k, 'max_ms'):>10.1f}{m(k, 'net_to_node_ms'):>9.1f}{m(k, 'node_queue_ms'):>8.2f}")

err = sum(float(r["error_rate"]) for r in rows if r.get("error_rate"))
print(f"\n  총 오류율 합계: {err:.4f}   (0 이 아니면 결과를 신뢰하지 않는다)")

# ── 후보별 peak 와 operating point ────────────────────────────────
# 정의를 코드에 못 박는다. 없으면 132.8 / 134.1 / 134.3 같은 결과에서
# "어디가 knee 냐" 가 매번 사람 판단으로 들어간다.
CEIL_FRAC = 0.98

print(NLC + "─" * W)
print(f"운영점 — peak 의 {CEIL_FRAC:.0%} 이상을 내는 **가장 낮은** concurrency")
print("─" * W)
print(f"{'conn':>6}{'peak':>9}{'@c':>6}{'  |':>4}{'op.conc':>9}"
      f"{'throughput':>12}{'peak대비':>10}{'p50':>9}{'p95':>9}{'p99':>9}{'max':>10}")
print("-" * W)
ceil = {}
oppt = {}
for n in conns:
    ks = sorted([k for k in g if k[0] == n], key=lambda k: k[1])
    peak_k = max(ks, key=lambda k: m(k, "throughput"))
    peak = m(peak_k, "throughput")
    ceil[n] = peak_k
    # 가장 낮은 concurrency 부터 훑어 임계를 처음 넘는 지점
    op = next((k for k in ks if m(k, "throughput") >= CEIL_FRAC * peak), peak_k)
    oppt[n] = op
    print(f"{n:>6}{peak:>9.1f}{peak_k[1]:>6}{'  |':>4}{op[1]:>9}"
          f"{m(op, 'throughput'):>12.1f}{m(op, 'throughput') / peak:>10.1%}"
          f"{m(op, 'p50_ms'):>9.1f}{m(op, 'p95_ms'):>9.1f}"
          f"{m(op, 'p99_ms'):>9.1f}{m(op, 'max_ms'):>10.1f}")

print()
print(f"  정의: operating concurrency = peak 의 {CEIL_FRAC:.0%} 이상을 내는 가장 낮은 concurrency.")
print("        98% 인 이유 — run 간 SD 가 ±1 inf/s 수준이라 99% 는 측정 noise 와 겹친다.")
if any(oppt[n][1] == concs[0] for n in conns):
    print(f"  ⚠ 운영점이 sweep 하단(c{concs[0]})인 후보가 있다 — 더 낮은 부하에서도")
    print("     ceiling 을 유지할 수 있다. 범위를 아래로 더 넓혀야 확정된다.")

# ── 포화했는가 ────────────────────────────────────────────────────
print("\n" + "─" * W)
print("포화 판정 — 최고점이 sweep 의 끝이면 아직 포화가 아닐 수 있다")
print("─" * W)
for n in conns:
    best_c = ceil[n][1]
    if best_c == concs[-1]:
        print(f"  conn {n}: 최고점이 sweep 상단(c{best_c})이다. "
              f"**포화 미확인** — 더 높은 concurrency 를 봐야 한다.")
    elif best_c == concs[0]:
        print(f"  conn {n}: 최고점이 sweep 하단(c{best_c})이다. "
              f"더 낮은 부하에서 최고일 수 있다.")
    else:
        print(f"  conn {n}: c{best_c} 에서 최고이고 양쪽이 더 낮다 → 포화 확인.")

# ── 운영점 후보 비교 ──────────────────────────────────────────────
if len(conns) >= 2:
    print("\n" + "─" * W)
    print(f"운영점 비교 (각 후보의 operating point = peak {CEIL_FRAC:.0%} 기준)")
    print("─" * W)
    a, b = conns[0], conns[1]
    ka, kb = oppt[a], oppt[b]
    ta, tb = m(ka, "throughput"), m(kb, "throughput")
    pa, pb = m(ka, "p95_ms"), m(kb, "p95_ms")
    qa, qb = m(ka, "p99_ms"), m(kb, "p99_ms")
    print(f"  conn {a} @c{ka[1]} : {ta:6.1f} inf/s   p95 {pa:6.1f}   p99 {qa:6.1f}")
    print(f"  conn {b} @c{kb[1]} : {tb:6.1f} inf/s   p95 {pb:6.1f}   p99 {qb:6.1f}")
    hi, lo = (b, a) if tb > ta else (a, b)
    khi, klo = oppt[hi], oppt[lo]
    dt = m(khi, "throughput") / m(klo, "throughput") - 1
    dp = m(khi, "p95_ms") / m(klo, "p95_ms") - 1
    dq = m(khi, "p99_ms") / m(klo, "p99_ms") - 1
    print(f"\n  conn {hi} 는 conn {lo} 대비 처리량 {dt:+.1%} 를 위해 "
          f"p95 {dp:+.1%}, p99 {dq:+.1%} 를 치른다.")
    if dt < 0.05 and dp > 0.15:
        print(f"  → 처리량 이득이 작고 tail 대가가 크다. **conn {lo} 를 권한다.**")
    elif dt >= 0.05 and dp <= dt * 2:
        print(f"  → 처리량 이득이 tail 대가에 비해 크다. **conn {hi} 를 권한다.**")
    else:
        print("  → 경계다. 규칙이 결론을 내주지 못한다 — 표를 근거로 사람이 정한다.")
        print("     (임계값을 결과에 맞춰 옮기지 않는다. 그건 사후 합리화다.)")
