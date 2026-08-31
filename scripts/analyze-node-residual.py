#!/usr/bin/env python3
"""S3.9b — node-side residual cost 분석.

답하려는 것
    161.5 -> 135.5 residual gap 에서 node-side serialization / copy /
    syscall 비용이 유의미한 비중을 차지하는가?

핵심 분해
    utime  유저 시간 — protobuf 직렬화, 유저공간 copy, HTTP/2 프레이밍
    stime  커널 시간 — syscall 진입, TCP 스택, copy_to_user, skb, 드라이버

    io_uring 이 줄이는 것은 **stime 의 일부**다(syscall 진입 + 등록
    버퍼로 절약 가능한 copy). TCP 스택 작업은 그대로 남는다. 따라서
    stime 전체가 io_uring 의 상한이고, 실제 회수 가능분은 그보다 작다.

사용법
    python scripts/analyze-node-residual.py <결과디렉터리>
"""

import io
import pathlib
import re
import sys

HZ = 100  # aarch64 USER_HZ
ROOT = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else ".")
RAW = ROOT / "raw"
NL = chr(10)

# 비교 기준 (experiments/README §3). 분모를 명시해 재인용 사고를 막는다.
LOCAL_DIRECT = 161.5
OPERATING = 135.5


def pstat(p):
    """/proc/PID/stat -> (utime, stime) in ticks. comm 에 공백이 있을 수 있다."""
    s = io.open(p, encoding="utf-8", errors="replace").read()
    rest = s[s.rindex(")") + 1:].split()
    return int(rest[11]), int(rest[12])


def uptime(p):
    return float(io.open(p).read().split()[0])


def pio(p):
    d = {}
    for ln in io.open(p, encoding="utf-8", errors="replace"):
        if ":" in ln:
            k, v = ln.split(":", 1)
            try:
                d[k.strip()] = int(v.strip())
            except ValueError:
                pass
    return d


def cond(label):
    """조건 하나를 읽어 델타를 낸다. 없으면 None."""
    d = RAW / label
    need = ["pstat.before", "pstat.after", "uptime.before", "uptime.after"]
    if not all((d / f).exists() for f in need):
        return None
    u0, s0 = pstat(d / "pstat.before")
    u1, s1 = pstat(d / "pstat.after")
    w = uptime(d / "uptime.after") - uptime(d / "uptime.before")
    out = {
        "wall": w,
        "utime_ms": (u1 - u0) * 1000.0 / HZ,
        "stime_ms": (s1 - s0) * 1000.0 / HZ,
    }
    out["cpu_ms"] = out["utime_ms"] + out["stime_ms"]
    if (d / "pio.before").exists() and (d / "pio.after").exists():
        a, b = pio(d / "pio.before"), pio(d / "pio.after")
        out["syscr"] = b.get("syscr", 0) - a.get("syscr", 0)
        out["syscw"] = b.get("syscw", 0) - a.get("syscw", 0)
        out["rbytes"] = b.get("read_bytes", 0) - a.get("read_bytes", 0)
    return out


W = 92
print("=" * W)
print("S3.9b  node-side residual cost  (운영점 c12 · conn2 · 1노드)")
print("=" * W)
print("  질문: residual gap 에서 node-side serialization/copy/syscall 이")
print("        유의미한 비중을 차지하는가. gap 전체를 설명하는 것이 목적이 아니다.")

gap = LOCAL_DIRECT - OPERATING
print(f"{NL}  로컬 direct {LOCAL_DIRECT}  운영점 {OPERATING}  "
      f"gap {gap:.1f} inf/s = direct 기준 {(1 - OPERATING / LOCAL_DIRECT) * 100:.1f}%")

conds = {k: cond(k) for k in ("idle", "op", "local")}
have = {k: v for k, v in conds.items() if v}
if "op" not in have or "local" not in have:
    print(f"{NL}  !! op / local 조건이 모두 필요하다. 수집된 것: {sorted(have)}")
    sys.exit(1)

# 요청당 값으로 환산한다. 처리량은 조건마다 다르므로 각자의 값을 쓴다.
def measured(label, fallback):
    """이번 run 이 실제로 낸 처리량을 쓴다. 기준 상수는 폴백일 뿐이다."""
    f = RAW / label / "throughput.txt"
    if f.exists():
        t = f.read_text().strip()
        if t:
            try:
                return float(t), "실측"
            except ValueError:
                pass
    return fallback, "기준값(폴백)"


tput, src = {}, {}
for k, fb in (("op", OPERATING), ("local", LOCAL_DIRECT)):
    tput[k], src[k] = measured(k, fb)
print(f"{NL}  이번 run 처리량:  " + "   ".join(
    f"{k} {tput[k]:.1f} ({src[k]})" for k in ("op", "local")))
print(f"{NL}{'조건':<10}{'wall_s':>8}{'utime/req':>12}{'stime/req':>12}"
      f"{'CPU-ms/req':>13}{'user%':>8}{'kernel%':>9}")
print("-" * W)
per = {}
for k in ("op", "local"):
    c = have[k]
    n = c["wall"] * tput[k]
    per[k] = {"u": c["utime_ms"] / n, "s": c["stime_ms"] / n, "t": c["cpu_ms"] / n}
    print(f"{k:<10}{c['wall']:>8.1f}{per[k]['u']:>12.2f}{per[k]['s']:>12.2f}"
          f"{per[k]['t']:>13.2f}{100 * c['utime_ms'] / c['cpu_ms']:>8.1f}"
          f"{100 * c['stime_ms'] / c['cpu_ms']:>9.1f}")

