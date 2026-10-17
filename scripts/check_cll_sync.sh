#!/usr/bin/env bash
# Check that the vendored CLL pipeline in src/cll is identical to the
# tergenzalvi sources, apart from crate-path glue.
#
# Usage: scripts/check_cll_sync.sh [PATH_TO_TERGENZALVI]
#   (default: ../lojban-morph-cll-sketch, or $TERGENZALVI)
#
# Glue that is normalised before diffing (tergenzalvi -> camxes-rs):
#   crate::            -> crate::cll::
#   tergenzalvi::      -> camxes_rs::cll::
#   ../../grammar-src/ -> ../data/        (include_str! data paths)
#   #[cfg(any(test, all(feature = "web", target_arch = "wasm32")))] -> #[cfg(test)]
# Files that exist only in tergenzalvi (lib.rs, web.rs, bin/, constructions/)
# and only in camxes-rs (mod.rs) are not compared.
# Exit status: 0 when in sync, 1 when anything differs.
set -euo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
tz="${1:-${TERGENZALVI:-$here/../lojban-morph-cll-sketch}}"
tz="$(cd "$tz" && pwd)"
modules=(cfg constraints grammar lex phonology shapes tables forms.rs)
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$tmp/cll/data"
for m in "${modules[@]}"; do
  if [ -e "$tz/src/$m" ]; then cp -r "$tz/src/$m" "$tmp/cll/"; fi
done
find "$tmp/cll" -name '*.rs' -print0 | xargs -0 sed -i \
  -e 's/crate::/crate::cll::/g' \
  -e 's/tergenzalvi::/camxes_rs::cll::/g' \
  -e 's#\.\./\.\./grammar-src/#../data/#g' \
  -e 's/#\[cfg(any(test, all(feature = "web", target_arch = "wasm32")))\]/#[cfg(test)]/'
cp "$tz/grammar-src/cll21.ebnf" "$tz/grammar-src/cmavo_selmaho.tsv" "$tmp/cll/data/"
status=0
for m in "${modules[@]}" data; do
  diff -ruN "$tmp/cll/$m" "$here/src/cll/$m" \
    | sed -e "s#$tmp/cll/#tergenzalvi/src/#g" -e "s#$here/src/cll/#camxes-rs/src/cll/#g" \
    | grep . && status=1
done
# Files present in camxes-rs src/cll but absent upstream (other than mod.rs).
for f in $(cd "$here/src/cll" && find . -type f ! -path ./mod.rs | sort); do
  [ -e "$tmp/cll/$f" ] || { echo "only in camxes-rs: src/cll/${f#./}"; status=1; }
done
if [ "$status" -eq 0 ]; then
  echo "src/cll is in sync with $tz ($(git -C "$tz" rev-parse --short HEAD 2>/dev/null || echo '?'))"
fi
exit "$status"
