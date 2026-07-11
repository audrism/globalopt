#!/usr/bin/env bash
# Build the TOMS submission package: review PDF + source zip.
set -euo pipefail
cd "$(dirname "$0")/../paper"
rm -f main_toms.aux main_toms.bbl main_toms.log
tectonic --keep-intermediates main_toms.tex >/dev/null
rm -rf toms_package && mkdir -p toms_package
cp main_toms.tex body.tex abstract.tex refs.bib main_toms.bbl toms_package/
mkdir -p toms_package/fig toms_package/tab
cp tab/median_gap.tex toms_package/tab/
grep -oE "includegraphics[^{]*\{[^}]+\}" body.tex | sed 's/.*{//;s/}//' | while read f; do
  cp "$f" "toms_package/$f"
done
cp main_toms.pdf toms_package/
(cd toms_package && zip -qr ../globalopt_toms.zip .)
echo "wrote paper/globalopt_toms.zip and paper/toms_package/ (review PDF: main_toms.pdf)"
