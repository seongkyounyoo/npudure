#!/usr/bin/env python3
"""그림이 없던 실험들의 발표용 그림.

S2 · S3 · S3.7 만 그림이 있었다(`make-figures.py`, `make-sweep-figures.py`).
이 스크립트가 나머지를 만든다.

  S3.6   fig_h2_window_vs_conns.png       flow control 이 아니라 커넥션이다
  S3.8   fig_scaleout_optimized.png       절대값은 올랐고 efficiency 는 내렸다
  S3.9a  fig_efficiency_loss_is_tail.png  p50 은 평평, p99 만 오른다
  S3.9b  fig_transport_cost_split.png     io_uring 이 닿는 몫은 이만큼이다
  S0-A/B fig_sustained_thermal.png        냉각이 sustained 운영점을 가른다
  S0-C   fig_policy_tail.png              RR 은 이질에서 tail 이 불안정하다
  S0-D   fig_capacity_calibration.png     이질을 다이얼로 만든다

규칙 (기존 스크립트와 동일하게 지킨다)
  - 라벨은 **영문**. 한글 폰트가 없는 환경에서 tofu 가 되는 것을 피한다.
    설명은 문서가 한다.
  - 팔레트는 검증된 default(light) 그대로.
  - **dual-axis 를 쓰지 않는다.** 축이 둘이면 "오른쪽 위가 좋다" 로 잘못
    읽힌다. 대신 패널을 나누거나 좌표로 쓴다(`make-sweep-figures.py` 주석).
  - 캡션 한 줄에 **측정 조건**을 적는다. 조건 없는 숫자는 3개월 뒤 쓸모없다.

사용법
    python scripts/make-experiment-figures.py            전부
    python scripts/make-experiment-figures.py s39b s0d   일부만
"""
import csv
import glob
import io
import json
import os
import statistics as st
import sys

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# --- 검증된 default 팔레트 (light) ---
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


def cap(ax, text):
    """측정 조건 캡션. 제목 바로 아래."""
    ax.text(0, 1.015, text, transform=ax.transAxes, fontsize=8.5, color=INK2)


def save(fig, resdir, name):
    d = os.path.join(ROOT, "results", resdir, "figures")
    os.makedirs(d, exist_ok=True)
    p = os.path.join(d, name)
    fig.savefig(p, bbox_inches="tight")
    plt.close(fig)
    print("  " + os.path.relpath(p, ROOT).replace(os.sep, "/"))


def rows(resdir):
    f = os.path.join(ROOT, "results", resdir, "raw", "results.csv")
    return [r for r in csv.DictReader(io.open(f, encoding="utf-8")) if r.get("throughput")]


def agg(rs, key, field):
    """key 별 field 의 (평균, 표준편차)."""
    g = {}
    for r in rs:
        g.setdefault(r[key], []).append(float(r[field]))
    return {k: (st.mean(v), st.stdev(v) if len(v) > 1 else 0.0) for k, v in g.items()}


# ── S3.6 — flow control 이 아니라 커넥션이다 ─────────────────────────
def fig_s36():
    rs = rows("h2-channel-ab-20260820")
    a = agg(rs, "cond", "throughput")
    order = ["A_1ch_default", "B_1ch_bigwin", "C_4ch_default", "D_4ch_bigwin"]
    order = [c for c in order if c in a]
    labels = ["1 conn\ndefault window", "1 conn\n64MB window",
              "4 conns\ndefault window", "4 conns\n64MB window"]
    vals = [a[c][0] for c in order]
    errs = [a[c][1] for c in order]
    cols = [INK2, ORANGE, BLUE, BLUE]

    fig, ax = plt.subplots(figsize=(8.2, 5))
    b = ax.bar(range(len(vals)), vals, yerr=errs, capsize=4, color=cols, zorder=3, width=.62)
    for i, v in enumerate(vals):
        ax.annotate(f"{v:.1f}", (i, v), textcoords="offset points",
                    xytext=(0, 10), ha="center", fontsize=11, color=INK, fontweight="bold")
    base = vals[0]
    for i in (1, 2):
        ax.annotate(f"{(vals[i] / base - 1) * 100:+.1f}%", (i, vals[i] * .5), ha="center",
                    fontsize=13, color="white", fontweight="bold")
    ax.set_xticks(range(len(vals)))
    ax.set_xticklabels(labels[:len(vals)])
    ax.set_ylabel("Throughput (inf/s)")
    ax.set_title("Connections help; enlarging the HTTP/2 window hurts")
    cap(ax, "S3.6 · 1 node · c32 · 5 runs per condition · error bars = SD")
    clean(ax)
    save(fig, "h2-channel-ab-20260820", "fig_h2_window_vs_conns.png")


