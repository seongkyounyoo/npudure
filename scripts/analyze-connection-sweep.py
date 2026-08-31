#!/usr/bin/env python3
"""S3.7 커넥션 sweep 요약.

핵심 원칙: **최적점을 최대 처리량으로 정하지 않는다.**
S3.6 에서 커넥션 1 → 4 는 처리량 +21.5% 를 줬지만 p95 는 46% 나빠졌다.
처리량만 보면 tail 이 무너지는 지점을 고르게 된다. 그래서 두 축을 같이 낸다.

두 가지를 문서에 못 박아 둔다.

1. 여기서 고르는 것은 통계적 최적값이 아니라 **의도적으로 정한 engineering
   heuristic** 이다. 그래서 이름도 "최적" 이 아니라 **Selected operating
   point** 로 쓴다. 전체 표를 같이 내보이므로, "왜 8 이 아니라 4 인가" 는
   데이터가 직접 답한다.

2. 표의 percentile 은 **run-level percentile 을 run 사이에서 평균한 값**이다.
   요청을 전부 합쳐 다시 구한 pooled percentile 이 아니다. bench 는 요약
   percentile 만 남기므로 현재 데이터로는 pooled 를 계산할 수 없다.
   조건 간 비교에는 유효하나 절대값을 "이 시스템의 p99" 로 인용하면 안 된다.
   (S2 보고서 §7.4.1)

사용법
    python scripts/analyze-connection-sweep.py <results.csv>
"""

import csv
import statistics
import sys

path = sys.argv[1] if len(sys.argv) > 1 else "results.csv"
rows = [r for r in csv.DictReader(open(path, encoding="utf-8")) if r.get("throughput")]
if not rows:
    print("유효한 run 이 없다.")
    sys.exit(1)

g = {}
for r in rows:
    g.setdefault(r["cond"], []).append(r)

# sweep 이면 커넥션 수 순, rps 모드면 off/on 순
IS_SWEEP = all(c.startswith("c") and c[1:].isdigit() for c in g)
conds = sorted(g, key=lambda c: int(c[1:])) if IS_SWEEP \
    else sorted(g, key=lambda c: 0 if "off" in c else 1)


def stat(cond, field):
    v = [float(r[field]) for r in g[cond] if r.get(field)]
    if not v:
        return None, None
    return statistics.mean(v), (statistics.stdev(v) if len(v) > 1 else 0.0)


W = 92
print("=" * W)
print("S3.7  " + ("노드당 커넥션 수 sweep" if IS_SWEEP else "RPS A/B (최적 N 위에서)"))
print("=" * W)
print(f"{'조건':<10}{'n':>3}{'TCP':>5}{'throughput':>17}"
      f"{'p50':>9}{'p95':>9}{'p99':>9}{'max':>9}{'→node':>9}")
print("  (지연은 모두 run-level percentile 의 run 간 평균 — pooled 아님)")
print("-" * W)

base_tp, _ = stat(conds[0], "throughput")
base_p95, _ = stat(conds[0], "p95_ms")
table = []
for c in conds:
    tp, sd = stat(c, "throughput")
    p50, _ = stat(c, "p50_ms")
    p95, _ = stat(c, "p95_ms")
    p99, _ = stat(c, "p99_ms")
    mx, _ = stat(c, "max_ms")
    n2n, _ = stat(c, "net_to_node_ms")
    tcp = statistics.mode([r["tcp_conns"] for r in g[c] if r.get("tcp_conns")]) \
        if any(r.get("tcp_conns") for r in g[c]) else "?"
    d = f"({tp / base_tp - 1:+5.1%})" if base_tp else ""
    print(f"{c:<10}{len(g[c]):>3}{tcp:>5}{tp:>8.1f}±{sd:<4.1f}{d:>9}"
          f"{p50:>9.1f}{p95:>9.1f}{p99:>9.1f}{mx:>9.1f}{n2n:>9.1f}")
    table.append(dict(cond=c, tp=tp, p50=p50, p95=p95, p99=p99))

err = sum(float(r["error_rate"]) for r in rows if r.get("error_rate"))
print(f"\n  총 오류율 합계: {err:.4f}   (0 이 아니면 결과를 신뢰하지 않는다)")

