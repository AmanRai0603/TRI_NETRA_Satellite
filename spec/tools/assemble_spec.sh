#!/usr/bin/env bash
# Assembles SPEC.md from spec/*.md in name order. Edit the sections, not SPEC.md.
#   tools/assemble_spec.sh           writes SPEC.md
#   tools/assemble_spec.sh --check   exits 1 if SPEC.md is not the sections joined
set -euo pipefail
cd "$(dirname "$0")/.."
case "${1:-}" in
  "") cat spec/*.md > SPEC.md
      echo "SPEC.md: $(wc -l < SPEC.md) lines, $(wc -w < SPEC.md) words, from $(ls spec/*.md | wc -l) sections" ;;
  --check) if cat spec/*.md | cmp -s - SPEC.md; then echo "SPEC.md is current"; else echo "SPEC.md is STALE: run tools/assemble_spec.sh"; exit 1; fi ;;
  *) echo "usage: tools/assemble_spec.sh [--check]"; exit 2 ;;
esac
