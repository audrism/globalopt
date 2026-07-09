#!/usr/bin/env bash
# Rebuild the arXiv submission bundle (paper/globalopt_arxiv.tar.gz).
#
# The bundled main.tex gets \pdfoutput=1 prepended (arXiv's pdflatex
# needs it; tectonic's XeTeX chokes on it, so the working copy stays
# clean).  The bundle ships main.bbl, not refs.bib, per arXiv practice.
set -euo pipefail
cd "$(dirname "$0")/../paper"

FIGS="perf_profile_python perf_profile_r data_profile_python \
data_profile_python_25n ffi_decomposition perf_profile_bbob \
data_profile_bbob_25n"

rm -f main.aux main.bbl main.blg main.log main.out
tectonic --keep-intermediates main.tex >/dev/null

rm -rf arxiv_bundle
mkdir -p arxiv_bundle/fig arxiv_bundle/tab
{ echo '\pdfoutput=1'; cat main.tex; } > arxiv_bundle/main.tex
cp main.bbl arxiv_bundle/
cp tab/median_gap.tex arxiv_bundle/tab/
for f in $FIGS; do cp "fig/$f.pdf" arxiv_bundle/fig/; done

# verify the bundle compiles from a clean state (strip the pdflatex-only
# first line for the tectonic check)
vdir=$(mktemp -d)
cp -r arxiv_bundle/* "$vdir"
sed -i '1d' "$vdir/main.tex"
(cd "$vdir" && tectonic main.tex >/dev/null)
rm -rf "$vdir"

tar czf globalopt_arxiv.tar.gz -C arxiv_bundle main.tex main.bbl \
    tab/median_gap.tex fig
echo "wrote paper/globalopt_arxiv.tar.gz:"
tar tzf globalopt_arxiv.tar.gz