# ── S3.8 — 절대값은 올랐고 efficiency 는 내렸다 ──────────────────────
def fig_s38():
    rs = rows("scaleout-optimized-20260820")
    # 각 노드 수의 운영점(노드당 c12)만 쓴다.
    sel = [r for r in rs if int(r["concurrency"]) == 12 * int(r["nodes"])]
    a = agg(sel, "nodes", "throughput")
    ns = sorted(a, key=int)
    opt = [a[n][0] for n in ns]
    err = [a[n][1] for n in ns]
    base = [115.2, 232.0, 341.8][:len(ns)]   # S3 ceiling (conn1)

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12.4, 5))
    x = range(len(ns))
    w = .36
    ax1.bar([i - w / 2 for i in x], base, w, color=INK2, label="baseline (1 conn/node)", zorder=3)
    ax1.bar([i + w / 2 for i in x], opt, w, yerr=err, capsize=4, color=BLUE,
            label="optimized (2 conns/node)", zorder=3)
    for i in x:
        ax1.annotate(f"+{(opt[i] / base[i] - 1) * 100:.1f}%", (i + w / 2, opt[i]),
                     textcoords="offset points", xytext=(0, 10), ha="center",
                     fontsize=10.5, color=INK, fontweight="bold")
    ax1.set_xticks(list(x)); ax1.set_xticklabels([f"{n} node" + ("s" if int(n) > 1 else "") for n in ns])
    ax1.set_ylabel("Throughput (inf/s)")
    ax1.set_title("Absolute throughput rises at every scale")
    cap(ax1, "S3.8 · operating point c12/node · 3 runs · error bars = SD")
    ax1.legend(frameon=False, fontsize=9.5, loc="upper left")
    clean(ax1)

    eff_b = [base[i] / base[0] / (i + 1) * 100 for i in range(len(ns))]
    eff_o = [opt[i] / opt[0] / (i + 1) * 100 for i in range(len(ns))]
    ax2.plot(x, eff_b, "o-", color=INK2, lw=2, ms=7, label="baseline")
    ax2.plot(x, eff_o, "o-", color=ORANGE, lw=2.4, ms=8, label="optimized")
    for i in x:
        ax2.annotate(f"{eff_o[i]:.1f}%", (i, eff_o[i]), textcoords="offset points",
                     xytext=(0, -18), ha="center", fontsize=10, color=ORANGE, fontweight="bold")
    ax2.axhline(100, color=GRID, lw=1.2, ls="--", zorder=1)
    ax2.set_xticks(list(x)); ax2.set_xticklabels([f"{n}N" for n in ns])
    ax2.set_ylabel("Scaling efficiency (%)")
    ax2.set_ylim(88, 104)
    ax2.set_title("...but efficiency drops: 98.9% → 95.3%")
    cap(ax2, "baseline >100% at 2N reflects run-to-run variation vs its own 1N reference")
    ax2.legend(frameon=False, fontsize=9.5, loc="lower left")
    clean(ax2)
    save(fig, "scaleout-optimized-20260820", "fig_scaleout_optimized.png")


