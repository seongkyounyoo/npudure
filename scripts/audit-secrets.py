#!/usr/bin/env python3
"""저장소에 비밀번호가 남아 있는지 전수 검사한다.

# 찾을 값을 이 파일에 담지 않는다

**2026-08-27 정정.** 예전에는 찾을 비밀번호를 문자 코드로 조립해
소스에 담았다. 리터럴을 피해 ①자기 검출 ②`git filter-repo
--replace-text` 로 인한 정규식 파손을 막으려는 것이었고, 내부 도구로는
합리적이었다.

**그러나 그 때문에 이 파일 자체가 공개 불가가 됐다.** 문자 코드는
난독화이지 보호가 아니다 — 몇 초면 되돌린다. 공개 저장소 리뷰에서
잡혔다.

이제 값은 **추적하지 않는 파일**에서 읽는다.

    release/audit-patterns.txt      (.gitignore 대상, 이 저장소에 없다)
    또는 $NPUFORGE_AUDIT_PATTERNS

없으면 **실패로 종료한다.** 조용히 0건을 반환하면 게이트가 통과한 것처럼
보인다 — 없는 안전을 파는 쪽이 더 위험하다.

# 멀티라인을 본다

한 줄 정규식으로만 찾으면 실제 개행으로 들어간 형태를 놓친다.
`preflight-check.sh` 에 이런 것이 남아 "제거 완료"로 오판했다.

    collect 'printf "<비밀번호>
    " | sudo -S -p "" cat ...' DRIVERS

사용법:
    python scripts/audit-secrets.py                  # 작업 트리
    python scripts/audit-secrets.py --root DIR       # 다른 트리 (export 스테이징 등)
    python scripts/audit-secrets.py --history        # 커밋 히스토리 전체

패턴 파일 형식 — 한 줄에 하나, `#` 주석 허용:

    board  | 계정명과 같은 문자열. 문맥 안에서만 잡는다
    global | 어디에 있든 잡는다

    board:pi
    global:<값>
"""

from __future__ import annotations

import io
import pathlib
import re
import subprocess
import sys

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")

_ARGS = sys.argv[1:]


def _opt(name: str, default=None):
    if name in _ARGS:
        i = _ARGS.index(name)
        return _ARGS[i + 1] if i + 1 < len(_ARGS) else default
    return default


ROOT = pathlib.Path(_opt("--root") or pathlib.Path(__file__).resolve().parent.parent).resolve()
HERE = pathlib.Path(__file__).resolve().parent.parent
SKIP_DIRS = {"target", ".git", ".claude", "datasets", "node_modules"}
EXTS = {".sh", ".md", ".py", ".rs", ".toml", ".in", ".yml", ".yaml", ".txt", ".json"}
WINDOW = 400


def load_patterns() -> tuple[list[str], list[str]]:
    """찾을 값을 외부 파일에서 읽는다. 소스에 담지 않는다(모듈 문서 참조)."""
    import os

    cand = [p for p in (os.environ.get("NPUFORGE_AUDIT_PATTERNS"),) if p]
    cand += [HERE / "release" / "audit-patterns.txt", ROOT / "release" / "audit-patterns.txt"]
    for c in cand:
        path = pathlib.Path(c)
        if not path.is_file():
            continue
        board, glob = [], []
        for raw in path.read_text(encoding="utf-8").splitlines():
            line = raw.split("#", 1)[0].strip()
            if not line or ":" not in line:
                continue
            kind, _, val = line.partition(":")
            (board if kind.strip() == "board" else glob).append(val.strip())
        return board, glob, path.resolve()
    sys.exit(
        "패턴 파일이 없다: release/audit-patterns.txt 또는 $NPUFORGE_AUDIT_PATTERNS. "
        "찾을 값은 이 소스에 담지 않는다 — 형식은 모듈 문서 참조. "
        "**0건으로 통과시키지 않는다.** 없는 안전을 파는 쪽이 더 위험하다."
    )


BOARD_VALS, GLOBAL_VALS, PATTERN_FILE = load_patterns()

# 비밀번호가 쓰이는 문맥. 이 근처의 리터럴만 문제로 본다.
#
# **산문 문맥을 반드시 포함한다.** 2026-08-22 에 공개 준비를 하다가
# `docs/handoff-2026082*.md` 의 아래 줄이 이 검사를 그냥 통과한 것을 발견했다.
#
#     - 보드 sudo 비번 `pi`, 로컬 `~/.npuforge/sudo-pass`.
#
# printf 도 파이프도 sudo -S 도 없으니 실행 문맥 패턴에 안 걸렸다.
# 원래 전제가 "비밀번호는 코드로 샌다" 였는데, 이 저장소는 문서가 훨씬 많다.
# 계측기가 정확히 새는 지점에서 눈이 멀어 있었다.
CONTEXT = re.compile(
    r"sudo\s+-S|SSH_ASKPASS|askpass|sshpass|--password|PASS\s*="
    r"|비번|비밀번호|passwd|password",
    re.I,
)

