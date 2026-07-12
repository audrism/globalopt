#!/usr/bin/env bash
# Build the replication-only artifact for reviewers (TOMS RCR):
# everything needed to run REPLICATION.md, nothing about submission
# logistics.  Produces globalopt_replication.tar.gz at the repo root.
#
# Included: sources (Rust/Fortran/R/Python), benchmark harnesses and raw
# result CSVs, docs, the pristine upstream distribution, and the paper
# sources whose figures/tables the analysis regenerates.
# Excluded: submission crib sheet and cover letter, venue packages and
# bundles, git metadata, build outputs.
set -euo pipefail
cd "$(dirname "$0")/.."

stage=$(mktemp -d)
git archive HEAD | tar -x -C "$stage"

rm -rf \
  "$stage/paper/TOMS_SUBMISSION.md" \
  "$stage/paper/main_toms."* \
  "$stage/paper/toms_package" \
  "$stage/paper/globalopt_arxiv.tar.gz" \
  "$stage/paper/globalopt_toms.zip" \
  "$stage/paper/arxiv_metadata.txt" \
  "$stage/paper/abstract.txt" \
  "$stage/tools/make_arxiv_bundle.sh" \
  "$stage/tools/make_toms_package.sh" \
  "$stage/tools/make_replication_artifact.sh"

# the reviewer's entry point goes first in the listing
mv "$stage/REPLICATION.md" "$stage/00_REPLICATION.md" 2>/dev/null || true
mv "$stage/00_REPLICATION.md" "$stage/REPLICATION.md"

tar czf globalopt_replication.tar.gz -C "$stage" .
rm -rf "$stage"
echo "wrote globalopt_replication.tar.gz ($(du -h globalopt_replication.tar.gz | cut -f1)):"
tar tzf globalopt_replication.tar.gz | grep -vE "/$" | head -8
echo "  ... ($(tar tzf globalopt_replication.tar.gz | grep -vcE '/$') files)"