if not IS_SWEEP:
    a, b = table[0], table[1]
    print("\n" + "─" * W)
    print(f"  RPS off → on : {a['tp']:.1f} → {b['tp']:.1f} inf/s "
          f"({b['tp'] / a['tp'] - 1:+.1%}),  p95 {a['p95']:.0f} → {b['p95']:.0f} ms")
    if abs(b["tp"] / a["tp"] - 1) < 0.03:
        print("  → 변화 없음. 흐름을 여러 개로 만든 뒤에도 무효라면,")
        print("     CPU0 softirq 는 상관관계일 뿐 처리량 limiter 가 아니다.")
        print("     (S3.5b 단독보다 훨씬 강한 배제 — 그때는 흐름이 하나뿐이라")
        print("      RPS 가 분산할 대상 자체가 없었다.)")
    elif b["tp"] > a["tp"]:
        print("  → 상승. 단일 커넥션 제약을 풀자 NIC 처리 병목이 드러났고,")
        print("     multi-flow 에서 비로소 RPS 가 효과를 갖기 시작했다.")
    else:
        print("  → 하락. RPS 가 오히려 해롭다 — 캐시 지역성 손실 등을 의심한다.")
    sys.exit(0)

# ── sweep: 처리량–tail 트레이드오프 ────────────────────────────────
print("\n" + "─" * W)
print("Selected operating point  —  처리량–tail 트레이드오프")
print("─" * W)
best_tp = max(table, key=lambda t: t["tp"])
print(f"{'조건':<10}{'throughput':>12}{'vs 최대':>10}{'p95':>10}{'vs 1ch':>10}   판단 근거")
print("-" * W)
for t in table:
    tp_ratio = t["tp"] / best_tp["tp"]
    p95_ratio = t["p95"] / base_p95 if base_p95 else float("nan")
    # 처리량이 최대의 97% 안에 들면서 tail 이 가장 덜 나빠진 지점을 찾는다.
    note = []
    if t is best_tp:
        note.append("최대 처리량")
    if tp_ratio >= 0.97:
        note.append("처리량 최대의 97% 이상")
    if p95_ratio > 2.0:
        note.append("!! p95 2배 초과")
    print(f"{t['cond']:<10}{t['tp']:>12.1f}{tp_ratio:>10.1%}"
          f"{t['p95']:>10.1f}{p95_ratio:>10.2f}x   {', '.join(note) or '-'}")

near = [t for t in table if t["tp"] / best_tp["tp"] >= 0.97]
pick = min(near, key=lambda t: t["p95"]) if near else best_tp
print()
print(f"  최대 처리량               : {best_tp['cond']}  {best_tp['tp']:.1f} inf/s, p95 {best_tp['p95']:.0f} ms")
print(f"  Selected operating point : {pick['cond']}  {pick['tp']:.1f} inf/s, p95 {pick['p95']:.0f} ms")
if pick["cond"] != best_tp["cond"]:
    print(f"    → 처리량은 {pick['tp'] / best_tp['tp']:.1%} 수준이지만 p95 가 "
          f"{best_tp['p95'] - pick['p95']:.0f} ms 낮다. 실시간 추론에서는 이쪽이 낫다.")
print()
print("  규칙(engineering heuristic, 통계적 최적값 아님):")
print("        처리량이 최대의 97% 이상인 후보 중 p95 가 가장 낮은 것.")
print("        곡선이 꺾이는지는 위 표의 'vs 최대' 열로 확인한다.")
print("        (꺾임의 *원인* 은 이 실험이 말하지 않는다 — H2 큐잉/inflight 편차/")
print("         TCP 처리/NPU 도착 burst 등이 섞여 있을 수 있다.)")
print()
print("  ⚠ 이 sweep 은 c32 고정이다. 커넥션을 늘리면 saturation concurrency 가")
print("     c32 위로 이동했을 수 있어, 여기 값은 각 설정의 ceiling 이 아니다.")
print("     상위 후보 1~2개에 대해 concurrency sweep(S3.7b)을 해야 확정된다.")
