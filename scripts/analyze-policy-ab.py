#!/usr/bin/env python3
"""S0-C — 팬리스 열 불균질 상태에서 정책 A/B 분석.

닫으려는 인과고리
    S0-A 관측: 열 편차 있음, king capacity 낮음, RR 이 king 에도 1/3 을 보냄.
    미확정  : "손실 = 열 편차 x 부하 무인지 정책"

    정책을 바꿨을 때 (a) 분배가 실제로 이동하고 (b) 손실이 회수되면 인과가
    닫힌다. 둘 다 봐야 한다 — 분배만 움직이고 처리량이 그대로일 수도 있다.

사용법
    python scripts/analyze-policy-ab.py <결과디렉터리>
"""

import csv
import pathlib
import statistics as st
import sys

ROOT = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else ".")
RAW = ROOT / "raw"
NL = chr(10)
ORDER = ["round-robin", "least-queue", "ect"]

rows = [r for r in csv.DictReader(open(RAW / "results.csv", encoding="utf-8"))
        if r.get("throughput")]
if not rows:
    print("유효한 run 이 없다.")
    sys.exit(1)

g = {}
for r in rows:
    g.setdefault(r["policy"], []).append(r)
pols = [p for p in ORDER if p in g]


def m(p, f):
    v = [float(r[f]) for r in g[p] if r.get(f)]
    return st.mean(v) if v else float("nan")


def sd(p, f):
    v = [float(r[f]) for r in g[p] if r.get(f)]
    return st.stdev(v) if len(v) > 1 else 0.0


W = 104
print("=" * W)
print("S0-C  정책 A/B (팬리스, thermal steady-state)")
print("=" * W)
print("  같은 3노드 / 커넥션 2·node / c36. 달라지는 것은 스케줄링 정책뿐이다.")

print(f"{NL}{'Policy':<14}{'n':>3}{'Throughput':>14}{'p50':>8}{'p95':>8}{'p99':>8}"
      f"{'king%':>8}{'jack%':>8}{'queen%':>8}{'err':>8}")
print("-" * W)
base = m(pols[0], "throughput") if pols else None
for p in pols:
    print(f"{p:<14}{len(g[p]):>3}{m(p, 'throughput'):>9.1f}±{sd(p, 'throughput'):<4.1f}"
          f"{m(p, 'p50_ms'):>8.1f}{m(p, 'p95_ms'):>8.1f}{m(p, 'p99_ms'):>8.1f}"
          f"{m(p, 'king_share'):>8.1f}{m(p, 'jack_share'):>8.1f}"
          f"{m(p, 'queen_share'):>8.1f}{m(p, 'error_rate'):>8.4f}")

print(f"{NL}── 노드별 p50 (ms) — 분배가 이동하면 여기도 따라 움직인다 " + "─" * 44)
print(f"{'Policy':<14}{'king':>10}{'jack':>10}{'queen':>10}{'최대/최소':>12}")
print("-" * W)
for p in pols:
    k, j, q = m(p, "king_p50"), m(p, "jack_p50"), m(p, "queen_p50")
    vals = [x for x in (k, j, q) if x > 0]
    ratio = max(vals) / min(vals) if vals and min(vals) > 0 else float("nan")
    print(f"{p:<14}{k:>10.1f}{j:>10.1f}{q:>10.1f}{ratio:>11.2f}x")

# ── 노드별 CPU busy ───────────────────────────────────────────────
def cpu_busy(tag, board):
    out = {}
    for when in ("before", "after"):
        f = RAW / f"cpu_{tag}_{board}.{when}"
        if not f.exists():
            return None
        v = [int(x) for x in f.read_text().split()[1:]]
        out[when] = v
    d = [a - b for a, b in zip(out["after"], out["before"])]
    tot = sum(d)
    if tot <= 0:
        return None
    idle = d[3] + d[4]
    return (tot - idle) / tot * 100


have_cpu = any((RAW / f"cpu_{p}_r{r['round']}_k.before").exists()
               for p in pols for r in g[p])
