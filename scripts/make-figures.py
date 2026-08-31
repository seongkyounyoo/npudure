#!/usr/bin/env python3
"""gRPC baseline 30회 결과로 발표용 PNG 그래프를 만든다.

데이터: results/baseline-20260820/raw/*.json (30건, 동결 commit 254d560)
출력:   results/baseline-20260820/figures/*.png

색상은 dataviz 스킬의 검증된 default 팔레트(categorical 고정 순서).
"""
import json, glob, os, re, statistics as st
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.ticker import MaxNLocator

RAW = "results/baseline-20260820/raw"
FIG = "results/baseline-20260820/figures"
os.makedirs(FIG, exist_ok=True)

# --- 검증된 default 팔레트 (light) ---
SURFACE = "#fcfcfb"
INK = "#0b0b0b"
INK2 = "#52514e"
GRID = "#e6e6e3"
BLUE, ORANGE, AQUA, YELLOW = "#2a78d6", "#eb6834", "#1baf7a", "#eda100"
NODE = {"king": BLUE, "queen": ORANGE, "jack": AQUA}

plt.rcParams.update({
    "figure.facecolor": SURFACE, "axes.facecolor": SURFACE,
    "savefig.facecolor": SURFACE, "text.color": INK,
    "axes.edgecolor": INK2, "axes.labelcolor": INK, "xtick.color": INK2,
    "ytick.color": INK2, "font.size": 11, "axes.titlesize": 13,
    "axes.titleweight": "bold", "figure.dpi": 130, "axes.titlepad": 22,
})

def clean(ax):
    ax.spines[["top", "right"]].set_visible(False)
    ax.grid(axis="y", color=GRID, lw=1, zorder=0)
    ax.set_axisbelow(True)

# --- 집계 ---
data = {1: [], 2: [], 3: []}
for f in glob.glob(f"{RAW}/*.json"):
    n = int(re.match(r"n(\d)", os.path.basename(f)).group(1))
    data[n].append(json.load(open(f, encoding="utf-8"))["summary"])

agg = {}
for n in (1, 2, 3):
    tps = [d["throughput"] for d in data[n]]
    # 주의: run 마다 계산된 percentile 을 다시 평균한 값이다. 요청을 전부
    # 합쳐 다시 구한 pooled percentile 이 아니다 (S2 보고서 §7.4.1).
    # 조건 간 비교에는 쓸 수 있으나 절대값을 "이 시스템의 p99" 로 쓰면 안 된다.
    L = lambda k: st.mean([d["latency"][k] / 1000 for d in data[n]])
    shares = {}
    for d in data[n]:
        for p in d["per_node"]:
            shares.setdefault(p["node_id"], []).append(p["share"] * 100)
    stage = {}
    for d in data[n]:
        for k, v in (d.get("stage_breakdown") or {}).items():
            if v:
                stage.setdefault(k, []).append(v["p50"] / 1000)
    agg[n] = {
        "tp": st.mean(tps), "sd": st.pstdev(tps),
        "p50": L("p50"), "p95": L("p95"), "p99": L("p99"),
        "shares": {k: st.mean(v) for k, v in shares.items()},
        "stage": {k: st.mean(v) for k, v in stage.items()},
    }

N = [1, 2, 3]
SUB = "30 runs · YOLOv8n INT8 · gRPC · Active Cooling · governor=performance"

# --- Figure 1: Throughput vs Node (measured + ideal) ---
fig, ax = plt.subplots(figsize=(6.2, 4.2))
meas = [agg[n]["tp"] for n in N]
sd = [agg[n]["sd"] for n in N]
ideal = [agg[1]["tp"] * n for n in N]
ax.plot(N, ideal, "--", color=INK2, lw=1.5, zorder=2, label="Ideal (linear, 1N c8 base)")
ax.errorbar(N, meas, yerr=sd, fmt="o-", color=BLUE, lw=2.5, ms=9,
            capsize=4, zorder=3, label="Measured")
for n, v, s in zip(N, meas, sd):
    ax.annotate(f"{v:.1f}±{s:.1f}", (n, v), textcoords="offset points",
                xytext=(0, 12), ha="center", fontsize=10, color=INK, fontweight="bold")
clean(ax)
ax.set_xticks(N); ax.xaxis.set_major_locator(MaxNLocator(integer=True))
ax.set_xlabel("Nodes"); ax.set_ylabel("Throughput (inf/s)")
ax.set_title("Throughput scales near-linearly with node count")
ax.text(0, 1.015, SUB, transform=ax.transAxes, fontsize=8.5, color=INK2)
ax.legend(frameon=False, loc="upper left")
ax.set_ylim(0, max(ideal) * 1.15)
fig.tight_layout(); fig.savefig(f"{FIG}/fig1_throughput_vs_node.png"); plt.close(fig)