du = per["op"]["u"] - per["local"]["u"]
ds = per["op"]["s"] - per["local"]["s"]
dt = du + ds
print("-" * W)
print(f"{'transport':<10}{'':>8}{du:>12.2f}{ds:>12.2f}{dt:>13.2f}"
      f"{100 * du / dt:>8.1f}{100 * ds / dt:>9.1f}")

print(f"{NL}── syscall ─────────────────────────────────────────────────────────────")
for k in ("op", "local"):
    c = have[k]
    if "syscw" not in c:
        continue
    n = c["wall"] * tput[k]
    print(f"  {k:<8} read/req {c['syscr'] / n:>8.1f}   write/req {c['syscw'] / n:>8.1f}")

# ── strace: syscall 체류시간 상한 ────────────────────────────────────
st = RAW / "strace" / "summary.txt"
# strace 창에서 처리한 요청 수. calls/req 환산에 쓴다.
_stp = RAW / "strace" / "throughput.txt"
_sdur = RAW / "strace" / "duration.txt"
try:
    STRACE_DUR_REQ = float(_stp.read_text().strip()) * float(_sdur.read_text().strip())
except Exception:
    STRACE_DUR_REQ = 0
strace_us = None
if st.exists():
    txt = io.open(st, encoding="utf-8", errors="replace").read()
    total = None
    rows = []
    # 컬럼: % time / seconds / usecs_per_call / calls / [errors] / syscall
    # 앞서 usecs_per_call 과 calls 를 뒤바꿔 읽어 호출 수가 100배 작게
    # 나왔고, 그 때문에 "strace 가 한 스레드에만 붙었다" 고 오판할 뻔했다.
    pat = re.compile(
        r"^\s*([\d.]+)\s+([\d.]+)\s+(\d+)\s+(\d+)(?:\s+(\d+))?\s+(\w+)\s*$")
    for ln in txt.splitlines():
        m = pat.match(ln)
        if not m:
            continue
        name = m.group(6)
        sec, us_per_call, calls = float(m.group(2)), int(m.group(3)), int(m.group(4))
        if name == "total":
            total = sec
        else:
            rows.append((name, sec, calls, us_per_call))
    if rows:
        print(f"{NL}── strace -c (부풀려진 **상한**) ─────────────────────────────────────")
        print("  ptrace 는 syscall 마다 정지시키므로 체류시간이 실제보다 크게 나온다.")
        print("  따라서 이 값이 작으면 실제는 확정적으로 더 작다 — 한쪽 방향 검정이다.")
        rows.sort(key=lambda r: -r[1])
        print(f"{NL}  {'syscall':<14}{'seconds':>10}{'calls':>10}{'us/call':>9}"
              f"{'calls/req':>11}   비고")
        NOTE = {
            "writev": "응답 송신  <- io_uring 대상",
            "write": "응답 송신  <- io_uring 대상",
            "recvfrom": "요청 수신  <- io_uring 대상",
            "ioctl": "RKNN 드라이버 (NPU 제출)",
            "futex": "스레드 동기화 대기 (블로킹)",
            "epoll_pwait": "이벤트 대기 (블로킹)",
        }
        nreq = STRACE_DUR_REQ
        for name, sec, calls, us in rows[:8]:
            cpr = f"{calls / nreq:>11.1f}" if nreq else f"{'-':>11}"
            print(f"  {name:<14}{sec:>10.4f}{calls:>10}{us:>9}{cpr}   {NOTE.get(name, '')}")
        if total:
            print(f"  {'합계':<14}{total:>10.4f}{sum(r[2] for r in rows):>10}")
        net = sum(sec for name, sec, _, _ in rows if name in ("writev", "write", "recvfrom"))
        if total:
            print(f"{NL}  네트워크 syscall(writev+write+recvfrom) 체류시간: "
                  f"{net:.2f}s / {total:.2f}s = **{100 * net / total:.1f}%**")
            print("  나머지는 futex(동기화 대기) · ioctl(NPU 드라이버) · epoll(이벤트 대기)로,")
            print("  **io_uring 이 손대는 영역이 아니다.**")

print(f"{NL}" + "─" * W)
print("판정 — io_uring 이 회수할 수 있는 상한")
print("─" * W)
print(f"""
  transport 가 요청당 쓰는 노드 CPU: {dt:.2f} ms
    유저 {du:.2f} ms ({100 * du / dt:.0f}%)  — 직렬화 / 유저공간 copy / HTTP2 프레이밍
    커널 {ds:.2f} ms ({100 * ds / dt:.0f}%)  — syscall 진입 / TCP 스택 / copy_to_user

  **io_uring 의 상한은 커널 {ds:.2f} ms 의 일부다.** syscall 진입 비용과
  등록 버퍼로 아낄 수 있는 copy 만 해당하고, TCP 스택 작업은 남는다.
  유저 {du:.2f} ms 는 io_uring 과 무관하다.
""")
print("  ⚠️ CPU-ms/req 는 **비용**이지 **제약**이 아니다. 보드 CPU 가 포화가")
print("     아니라면 이 비용을 줄여도 처리량이 오른다는 보장이 없다.")
print("     mpstat.txt 의 코어별 idle 을 함께 봐야 한다.")
