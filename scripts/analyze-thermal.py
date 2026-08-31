#!/usr/bin/env python3
"""열 비교 run 의 로그를 요약한다.

평탄역만 잘라 비교한다. 상승 구간을 포함하면 부하 시작 시각이 몇 초만
어긋나도 평균이 흔들린다.

사용법: python scripts/analyze-thermal.py results/thermal-<타임스탬프>
"""

import sys
import pathlib
import statistics as st

BOARDS = ["king", "queen", "jack"]
# 부하 시작 후 이 시간이 지난 뒤부터 평탄역으로 본다.
PLATEAU_START_S = 300


def load(path):
    rows = []
    with open(path, encoding="utf-8") as f:
        header = f.readline().split()
        for line in f:
            p = line.split()
            if len(p) != len(header):
                continue
            try:
                rows.append({k: float(v) for k, v in zip(header, p)})
            except ValueError:
                continue
    return rows


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    d = pathlib.Path(sys.argv[1])

    data = {}
    for b in BOARDS:
        p = d / f"{b}-temp.log"
        if not p.exists():
            print(f"  {b}: 로그 없음")
            continue
        data[b] = load(p)

    if not data:
        print("분석할 데이터가 없다")
        return 1

    # 각 보드의 부하 시작 시점을 온도 상승으로 추정하지 않는다.
    # 로거를 부하보다 먼저 띄웠으므로 로그 시작 + 고정 오프셋을 쓴다.
    print(f"평탄역 정의: 로그 시작 후 {PLATEAU_START_S}초 ~ 부하 종료")
    print()

    summary = {}
    for b, rows in data.items():
        t0 = rows[0]["epoch"]
        # 하강 구간(부하 종료 후)을 빼기 위해 NPU 온도가 최고점 대비
        # 15°C 이상 떨어진 지점 이후는 버린다.
        peak = max(r["npu_c"] for r in rows)
        plateau = []
        for r in rows:
            el = r["epoch"] - t0
            if el < PLATEAU_START_S:
                continue
            if r["npu_c"] < peak - 15:
                break
            plateau.append(r)

        if not plateau:
            print(f"  {b}: 평탄역 샘플 없음")
            continue

        npu = [r["npu_c"] for r in plateau]
        soc = [r["soc_c"] for r in plateau]
        volt = [r["volt_v"] for r in plateau if r["volt_v"] > 0]
        cpu = [r["cpu_mhz"] for r in plateau if r["cpu_mhz"] > 0]
        npuf = [r["npu_mhz"] for r in plateau if r["npu_mhz"] > 0]

        # CPU 강등 여부. NPU 클럭만 보면 "throttling 없음"으로 오판한다.
        # 실제로 그렇게 오판했다 — discuss.md §12.
        cpu_max = max(cpu) if cpu else 0
        cpu_min = min(cpu) if cpu else 0

        summary[b] = {
            "cpu_max": cpu_max,
            "cpu_min": cpu_min,
            "n": len(plateau),
            "npu_mean": st.mean(npu),
            "npu_max": max(npu),
            "soc_mean": st.mean(soc),
            "volt_min": min(volt) if volt else 0,
            "cpu_mean": st.mean(cpu) if cpu else 0,
            "npu_mhz_min": min(npuf) if npuf else 0,
            "npu_mhz_mean": st.mean(npuf) if npuf else 0,
        }

    print(f"{'보드':<7}{'샘플':>5}{'NPU평균':>9}{'NPU최대':>9}{'SoC평균':>9}"
          f"{'전압최저':>10}{'CPU평균':>9}{'CPU최저':>9}{'NPU클럭':>9}")
    for b in BOARDS:
        if b not in summary:
            continue
        s = summary[b]
        print(f"{b:<7}{s['n']:>5}{s['npu_mean']:>9.1f}{s['npu_max']:>9.1f}"
              f"{s['soc_mean']:>9.1f}{s['volt_min']:>10.3f}"
              f"{s['cpu_mean']:>9.0f}{s['cpu_min']:>9.0f}{s['npu_mhz_mean']:>9.0f}")

    if len(summary) >= 2:
        means = {b: s["npu_mean"] for b, s in summary.items()}
        maxs = {b: s["npu_max"] for b, s in summary.items()}
        hi, lo = max(means, key=means.get), min(means, key=means.get)
        print()
        print(f"평균 최대 편차: {means[hi] - means[lo]:.1f}°C  ({hi} - {lo})")
        print(f"최대치 편차   : {max(maxs.values()) - min(maxs.values()):.1f}°C")
        print(f"최고 온도     : {max(maxs.values()):.1f}°C "
              f"({max(maxs, key=maxs.get)})")

        over90 = [b for b, v in maxs.items() if v >= 90.0]
        print(f"90°C 초과     : {over90 if over90 else '없음'}")

        # CPU 강등 판정. NPU 클럭만 보면 "throttling 없음"으로 오판한다.
        print()
        print("CPU 강등 (최대 → 최저)")
        for b in BOARDS:
            if b not in summary:
                continue
            s2 = summary[b]
            hi, lo = s2["cpu_max"], s2["cpu_min"]
            drop = (1 - lo / hi) * 100 if hi else 0
            mark = "  ← 강등" if drop >= 20 else ""
            print(f"  {b:<7}{hi:>6.0f} → {lo:>6.0f} MHz  (-{drop:.0f}%){mark}")
        print()
        print("NPU 클럭이 유지되어도 CPU 가 꺾이면 처리량은 떨어진다.")
        print("추론 한 건은 입력 설정(CPU) → NPU → 출력 취득(CPU) 이다.")
        print("discuss.md §12 참조.")

    # 처리량
    print()
    print("처리량 (load.log 요약)")
    for b in BOARDS:
        p = d / f"{b}-load.log"
        if not p.exists():
            continue
        txt = p.read_text(encoding="utf-8", errors="replace")
        vals = {}
        for line in txt.splitlines():
            for key in ("처리량", "총 추론", "평균 지연", "오류"):
                if line.startswith(key):
                    vals[key] = line.split(":", 1)[1].strip()
        if vals:
            print(f"  {b:<6} {vals.get('처리량', '?'):<12} "
                  f"총 {vals.get('총 추론', '?'):<8} "
                  f"지연 {vals.get('평균 지연', '?'):<10} "
                  f"오류 {vals.get('오류', '?')}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
