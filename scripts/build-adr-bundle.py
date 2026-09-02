#!/usr/bin/env python3
"""adrs/ 의 문서를 하나의 마크다운 파일로 묶는다.

파일 간 링크(`007-....md`)는 문서 내 앵커(`#adr-007`)로 바꾼다.
그대로 두면 묶음 안에서 전부 깨진 링크가 된다.

  python scripts/build-adr-bundle.py [생성일]
"""
import re
import sys
from pathlib import Path

ADRS = Path(__file__).resolve().parent.parent / "adrs"
OUT = ADRS / "ALL.md"
DATE = sys.argv[1] if len(sys.argv) > 1 else "미상"

adr_files = [p for p in sorted(ADRS.glob("[0-9][0-9][0-9]-*.md"))
             if not p.name.endswith(".ko.md")]   # 한글 보조본은 묶음에 넣지 않는다
order = [ADRS / "README.md", ADRS / "OVERVIEW.md", *adr_files, ADRS / "TEMPLATE.md"]

# 파일명 → 앵커
anchor = {"README.md": "index", "OVERVIEW.md": "overview", "TEMPLATE.md": "template"}
for f in adr_files:
    anchor[f.name] = f"adr-{f.name[:3]}"


def relink(text: str) -> str:
    """](파일명.md) → ](#앵커)"""
    def sub(m):
        name = m.group(1)
        return f"](#{anchor[name]})" if name in anchor else m.group(0)
    return re.sub(r"\]\(([^)#]+\.md)\)", sub, text)


parts = []
for path in order:
    body = relink(path.read_text(encoding="utf-8").rstrip())
    parts.append(f'<a id="{anchor[path.name]}"></a>\n\n{body}')

banner = f"""
> **This file is generated. Do not edit it directly.**
> It concatenates the {len(order)} source documents under `adrs/` for reading,
> printing and sharing.
> If something needs fixing, fix the source and regenerate.
>
> ```bash
> python scripts/build-adr-bundle.py $(git log -1 --format=%cs -- adrs/)
> ```
>
> - Generated as of: **{DATE}** (the last commit date for `adrs/`)
> - Sources: `adrs/README.md`, `adrs/OVERVIEW.md`, {len(adr_files)} ADRs, `adrs/TEMPLATE.md`
> - Links between files have been rewritten to in-document anchors
"""

# 첫 문서(README)의 제목 바로 뒤에 배너를 넣는다.
head, rest = parts[0].split("\n", 3)[0], parts[0].split("\n", 1)[1]
title_line = rest.lstrip("\n").split("\n", 1)[0]
remainder = rest.lstrip("\n").split("\n", 1)[1]
parts[0] = f"{head}\n\n{title_line}\n{banner}{remainder}"

OUT.write_text("\n\n---\n\n".join(parts) + "\n", encoding="utf-8")
print(f"{OUT.relative_to(ADRS.parent)}  {len(OUT.read_text(encoding='utf-8').splitlines())} lines")