if have_cpu:
    print(f"{NL}── 노드별 CPU busy% — '놀고 있다' 를 지연이 아니라 utilization 으로 " + "─" * 30)
    print(f"{'Policy':<14}{'king':>10}{'jack':>10}{'queen':>10}")
    print("-" * W)
    for p in pols:
        acc = {b: [] for b in ("k", "j", "q")}
        for r in g[p]:
            for b in acc:
                v = cpu_busy(f"{p}_r{r['round']}", b)
                if v is not None:
                    acc[b].append(v)
        if not any(acc.values()):
            continue
        print(f"{p:<14}" + "".join(
            f"{st.mean(acc[b]):>9.1f}%" if acc[b] else f"{'-':>10}"
            for b in ("k", "j", "q")))

err = sum(float(r["error_rate"]) for r in rows if r.get("error_rate"))
print(f"{NL}  총 오류율 합계: {err:.4f}")
socs = [float(r["max_soc_c"]) for r in rows if r.get("max_soc_c")]
if socs:
    print(f"  soc 최대 온도 범위: {min(socs):.1f} ~ {max(socs):.1f}°C "
          f"(정책 간 열 조건이 비슷해야 공정한 비교다)")

# ── 판정 ──────────────────────────────────────────────────────────
print(NL + "─" * W)
print("판정 — 분배가 이동했는가, 그리고 손실이 회수됐는가")
print("─" * W)
rr = "round-robin"
if rr not in g:
    sys.exit(0)
rr_tp = m(rr, "throughput")
for p in pols:
    if p == rr:
        continue
    tp = m(p, "throughput")
    ks = m(p, "king_share")
    moved = abs(ks - m(rr, "king_share")) >= 3.0      # king 분배가 3%p 이상 이동
    gained = (tp / rr_tp - 1) >= 0.02                 # 처리량 2% 이상 상승
    print(f"{NL}  {p}: 처리량 {rr_tp:.1f} → {tp:.1f} ({tp / rr_tp - 1:+.1%}), "
          f"king 분배 {m(rr, 'king_share'):.1f}% → {ks:.1f}%")
    if moved and gained:
        print("    → **인과 닫힘.** 분배가 이동했고 손실이 회수됐다.")
        print("       Thermal heterogeneity reduces node capacity, and state-aware")
        print("       scheduling recovers performance by adapting load allocation.")
    elif moved and not gained:
        print("    → 분배는 이동했으나 처리량이 안 올랐다. 이동 폭이 부족하거나")
        print("       병목이 다른 곳(전송 경로)에 있다.")
    elif not moved:
        print("    → **분배가 이동하지 않았다.** 정책의 상태 신호가 thermal-induced")
        print("       capacity degradation 을 감지하지 못한다는 뜻이다.")
        print("       (queue_depth/in_flight 는 느린 노드에서도 낮게 유지될 수 있다 —")
        print("        느린 것은 큐가 아니라 서비스 시간이기 때문이다.)")
print(NL + "  기준: king 분배 3%p 이상 이동 = '이동', 처리량 2% 이상 = '회수'.")
print("        측정 전에 정한 값이며 결과에 맞춰 바꾸지 않는다.")

# ── 4차: 이질 게이트 + LQ vs ECT 판정 (S0_C_POLICY_AB.md §17) ──────
# 규칙은 측정 전에 문서에 등록했다. 여기 숫자는 그 문서의 사본이다.
GATE_RATIO = 2.0     # RR 노드 p50 최대/최소 — 강한 이질 재현 기준
GATE_SOC = 85.0      # 보조 지표
BAND_TP = 0.02       # 처리량 승리 밴드
BAND_TAIL = 0.05     # p99 승리 밴드

