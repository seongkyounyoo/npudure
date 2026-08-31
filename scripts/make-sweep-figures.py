#!/usr/bin/env python3
"""S3.7 커넥션 sweep 그림.

왜 dual-axis 가 아니라 Pareto scatter 인가
    처리량과 tail 을 한 그림의 좌우 축에 얹으면 "오른쪽 위가 좋다" 처럼
    읽히는데, 여기서는 그게 틀린 독법이다. 처리량은 높을수록, p95 는
    낮을수록 좋다. 두 축을 좌표로 쓰면 트레이드오프가 점의 위치로 바로
    보인다 — 8ch 가 처리량은 조금 높은데 p95 가 폭발하면 점이 오른쪽으로
    멀리 밀려나 한눈에 드러난다. 발표에서 dual-axis 보다 방어적이다.

출력
    fig_sweep_pareto.png     X=p95, Y=throughput, 점=커넥션 수  ← 대표
    fig_sweep_throughput.png 커넥션 수 → 처리량 (곡선이 꺾이는지)
    fig_sweep_latency.png    커넥션 수 → p50/p95/p99

색상은 저장소의 기존 검증 팔레트(`make-figures.py`)를 그대로 쓴다.

사용법
    python scripts/make-sweep-figures.py <results.csv> [출력디렉터리]
"""
import csv
import os
import statistics as st
import sys

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

CSV = sys.argv[1] if len(sys.argv) > 1 else "results/connection-sweep-20260820/raw/results.csv"
FIG = sys.argv[2] if len(sys.argv) > 2 else os.path.join(os.path.dirname(CSV), "..", "figures")
os.makedirs(FIG, exist_ok=True)

# --- 검증된 default 팔레트 (light), make-figures.py 와 동일 ---
SURFACE, INK, INK2, GRID = "#fcfcfb", "#0b0b0b", "#52514e", "#e6e6e3"
BLUE, ORANGE, AQUA, YELLOW = "#2a78d6", "#eb6834", "#1baf7a", "#eda100"

plt.rcParams.update({
    "figure.facecolor": SURFACE, "axes.facecolor": SURFACE,
    "savefig.facecolor": SURFACE, "text.color": INK,
    "axes.edgecolor": INK2, "axes.labelcolor": INK, "xtick.color": INK2,
    "ytick.color": INK2, "font.size": 11, "axes.titlesize": 13,
    "axes.titleweight": "bold", "figure.dpi": 130, "axes.titlepad": 22,
})


def clean(ax, axis="y"):
    ax.spines[["top", "right"]].set_visible(False)
    ax.grid(axis=axis, color=GRID, lw=1, zorder=0)
    ax.set_axisbelow(True)


# --- 집계 ---
rows = [r for r in csv.DictReader(open(CSV, encoding="utf-8")) if r.get("throughput")]
g = {}
for r in rows:
    g.setdefault(int(r["conns"]), []).append(r)
conns = sorted(g)


def m(n, f):
    return st.mean([float(r[f]) for r in g[n] if r.get(f)])


def sd(n, f):
    v = [float(r[f]) for r in g[n] if r.get(f)]
    return st.stdev(v) if len(v) > 1 else 0.0


tp = {n: m(n, "throughput") for n in conns}
p50 = {n: m(n, "p50_ms") for n in conns}
p95 = {n: m(n, "p95_ms") for n in conns}
p99 = {n: m(n, "p99_ms") for n in conns}

# percentile 이 run-level 평균이라는 사실을 그림에도 남긴다.
# 표에만 적어 두면 그림이 따로 돌아다닐 때 오해가 생긴다.
CAVEAT = os.environ.get("NPUFORGE_CAPTION",
    "run-level percentiles averaged over runs (not pooled) · closed-loop c32")

# ── Figure 1: Pareto scatter (대표) ───────────────────────────────
fig, ax = plt.subplots(figsize=(6.6, 4.6))
best = max(tp, key=lambda n: tp[n])
near = [n for n in conns if tp[n] >= 0.97 * tp[best]]
pick = min(near, key=lambda n: p95[n]) if near else best