_B = "(?:" + "|".join(re.escape(v) for v in BOARD_VALS) + ")"

SECRET = re.compile(
    "|".join([
        # printf "pw\n" / printf 'pw' / printf "pw + 실제 개행
        r"printf\s+\\?[\"']" + _B + r"(?:\\+n)?\\?[\"']?",
        # 'pw' |   (파이프로 넘기는 형태)
        r"[\"']" + _B + r"[\"']\s*\|",
        # ${NPUFORGE_..._PASS:-pw}  — 계정명 USER 는 제외한다
        r"_PASS[^\n]{0,12}:-" + _B + r"\}",
        # 산문에서 값으로 적힌 형태: `pi` / **pi** / "pi" / 'pi'
        #
        # 보드 비번은 **계정명과 같은 문자열**이라 노트북 비번처럼 문맥 없이
        # 전역 검색을 할 수 없다(/home/pi, ssh pi@ 가 수백 건 걸린다).
        # 그래서 산문 문맥(CONTEXT) 안에서 **구분자로 감싼 경우만** 잡는다.
        r"[`*\"']" + _B + r"[`*\"']",
    ]),
    re.I,
)

SELF = pathlib.Path(__file__).name


def scan_text(name: str, text: str) -> list[str]:
    out = []
    for m in CONTEXT.finditer(text):
        lo = max(0, m.start() - WINDOW)
        hi = min(len(text), m.start() + WINDOW)
        window = text[lo:hi]
        for sm in SECRET.finditer(window):
            line = text[: lo + sm.start()].count("\n") + 1
            snippet = window[max(0, sm.start() - 30) : sm.start() + 50]
            out.append(f"{name}:{line}  ...{snippet.replace(chr(10), '/n')}...")
    # global 값은 문맥과 무관하게 잡는다. 어디에 있든 나오면 안 된다.
    for val in GLOBAL_VALS:
        for sm in re.finditer(re.escape(val), text, re.I):
            out.append(f"{name}:{text[: sm.start()].count(chr(10)) + 1}  <전역 금지 값>")

    # **문자 코드로 조립한 형태도 잡는다.**
    #
    # 2026-08-27: 이 스크립트 자신이 찾을 값을 `chr(c) for c in (49, 50, ...)`
    # 로 담고 있었고, 리터럴이 아니라는 이유로 스스로를 통과시켰다. 공개
    # 저장소 리뷰에서 잡혔다. 난독화는 보호가 아니다.
    for val in BOARD_VALS + GLOBAL_VALS:
        if len(val) < 4:
            continue                      # 짧은 값은 오탐이 많다
        codes = [str(ord(ch)) for ch in val]
        pat = r"[,\s(\[]\s*".join(codes)
        for sm in re.finditer(pat, text):
            ln = text[: sm.start()].count(chr(10)) + 1
            out.append(f"{name}:{ln}  <문자 코드로 조립된 금지 값>")
    return out


def scan_worktree() -> list[str]:
    hits: list[str] = []
    for p in ROOT.rglob("*"):
        if not p.is_file() or p.name == SELF or p.resolve() == PATTERN_FILE:
            continue
        if any(part in SKIP_DIRS for part in p.parts) or p.suffix not in EXTS:
            continue
        try:
            text = p.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue
        hits += scan_text(str(p.relative_to(ROOT)).replace("\\", "/"), text)
    return hits


def scan_history() -> list[str]:
    """모든 커밋의 모든 블롭을 훑는다."""
    revs = subprocess.run(
        ["git", "rev-list", "--all"], cwd=ROOT,
        capture_output=True, text=True, check=True,
    ).stdout.split()

    hits: list[str] = []
    seen: set[str] = set()
    for rev in revs:
        listing = subprocess.run(
            ["git", "ls-tree", "-r", rev], cwd=ROOT,
            capture_output=True, text=True, check=True,
        ).stdout
        for line in listing.splitlines():
            try:
                meta, path = line.split("\t", 1)
                blob = meta.split()[2]
            except (ValueError, IndexError):
                continue
            if blob in seen or path.endswith(SELF):
                continue
            seen.add(blob)
            if pathlib.PurePosixPath(path).suffix not in EXTS:
                continue
            raw = subprocess.run(
                ["git", "cat-file", "-p", blob], cwd=ROOT,
                capture_output=True, check=False,
            ).stdout
            try:
                text = raw.decode("utf-8")
            except UnicodeDecodeError:
                continue
            hits += scan_text(f"{rev[:8]} {path}", text)
    return hits


def main() -> int:
    history = "--history" in sys.argv
    hits = scan_history() if history else scan_worktree()
    label = "커밋 히스토리" if history else "작업 트리"
    uniq = list(dict.fromkeys(hits))

    if not uniq:
        print(f"{label}: 비밀번호 없음")
        return 0

    print(f"{label}: {len(uniq)}건 발견")
    for h in uniq:
        print(f"  {h}")
    return 1


if __name__ == "__main__":
    sys.exit(main())