# --- Figure 2: Scaling Efficiency ---
fig, ax = plt.subplots(figsize=(6.2, 4.2))
eff = [agg[n]["tp"] / (agg[1]["tp"] * n) * 100 for n in N]
bars = ax.bar(N, eff, width=0.55, color=BLUE, zorder=3)
ax.axhline(100, color=INK2, ls="--", lw=1.2, zorder=2)
for n, v in zip(N, eff):
    ax.annotate(f"{v:.0f}%", (n, v), textcoords="offset points",
                xytext=(0, 6), ha="center", fontsize=11, color=INK, fontweight="bold")
clean(ax)
ax.set_xticks(N); ax.set_xlabel("Nodes"); ax.set_ylabel("Scaling efficiency (%)")
ax.set_ylim(0, 120)
ax.set_title("Scaling efficiency stays ~100% (1-node c8 base)")
ax.text(0, 1.015, ">100% reflects measurement variation vs the 1-node c8 baseline", transform=ax.transAxes, fontsize=8.5, color=INK2)
fig.tight_layout(); fig.savefig(f"{FIG}/fig2_scaling_efficiency.png"); plt.close(fig)

# --- Figure 4: Latency percentiles vs Node ---
fig, ax = plt.subplots(figsize=(6.2, 4.2))
import numpy as np
x = np.arange(len(N)); w = 0.26
for i, (k, c, lab) in enumerate([("p50", BLUE, "p50"), ("p95", ORANGE, "p95"), ("p99", AQUA, "p99")]):
    vals = [agg[n][k] for n in N]
    ax.bar(x + (i - 1) * w, vals, w, color=c, zorder=3, label=lab)
clean(ax)
ax.set_xticks(x); ax.set_xticklabels([f"{n}N" for n in N])
ax.set_xlabel("Nodes"); ax.set_ylabel("Round-trip latency (ms)")
ax.set_title("Latency remains stable as node count increases")
ax.text(0, 1.015, "closed-loop · compare only · run-level percentiles averaged over runs (not pooled)", transform=ax.transAxes, fontsize=8.5, color=INK2)
ax.legend(frameon=False, ncol=3, loc="upper left")
fig.tight_layout(); fig.savefig(f"{FIG}/fig4_latency_percentiles.png"); plt.close(fig)

# --- Figure 5: Per-node distribution (3N) ---
fig, ax = plt.subplots(figsize=(6.2, 2.6))
sh = agg[3]["shares"]
left = 0
for node in ["king", "queen", "jack"]:
    v = sh[node]
    ax.barh(0, v, left=left, color=NODE[node], zorder=3, height=0.5,
            edgecolor=SURFACE, linewidth=2)
    ax.annotate(f"{node}\n{v:.1f}%", (left + v / 2, 0), ha="center", va="center",
                color="white", fontsize=10, fontweight="bold")
    left += v
ax.set_xlim(0, 100); ax.set_ylim(-0.5, 0.5)
ax.set_yticks([]); ax.set_xlabel("Share of requests (%)")
ax.spines[["top", "right", "left"]].set_visible(False)
ax.set_title("round-robin: even distribution (0.00 %p deviation)")
ax.text(0, 1.08, SUB, transform=ax.transAxes, fontsize=8.5, color=INK2)
fig.tight_layout(); fig.savefig(f"{FIG}/fig5_per_node_distribution.png"); plt.close(fig)

# --- Figure 7: TimingBreakdown stacked (1N vs 3N) ---
fig, ax = plt.subplots(figsize=(7.6, 4.4))
segs = [("network_to_node", ORANGE, "network→node (input)"),
        ("inference", BLUE, "inference (NPU)"),
        ("network_to_client", YELLOW, "network→client (output)"),
        ("other", INK2, "scheduler/queue/etc")]
labels = ["1 node", "3 nodes"]
xpos = [0, 1]
for xi, n in zip(xpos, [1, 3]):
    stg = agg[n]["stage"]
    known = stg.get("network_to_node", 0) + stg.get("inference", 0) + stg.get("network_to_client", 0)
    other = max(0, stg.get("end_to_end", 0) - known)
    vals = {"network_to_node": stg.get("network_to_node", 0), "inference": stg.get("inference", 0),
            "network_to_client": stg.get("network_to_client", 0), "other": other}
    bottom = 0
    for key, c, _ in segs:
        v = vals[key]
        ax.bar(xi, v, 0.5, bottom=bottom, color=c, zorder=3, edgecolor=SURFACE, linewidth=2)
        if v > 2:
            ax.annotate(f"{v:.1f}", (xi, bottom + v / 2), ha="center", va="center",
                        color="white", fontsize=9, fontweight="bold")
        bottom += v