for n in conns:
    is_pick = (n == pick)
    ax.scatter(p95[n], tp[n], s=190 if is_pick else 120,
               color=BLUE if is_pick else INK2,
               edgecolor=INK if is_pick else "none",
               linewidth=1.6 if is_pick else 0, zorder=4)
    ax.annotate(f"{n}ch", (p95[n], tp[n]), textcoords="offset points",
                xytext=(10, 6), fontsize=10.5,
                color=INK if is_pick else INK2,
                fontweight="bold" if is_pick else "normal")

# 커넥션 수 순서대로 이어 경로를 보여 준다. 어느 방향으로 움직였는지가 보인다.
ax.plot([p95[n] for n in conns], [tp[n] for n in conns],
        color=GRID, lw=1.4, zorder=2)

clean(ax, axis="both")
ax.set_xlabel("p95 latency (ms)  ← lower is better")
ax.set_ylabel("Throughput (inf/s)  ↑ higher is better")
ax.set_title(os.environ.get("NPUFORGE_TITLE",
    "Connections per node: throughput–tail trade-off"))
ax.text(0, 1.015, CAVEAT, transform=ax.transAxes, fontsize=8.5, color=INK2)
# 범례에 scatter proxy 를 쓰면 그 마커가 축 안에 그려져 여섯 번째
# 데이터점처럼 읽힌다. 텍스트로 대체한다.
ax.text(0.02, 0.04, f"● selected operating point: {pick}ch",
        transform=ax.transAxes, ha="left", va="bottom",
        fontsize=9.5, color=BLUE, fontweight="bold")
fig.tight_layout()
fig.savefig(f"{FIG}/fig_sweep_pareto.png")
plt.close(fig)

# ── Figure 2: 커넥션 수 → 처리량 (꺾이는 지점 확인) ───────────────
fig, ax = plt.subplots(figsize=(6.2, 4.2))
x = range(len(conns))
ax.errorbar(list(x), [tp[n] for n in conns],
            yerr=[sd(n, "throughput") for n in conns],
            marker="o", ms=7, lw=2, color=BLUE, capsize=4, zorder=3)
clean(ax)
ax.set_xticks(list(x))
ax.set_xticklabels([f"{n}" for n in conns])
ax.set_xlabel("Connections per node")
ax.set_ylabel("Throughput (inf/s)")
ax.set_title("Throughput vs connections per node")
ax.text(0, 1.015, CAVEAT,
        transform=ax.transAxes, fontsize=8.5, color=INK2)
fig.tight_layout()
fig.savefig(f"{FIG}/fig_sweep_throughput.png")
plt.close(fig)

# ── Figure 3: 커넥션 수 → 지연 분포 ───────────────────────────────
fig, ax = plt.subplots(figsize=(6.2, 4.2))
for k, c, lab in [(p50, BLUE, "p50"), (p95, ORANGE, "p95"), (p99, AQUA, "p99")]:
    ax.plot(list(x), [k[n] for n in conns], marker="o", ms=6, lw=2,
            color=c, label=lab, zorder=3)
clean(ax)
ax.set_xticks(list(x))
ax.set_xticklabels([f"{n}" for n in conns])
ax.set_xlabel("Connections per node")
ax.set_ylabel("Latency (ms)")
ax.set_title("Median improves while the tail worsens")
ax.text(0, 1.015, CAVEAT, transform=ax.transAxes, fontsize=8.5, color=INK2)
ax.legend(frameon=False, ncol=3, loc="upper left")
fig.tight_layout()
fig.savefig(f"{FIG}/fig_sweep_latency.png")
plt.close(fig)

print(f"그림 3장 -> {FIG}")
print(f"  최대 처리량               : {best}ch  {tp[best]:.1f} inf/s, p95 {p95[best]:.0f} ms")
print(f"  Selected operating point : {pick}ch  {tp[pick]:.1f} inf/s, p95 {p95[pick]:.0f} ms")
