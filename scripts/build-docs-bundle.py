#!/usr/bin/env python3
"""docs/ 의 문서를 하나의 마크다운 파일로 묶는다.

`adrs/ALL.md` 와 같은 목적이다 — 읽기·인쇄·검토를 한 파일에서 하려는 것.

파일 간 링크는 문서 내 앵커로 바꾼다. 그대로 두면 묶음 안에서 전부
깨진 링크가 된다. `adrs/` 와 달리 `docs/` 는 하위 디렉터리가 있어
**각 원본의 위치를 기준으로 상대 경로를 해석**해야 한다.

`docs/` 밖을 가리키는 링크(`../results/...`)는 건드리지 않는다.
묶음도 `docs/` 안에 놓이므로 그대로 유효하다.

  python scripts/build-docs-bundle.py [생성일]

**비공개 문서는 넣지 않는다** — `handoff-*.md`, `public/`.
이 묶음은 공개본으로 나간다.
"""
import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOCS = ROOT / "docs"
OUT = DOCS / "ALL.md"
DATE = sys.argv[1] if len(sys.argv) > 1 else "미상"

# 읽는 순서 — 무엇을 만들려 했나 → 어떻게 만드나 → 무엇이 나왔나 → 어쩌다 이렇게 됐나
def g(pattern, exclude=()):
    return [p for p in sorted(DOCS.glob(pattern))
            if p.name not in exclude and not p.name.endswith(".ko.md")]

ORDER = [
    DOCS / "00-PRD.md",
    DOCS / "01-TECHSPEC.md",
    DOCS / "02-HARDWARE-SETUP.md",
    DOCS / "03-DEVELOPMENT-REQUIREMENTS.md",
    DOCS / "FAQ.md",
    DOCS / "GLOSSARY.md",
    DOCS / "infrastructure.md",
    DOCS / "environment-matrix.md",
    DOCS / "hosts" / "README.md",
    *g("hosts/*.md", exclude=("README.md",)),
    DOCS / "experiments" / "README.md",
    *g("experiments/*.md", exclude=("README.md",)),
    DOCS / "RESULTS.md",
    DOCS / "discuss.md",
    DOCS / "board-worklog.md",
    DOCS / "TODO.md",
]
missing = [p for p in ORDER if not p.exists()]
if missing:
    sys.exit("원본 없음: " + ", ".join(str(p.relative_to(ROOT)) for p in missing))

# 묶음에 들어가지 않은 docs/*.md 가 있으면 알린다 (조용히 빠지는 것을 막는다)
bundled = {p.resolve() for p in ORDER}
known_private = {"ALL.md", "PROVENANCE.md"}
for p in sorted(DOCS.rglob("*.md")):
    if p.resolve() in bundled or p.name in known_private:
        continue
    if p.name.startswith("handoff-") or p.parent.name == "public":
        continue                                   # 의도적 비공개
    if p.name.endswith(".ko.md"):
        continue                                   # 한글 보조본. 영문 정본이 묶음에 들어간다
    print(f"  ⚠️ 묶음에 없음: {p.relative_to(ROOT)}")

def slug(path):
    rel = path.relative_to(DOCS)
    stem = rel.stem.lower().replace("_", "-").replace(".", "-")
    if rel.parent == Path("."):
        return stem
    return f"{rel.parent.as_posix().replace('/', '-')}-{stem}"

anchor = {p.resolve(): slug(p) for p in ORDER}

# 링크·이미지 전부. 묶음 밖으로 나가는 것도 경로를 다시 계산해야 한다.
LINK = re.compile(r"(!?\[[^\]]*\])\((?!https?://|mailto:|#)([^)\s]+?)(#[^)]*)?\)")

def relink(text, src):
    """묶음 안이면 앵커로, 밖이면 `docs/` 기준 상대경로로 바꾼다.

    하위 디렉터리 원본의 `../../adrs/x.md` 를 그대로 두면 묶음(`docs/ALL.md`)
    에서는 저장소 밖을 가리키게 된다.
    """
    def sub(m):
        label, target, frag = m.group(1), m.group(2), m.group(3) or ""
        resolved = (src.parent / target).resolve()
        a = anchor.get(resolved)
        if a:
            return f"{label}(#{a})"
        try:
            rel = os.path.relpath(resolved, DOCS).replace(os.sep, "/")
        except ValueError:
            return m.group(0)                      # 다른 드라이브 등
        return f"{label}({rel}{frag})"
    return LINK.sub(sub, text)

parts, toc = [], []
for path in ORDER:
    a = anchor[path.resolve()]
    body = relink(path.read_text(encoding="utf-8").rstrip(), path)
    title = next((l.lstrip("# ").strip() for l in body.splitlines() if l.startswith("# ")),
                 path.stem)
    toc.append(f"- [{title}](#{a})  ·  `{path.relative_to(ROOT).as_posix()}`")
    parts.append(f'<a id="{a}"></a>\n\n{body}')

header = f"""<a id="index"></a>

# NPUDure 문서 묶음

> **이 파일은 생성물이다. 직접 편집하지 않는다.**
> `docs/` 의 원본 {len(ORDER)}개를 읽기·인쇄·검토용으로 이어 붙인 것이다.
> 고칠 것이 있으면 원본을 고치고 다시 만든다.
>
> ```bash
> python scripts/build-docs-bundle.py $(git log -1 --format=%cs -- docs/)
> ```
>
> - 생성 기준: **{DATE}** (`docs/` 최종 커밋일)
> - 파일 간 링크는 문서 내 앵커로 바뀌어 있다
> - `docs/` 밖을 가리키는 링크(`../results/...`)는 그대로다
> - **세션 인수인계 메모(`handoff-*.md`)와 공개 준비 문서(`public/`)는 빠져 있다.**
>   연구 산출물이 아니다
>
> 아키텍처 결정 기록은 별도 묶음이다 — [`adrs/ALL.md`](../adrs/ALL.md)

## 차례

{chr(10).join(toc)}
"""

OUT.write_text(header + "\n---\n\n" + "\n\n---\n\n".join(parts) + "\n", encoding="utf-8")
n = len(OUT.read_text(encoding="utf-8").splitlines())
print(f"{OUT.relative_to(ROOT)}  {len(ORDER)}개 문서, {n} lines")