if "least-queue" in g and "ect" in g:
    print(NL + "─" * W)
    print("4차 판정 — 강한 이질 게이트 → LQ vs ECT (§17, 사전 등록)")
    print("─" * W)

    # 게이트: RR 은 적응하지 않으므로 raw capacity 편차를 그대로 드러낸다.
    rr_ratios = []
    for r in g.get(rr, []):
        v = [float(r[f]) for f in ("king_p50", "jack_p50", "queen_p50")
             if r.get(f) and float(r[f]) > 0]
        if len(v) == 3:
            rr_ratios.append(max(v) / min(v))
    rr_ratio = st.mean(rr_ratios) if rr_ratios else float("nan")
    # soc 는 **1초 열 로거**에서 읽는다. CSV 의 max_soc_c 는 run 종료 후
    # 순간값이라 run 간 냉각 골짜기에 떨어진다 — 이름과 달리 최대가 아니고
    # 실제보다 ~5°C 낮다 (§17.5). 기준(85°C)은 그대로, 출처만 바꾼다.
    soc_src = "열로거"
    peaks = {}
    for b in ("k", "q", "j"):
        f = RAW / "thermal" / f"{b}.log"
        if not f.exists():
            continue
        v = [float(ln.split()[1]) for ln in f.read_text().splitlines()[1:]
             if len(ln.split()) > 1]
        if v:
            peaks[b] = max(v)
    if peaks:
        rr_soc = min(peaks.values())   # 세 보드 모두 넘어야 한다
    else:
        soc_src = "CSV(순간값 — ~5°C 과소)"
        rr_soc = max((float(r["max_soc_c"]) for r in g.get(rr, []) if r.get("max_soc_c")),
                     default=float("nan"))

    print(f"{NL}  이질 게이지 (RR 노드 p50 최대/최소): {rr_ratio:.2f}x  "
          f"[기준 >= {GATE_RATIO}x]{NL}  soc 최대({soc_src}, 세 보드 중 최저): "
          f"{rr_soc:.1f}°C [보조 >= {GATE_SOC}°C]  " +
          " ".join(f"{b}={peaks[b]:.1f}" for b in ("k", "q", "j") if b in peaks))
    print(f"  참고: S0-A 2.4x / 86~88°C,  S0-C 2차 1.33x / 78~79°C")

    if not (rr_ratio >= GATE_RATIO):
        print(f"{NL}  → **게이트 미달. LQ vs ECT 를 판정하지 않는다.**")
        print("     조건 미달을 결과로 포장하지 않는다 — 연속 가열 설계를 다시 본다.")
    else:
        lq_tp, ec_tp = m("least-queue", "throughput"), m("ect", "throughput")
        lq_99, ec_99 = m("least-queue", "p99_ms"), m("ect", "p99_ms")
        d_tp = ec_tp / lq_tp - 1          # + 면 ECT 처리량 우위
        d_99 = 1 - lq_99 / ec_99          # + 면 LQ tail 우위
        print(f"{NL}  처리량  LQ {lq_tp:.1f}±{sd('least-queue','throughput'):.1f}"
              f"   ECT {ec_tp:.1f}±{sd('ect','throughput'):.1f}   ECT−LQ {d_tp:+.1%}"
              f"  [밴드 {BAND_TP:.0%}]")
        print(f"  p99     LQ {lq_99:.1f}±{sd('least-queue','p99_ms'):.1f}"
              f"   ECT {ec_99:.1f}±{sd('ect','p99_ms'):.1f}   LQ 유리 {d_99:+.1%}"
              f"  [밴드 {BAND_TAIL:.0%}]")

        tp_win = d_tp >= BAND_TP
        tail_win = d_99 >= BAND_TAIL
        if tp_win and not tail_win:
            v = "**`ect` 유지.** 처리량 우위가 밴드를 넘고 tail 손해는 안 넘는다. 질문 닫힘."
        elif tail_win and not tp_win:
            v = "**기본값을 `least-queue` 로 변경.** tail 우위가 밴드를 넘고 처리량 손해는 안 넘는다. 질문 닫힘."
        elif tp_win and tail_win:
            v = ("**지배 없음.** 둘 다 밴드를 넘는다 — `ect` 유지(현직)하되 "
                 "트레이드오프를 명시하고 질문은 열어 둔다.")
        else:
            v = ("**구분 불가.** 강한 이질에서도 어느 축도 밴드를 못 넘었다. "
                 "`ect` 유지, 질문 닫힘 — 기본값 선택이 중요하지 않다는 것도 답이다.")
        print(f"{NL}  → {v}")
    print(f"{NL}  밴드/게이트는 측정 전 §17 에 등록했다. 결과에 맞춰 옮기지 않는다.")