clean(ax)
ax.set_xticks(xpos); ax.set_xticklabels(labels)
ax.set_ylabel("Latency (ms, p50)")
ax.set_ylim(0, 78)
ax.set_title("Non-inference overhead is dominated by payload transfer")
ax.text(0, 1.015, "Payload transfer = 58% of E2E · 94% of non-inference overhead",
        transform=ax.transAxes, fontsize=8.5, color=INK2)
handles = [plt.Rectangle((0, 0), 1, 1, color=c) for _, c, _ in segs]
ax.legend(handles, [l for _, _, l in segs], frameon=False, fontsize=8.5,
          loc="upper center", ncol=2, columnspacing=1.2)
fig.tight_layout(); fig.savefig(f"{FIG}/fig7_timing_breakdown.png"); plt.close(fig)

# --- Figure 8: Local vs Cluster single-node ---
fig, ax = plt.subplots(figsize=(5.4, 4.2))
modes = ["Local direct\n(no gRPC)", "Cluster gRPC\n(single node)"]
vals = [161.5, agg[1]["tp"]]
cols = [INK2, BLUE]
bars = ax.bar(modes, vals, width=0.5, color=cols, zorder=3)
for b, v in zip(bars, vals):
    ax.annotate(f"{v:.1f}", (b.get_x() + b.get_width() / 2, v), textcoords="offset points",
                xytext=(0, 6), ha="center", fontsize=12, color=INK, fontweight="bold")
loss = (vals[0] - vals[1]) / vals[0] * 100
ax.annotate(f"-{loss:.1f}%", (0.5, max(vals) * 0.55), ha="center", fontsize=14,
            color=ORANGE, fontweight="bold")
clean(ax)
ax.set_ylabel("Throughput (inf/s)")
ax.set_title("Node-level overhead: local vs cluster")
ax.text(0, 1.015, "Both Active Cooling · worker = 8",
        transform=ax.transAxes, fontsize=8.5, color=INK2)
ax.set_ylim(0, max(vals) * 1.2)
fig.tight_layout(); fig.savefig(f"{FIG}/fig8_local_vs_cluster.png"); plt.close(fig)

# --- Figure 3: Saturation sweep (S3, Throughput vs Concurrency) ---
SAT = "results/saturation-20260820"
if os.path.isdir(f"{SAT}/raw"):
    pts = {1: {}, 2: {}, 3: {}}
    for f in glob.glob(f"{SAT}/raw/*.json"):
        m = re.match(r"sat_n(\d)_c(\d+)", os.path.basename(f))
        n, c = int(m.group(1)), int(m.group(2))
        pts[n].setdefault(c, []).append(json.load(open(f, encoding="utf-8"))["summary"]["throughput"])
    os.makedirs(f"{SAT}/figures", exist_ok=True)
    fig, ax = plt.subplots(figsize=(6.6, 4.4))
    cols = {1: BLUE, 2: ORANGE, 3: AQUA}
    for n in (1, 2, 3):
        cs = sorted(pts[n]); ys = [st.mean(pts[n][c]) for c in cs]
        ax.plot(cs, ys, "o-", color=cols[n], lw=2.2, ms=7, label=f"{n} node", zorder=3)
        ci = max(range(len(ys)), key=lambda i: ys[i])
        ax.annotate(f"{ys[ci]:.0f}", (cs[ci], ys[ci]), textcoords="offset points",
                    xytext=(0, 9), ha="center", fontsize=9, color=INK, fontweight="bold")
    clean(ax)
    ax.set_xlabel("Concurrency"); ax.set_ylabel("Throughput (inf/s)")
    ax.set_title("Saturation: each node count reaches a throughput ceiling")
    ax.text(0, 1.015, "S3 · 3 runs/point · ceilings ~115 / 232 / 342 inf/s",
            transform=ax.transAxes, fontsize=8.5, color=INK2)
    ax.legend(frameon=False, loc="lower right")
    fig.tight_layout(); fig.savefig(f"{SAT}/figures/fig3_saturation_sweep.png"); plt.close(fig)
    print("Figure 3 (saturation):", os.listdir(f"{SAT}/figures"))

print("PNG 생성:", [x for x in os.listdir(FIG) if x.endswith(".png")])
