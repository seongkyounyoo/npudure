#!/usr/bin/env python3
"""S0 — 지속 부하 분석. short-run 과 sustained 운영점이 같은가.

묻는 것
    1. 처리량이 시간에 따라 꺾이는가. 꺾이면 **언제**.
    2. 꺾임이 열/주파수와 대응하는가 (CPU 강등? NPU 강등?).
    3. sustained 처리량은 short-run 대비 몇 %인가.

정상 상태 정의를 규칙으로 못 박는다 — 결과에 맞춰 고르지 않기 위해서다.

    steady-state = 마지막 1/3 구간의 평균
    ramp         = 첫 run
    degradation  = 1 - steady / peak

사용법
    python scripts/analyze-sustained.py <결과디렉터리>
"""

import csv
import pathlib
import statistics as st
import sys

ROOT = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else ".")
RAW = ROOT / "raw"
NL = chr(10)

rows = [r for r in csv.DictReader(open(RAW / "throughput.csv", encoding="utf-8"))
        if r.get("throughput")]
if not rows:
    print("유효한 run 이 없다.")
    sys.exit(1)

tp = [float(r["throughput"]) for r in rows]
el = [int(r["elapsed_s"]) for r in rows]
n = len(tp)

peak = max(tp)
first = tp[0]
tail_n = max(1, n // 3)
steady = st.mean(tp[-tail_n:])
steady_sd = st.stdev(tp[-tail_n:]) if tail_n > 1 else 0.0

W = 92
print("=" * W)
print(f"S0  지속 부하 — {ROOT.name}")
print("=" * W)
print(f"  run {n}개 × 60초 = {el[-1] / 60:.1f}분 연속")

# ── 시계열 ────────────────────────────────────────────────────────
print(f"{NL}{'run':>4}{'t+분':>7}{'throughput':>12}{'vs peak':>10}"
      f"{'p50':>8}{'p95':>8}{'p99':>8}{'err':>8}")
print("-" * W)
for i, r in enumerate(rows):
    t = float(r["throughput"])
    # 처음 3개, 마지막 3개, 그리고 5의 배수만 찍는다 (30개를 다 찍으면 안 읽힌다)
    if i < 3 or i >= n - 3 or (i + 1) % 5 == 0:
        print(f"{r['run']:>4}{int(r['elapsed_s']) / 60:>7.1f}{t:>12.1f}"
              f"{t / peak:>9.1%}{float(r['p50_ms']):>8.1f}"
              f"{float(r['p95_ms']):>8.1f}{float(r['p99_ms']):>8.1f}"
              f"{float(r['error_rate']):>8.4f}")
    elif i == 3:
        print(f"{'...':>4}")

err = sum(float(r["error_rate"]) for r in rows)
print(f"{NL}  총 오류율 합계: {err:.4f}")

# ── 판정 ──────────────────────────────────────────────────────────
print(NL + "─" * W)
print("정상 상태 판정 (규칙: steady = 마지막 1/3 평균)")
print("─" * W)
deg = 1 - steady / peak
print(f"  첫 run          {first:7.1f} inf/s")
print(f"  peak            {peak:7.1f} inf/s  (run {tp.index(peak) + 1})")
print(f"  steady (뒤 1/3) {steady:7.1f} ± {steady_sd:.1f} inf/s")
print(f"  **degradation   {deg:7.1%}**  (peak → steady)")

# ── 열·주파수 ─────────────────────────────────────────────────────
TH = RAW / "thermal"
if TH.exists():
    print(NL + "─" * W)
    print("열·주파수 (보드별, 1초 샘플)")
    print("─" * W)
    print(f"{'board':>7}{'soc시작':>9}{'soc최대':>9}{'npu시작':>9}{'npu최대':>9}"
          f"{'cpuMHz시작':>12}{'cpuMHz최소':>12}{'npuMHz최소':>12}")
    print("-" * W)
    for f in sorted(TH.glob("*.log")):
        try:
            lines = f.read_text(encoding="utf-8", errors="replace").strip().splitlines()
            hdr, data = lines[0].split(), [l.split() for l in lines[1:] if l.strip()]
            if not data:
                continue
            col = {k: i for i, k in enumerate(hdr)}

            def series(k):
                return [float(d[col[k]]) for d in data
                        if len(d) > col[k] and d[col[k]] not in ("", "0")]
            soc, npu = series("soc_c"), series("npu_c")
            cpu, nf = series("cpu_mhz"), series("npu_mhz")
            print(f"{f.stem:>7}{soc[0]:>8.1f}C{max(soc):>8.1f}C"
                  f"{npu[0]:>8.1f}C{max(npu):>8.1f}C"
                  f"{cpu[0]:>11.0f}{min(cpu):>12.0f}{min(nf) if nf else 0:>12.0f}")
        except Exception as e:
            print(f"  {f.stem}: 파싱 실패 ({e})")

    print(f"{NL}  cpuMHz 최소가 시작값보다 낮으면 CPU 강등이 일어난 것이다.")
    print("  npuMHz 최소도 같은 방식으로 본다 (RK3576 NPU 는 300~950MHz).")

# ── 해석 ──────────────────────────────────────────────────────────
print(NL + "─" * W)
print("해석")
print("─" * W)
if deg < 0.03:
    print(f"  → **열화 없음** ({deg:.1%}). short-run 운영점이 지속 부하에서도 유지된다.")
    print("     지금까지의 60초 결과가 그대로 유효하다 — 적용 범위가 넓어졌다.")
elif deg < 0.10:
    print(f"  → **경미한 열화** ({deg:.1%}). sustained 운영점이 short-run 보다 낮다.")
    print("     두 운영점을 나란히 제시해야 한다.")
else:
    print(f"  → **뚜렷한 열화** ({deg:.1%}). short-run 결과를 지속 운전에 인용하면 안 된다.")
    print("     sustained operating point 를 별도로 확정해야 하고, 이 항이")
    print("     transport 미세 최적화(syscall/copy)보다 훨씬 크다.")
print()
print("  주의: 60초 run 을 이어 붙였으므로 run 사이 2~4초 공백이 있다.")
print("        열 시상수보다 훨씬 짧아 열 상태를 되돌리지는 않는다.")