# ── S3.9a — p50 은 평평, p99 만 오른다 ───────────────────────────────
def fig_s39a():
    rs = rows("scaleout-profile-20260821")
    ns = sorted({r["nodes"] for r in rs}, key=int)
    series = [("p50_ms", "p50", AQUA), ("p95_ms", "p95", YELLOW), ("p99_ms", "p99", ORANGE)]

    fig, ax = plt.subplots(figsize=(8.4, 5.2))
    x = range(len(ns))
    for f, lab, c in series:
        a = agg(rs, "nodes", f)
        v = [a[n][0] for n in ns]
        ax.plot(x, v, "o-", color=c, lw=2.4, ms=8, label=lab, zorder=3)
        ax.annotate(f"{(v[-1] / v[0] - 1) * 100:+.0f}%", (x[-1], v[-1]),
                    textcoords="offset points", xytext=(12, -3), fontsize=11,
                    color=c, fontweight="bold")
    ax.set_xticks(list(x)); ax.set_xticklabels([f"{n} node" + ("s" if int(n) > 1 else "") for n in ns])
    ax.set_ylabel("Latency (ms)")
    ax.set_xlim(-.25, len(ns) - .55)
    ax.set_title("The efficiency loss is entirely in the tail")
    cap(ax, "S3.9a · operating point c12/node · 3 runs · server resources ruled out · "
            "run-level percentiles averaged (not pooled)")
    # 기본 'best' 는 오른쪽 위를 골라 p99 증가율 라벨을 덮는다.
    ax.legend(frameon=False, fontsize=10, loc="upper left")
    clean(ax)
    save(fig, "scaleout-profile-20260821", "fig_efficiency_loss_is_tail.png")


# ── S3.9b — io_uring 이 닿는 몫 ──────────────────────────────────────
def fig_s39b():
    HZ = 100
    base = os.path.join(ROOT, "results", "node-residual-20260821", "raw")

    def pstat(p):
        s = io.open(p, encoding="utf-8", errors="replace").read()
        r = s[s.rindex(")") + 1:].split()
        return int(r[11]), int(r[12])

    def cond(label, tp):
        d = os.path.join(base, label)
        u0, s0 = pstat(os.path.join(d, "pstat.before"))
        u1, s1 = pstat(os.path.join(d, "pstat.after"))
        w = (float(io.open(os.path.join(d, "uptime.after")).read().split()[0])
             - float(io.open(os.path.join(d, "uptime.before")).read().split()[0]))
        n = w * tp
        return (u1 - u0) * 1000.0 / HZ / n, (s1 - s0) * 1000.0 / HZ / n

    lu, ls = cond("local", 157.9)
    ou, os_ = cond("op", 136.6)
    du, ds = ou - lu, os_ - ls

    fig, ax = plt.subplots(figsize=(10.6, 5.4))
    xs = [0, 1, 2]
    U = [lu, ou, du]
    S = [ls, os_, ds]
    ax.bar(xs, U, .5, color=BLUE, label="user  (serialization, user-space copy, HTTP/2)", zorder=3)
    ax.bar(xs, S, .5, bottom=U, color=ORANGE,
           label="kernel  (syscall entry, TCP stack, copy_to_user)", zorder=3)
    for i in xs:
        ax.annotate(f"{U[i] + S[i]:.2f}", (i, U[i] + S[i]), textcoords="offset points",
                    xytext=(0, 8), ha="center", fontsize=11, color=INK, fontweight="bold")
        ax.annotate(f"{U[i]:.2f}", (i, U[i] / 2), ha="center", va="center",
                    fontsize=10, color="white", fontweight="bold")
        ax.annotate(f"{S[i]:.2f}", (i, U[i] + S[i] / 2), ha="center", va="center",
                    fontsize=10, color="white", fontweight="bold")
    # io_uring 이 닿는 몫: syscall 진입(~165 calls x 1us) + 1.2MB copy 양방향 가정.
    # 화살표로 그리면 1.4ms 가 점처럼 보여 "구간" 으로 안 읽힌다.
    # transport 막대 위쪽에 **띠**로 겹쳐 그리고 지시선을 뺀다.
    reach = 0.165 + 1.2
    top = du + ds
    ax.bar([2], [reach], .5, bottom=top - reach, color=AQUA, zorder=4)
    ax.plot([2.25, 2.95], [top - reach / 2, top - reach / 2], color=AQUA, lw=1.4, zorder=4)
    ax.text(3.02, top - reach / 2,
            f"io_uring reaches only this\n"
            f"≈{reach:.1f} ms = {reach / top * 100:.0f}% of transport cost\n"
            f"syscall entry {0.165:.2f} + assumed 1.2 MB copy",
            fontsize=9.5, color=INK, va="center", fontweight="bold")
    ax.set_xticks(xs)
    ax.set_xticklabels(["local direct\nno network", "operating point\nc12 · 2 conns",
                        "transport cost\ndifference"])
    ax.set_ylabel("Node CPU per request (ms)")
    ax.set_xlim(-.6, 4.75)
    ax.set_ylim(0, 30)
    ax.set_title("What io_uring could reach is a sliver — and CPU is not the constraint")
    cap(ax, "S3.9b · 1 node · 45 s window · board CPU 48.9% idle, no core saturated")
    ax.legend(frameon=False, fontsize=9.5, loc="upper right", bbox_to_anchor=(1.0, .80))
    clean(ax)
    save(fig, "node-residual-20260821", "fig_transport_cost_split.png")


