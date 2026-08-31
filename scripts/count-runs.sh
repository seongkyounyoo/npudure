#!/usr/bin/env bash
# 측정 run 합계를 센다.
#
# 왜 스크립트인가
#   이 숫자를 두 문서가 각자 손으로 들고 있다가 어긋났다
#   (experiments/README 343건 vs handoff 약 420건, 2026-08-21 발견).
#   13.2% 오인용과 같은 부류다 — **손으로 관리하는 파생 수치는 갈라진다.**
#   문서에 박을 때는 이 스크립트 출력을 쓰고, 출처를 함께 적는다.
#
# 세는 규칙
#   raw/results.csv 가 있으면  헤더를 뺀 행 수
#   없으면                     probe 를 뺀 *.json 파일 수
#   프로파일 실험(S3.5·S3.9b)은 bench JSON 이 서버에 남고 보드 것만
#   수거하므로 위 규칙으로는 0 이 된다. raw/<조건>/pstat.before 로
#   **조건 수**를 따로 센다.
#   -contaminated 로 끝나는 디렉터리는 **폐기**로 따로 센다.
#   -althost 로 끝나는 디렉터리는 **스케줄러 호스트가 다른** run 이다.
#   유효한 측정이지만 421 계보와 직접 비교할 수 없으므로 합산하지 않고
#   따로 센다 (2026-08-26 서버 교체). docs/environment-matrix.md §10.2
#
# 사용법
#   bash scripts/count-runs.sh          요약
#   bash scripts/count-runs.sh -v       디렉터리별 내역
set -u

VERBOSE=0
[ "${1:-}" = "-v" ] && VERBOSE=1

cd "$(dirname "${BASH_SOURCE[0]}")/.." || exit 1

valid=0; dropped=0; ndir=0; prof=0; alt=0
[ "$VERBOSE" = 1 ] && printf "  %-44s %6s  %s\n" "디렉터리" "run" "근거"

for d in results/*/; do
    b=$(basename "$d")
    csv="$d/raw/results.csv"
    if [ -f "$csv" ]; then
        n=$(( $(wc -l < "$csv") - 1 )); src=CSV
    else
        n=$(find "$d" -name "*.json" 2>/dev/null | grep -vi probe | wc -l); src=JSON
    fi
    if [ "$n" -eq 0 ]; then
        # 프로파일 실험: 조건 디렉터리 수를 센다
        c=$(find "$d/raw" -maxdepth 2 -name pstat.before 2>/dev/null | wc -l)
        if [ "$c" -gt 0 ]; then
            ndir=$((ndir + 1)); prof=$((prof + c))
            [ "$VERBOSE" = 1 ] && printf "  %-44s %6d  %-4s %s\n" "$b" "$c" "조건" "프로파일"
        fi
        continue
    fi
    ndir=$((ndir + 1))
    case "$b" in
        *-contaminated) dropped=$((dropped + n)); tag="폐기" ;;
        *-althost)      alt=$((alt + n));         tag="다른 호스트" ;;
        *)              valid=$((valid + n));     tag="" ;;
    esac
    [ "$VERBOSE" = 1 ] && printf "  %-44s %6d  %-4s %s\n" "$b" "$n" "$src" "$tag"
done

[ "$VERBOSE" = 1 ] && echo "  ──────────────────────────────────────────────────────────"
printf '측정 run 합계: %d건 (bench 유효) + 프로파일 조건 %d건 = %d건\n' \
    "$valid" "$prof" "$((valid + prof))"
printf '  폐기 %d건 (하네스 충돌 오염), 디렉터리 %d개\n' "$dropped" "$ndir"
[ "$alt" -gt 0 ] && printf '  별도 %d건 (스케줄러 호스트 다름 — 합산하지 않는다)
' "$alt"
printf '  생성: bash scripts/count-runs.sh  (%s)\n' "$(date +%Y-%m-%d)"