# ── S0-A/B — 냉각이 sustained 운영점을 가른다 ────────────────────────
def fig_s0ab():
    def series(cond):
        fs = sorted(glob.glob(os.path.join(ROOT, "results",
                                           f"sustained-20260821-{cond}", "raw", "*.json")))
        return [json.load(io.open(f, encoding="utf-8"))["summary"]["throughput"] for f in fs]

    fan, fanless = series("fan"), series("fanless")
    fig, ax = plt.subplots(figsize=(9.2, 5.2))
    # 라벨을 선 위/아래로 갈라 놓는다. 같은 쪽에 두면 곡선을 가로지른다.
    for v, c, lab, dy in ((fan, BLUE, "active cooling", 16), (fanless, ORANGE, "fanless", -26)):
        x = [i + 1 for i in range(len(v))]
        ax.plot(x, v, "-", color=c, lw=2.2, label=lab, zorder=3)
        steady = st.mean(v[-len(v) // 3:])
        ax.annotate(f"steady {steady:.1f}   −{(1 - steady / max(v)) * 100:.1f}%",
                    (x[-1], steady), textcoords="offset points", xytext=(-8, dy),
                    ha="right", fontsize=10.5, color=c, fontweight="bold")
    ax.set_xlabel("Run # (60 s each, ~31 min continuous)")
    ax.set_ylabel("Throughput (inf/s)")
    ax.set_title("Cooling decides whether the short-run operating point survives")
    cap(ax, "S0-A/B · 3 nodes · c36 · 2 conns/node · 30 runs each · no node ever excluded, 0 errors")
    ax.legend(frameon=False, fontsize=10)
    clean(ax)
    save(fig, "sustained-20260821-fanless", "fig_sustained_thermal.png")


# ── S0-C — RR 은 이질에서 tail 이 불안정하다 ─────────────────────────
def fig_s0c():
    conds = [("policy-ab-20260821b", "fanless (heterogeneous)"),
             ("policy-ab-20260821fan", "active cooling (homogeneous)")]
    pols = ["round-robin", "least-queue", "ect"]
    cols = [INK2, AQUA, BLUE]

    fig, axes = plt.subplots(1, 2, figsize=(12.4, 5), sharey=True)
    for ax, (d, title) in zip(axes, conds):
        a = agg(rows(d), "policy", "p99_ms")
        v = [a[p][0] for p in pols]
        e = [a[p][1] for p in pols]
        ax.bar(range(3), v, .58, yerr=e, capsize=5, color=cols, zorder=3)
        for i, (m, s) in enumerate(zip(v, e)):
            # 오차막대 **위**에 붙인다. 평균 기준으로 두면 SD 가 큰 막대에서 겹친다.
            ax.annotate(f"{m:.1f}\n±{s:.1f}", (i, m + s), textcoords="offset points",
                        xytext=(0, 10), ha="center", fontsize=10.5,
                        color=ORANGE if s > 10 else INK, fontweight="bold")
        ax.set_xticks(range(3)); ax.set_xticklabels(["round-robin", "least-queue", "ect"])
        ax.set_title(title)
        clean(ax)
    axes[0].set_ylabel("p99 latency (ms)")
    axes[0].set_ylim(0, 300)
    cap(axes[0], "S0-C · 3 nodes · c36 · 4 rounds per policy · error bars = SD across rounds")
    cap(axes[1], "round-robin's SD explodes only under heterogeneity")
    fig.suptitle("Round-robin's tail is unpredictable when nodes differ",
                 fontsize=14, fontweight="bold", y=1.02)
    save(fig, "policy-ab-20260821b", "fig_policy_tail.png")


# ── S0-D — 이질을 다이얼로 만든다 ────────────────────────────────────
def fig_s0d():
    rs = rows("capacity-calib-20260821")
    caps = sorted({int(r["cap_mhz"]) for r in rs}, reverse=True)
    key = "cap_mhz"

    def mean_by(field):
        a = agg(rs, key, field)
        return [a[str(c)][0] for c in caps]

    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(8.8, 8.2), sharex=True)
    x = range(len(caps))
    for f, lab, c in (("king_p50", "king (capped)", ORANGE),
                      ("jack_p50", "jack", BLUE), ("queen_p50", "queen", AQUA)):
        ax1.plot(x, mean_by(f), "o-", color=c, lw=2.3, ms=7, label=lab, zorder=3)
    ax1.set_ylabel("Per-node p50 (ms)")
    ax1.set_title("Capping one node's CPU makes the other two idle")
    cap(ax1, "S0-D · fan ON (thermal removed) · 3 nodes · c36 · round-robin · 2 runs per cap")
    ax1.legend(frameon=False, fontsize=10)
    clean(ax1)

    sp = mean_by("spread")
    ax2.plot(x, sp, "o-", color=INK, lw=2.6, ms=8, zorder=3)
    ax2.axhline(2.4, color=ORANGE, lw=1.6, ls="--", zorder=2)
    # 오른쪽에 두면 816 의 2.26x 라벨과 겹친다.
    ax2.text(0.05, 2.46, "S0-A (thermal) = 2.4×", fontsize=9.5,
             color=ORANGE, ha="left", fontweight="bold")
    for i, v in enumerate(sp):
        ax2.annotate(f"{v:.2f}×", (i, v), textcoords="offset points", xytext=(0, 10),
                     ha="center", fontsize=10, color=INK, fontweight="bold")
    ax2.set_xticks(list(x)); ax2.set_xticklabels([f"{c}" for c in caps])
    ax2.set_xlabel("king CPU cap (MHz)   ←  more heterogeneous")
    ax2.set_ylabel("Spread  (max/min node p50)")
    ax2.set_title("Heterogeneity becomes a dial: 816 MHz reproduces S0-A")
    cap(ax2, "deterministic, repeatable, no 30-minute preheat and no silicon luck")
    clean(ax2)
    save(fig, "capacity-calib-20260821", "fig_capacity_calibration.png")


FIGS = {"s36": fig_s36, "s38": fig_s38, "s39a": fig_s39a, "s39b": fig_s39b,
        "s0ab": fig_s0ab, "s0c": fig_s0c, "s0d": fig_s0d}

if __name__ == "__main__":
    want = sys.argv[1:] or list(FIGS)
    for k in want:
        if k not in FIGS:
            print(f"  ! 알 수 없는 그림: {k}  (가능: {', '.join(FIGS)})")
            continue
        print(f"[{k}]")
        FIGS[k]()
