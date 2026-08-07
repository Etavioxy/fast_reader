#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

B=../target/release/fast-reader-bench

echo "[0/3] building"
(cd .. && cargo build --release -p fast-reader-bench 2>&1 | tail -1)

echo "[1/3] generating dataset"
python scripts/gen_data.py --lines 1000000 --out data/sample.jsonl

echo "[2/3] running full matrix -> data/results.tsv"
"$B" all data/sample.jsonl

echo "[3/3] generating REPORT.md"
python scripts/make_report.py data/results.tsv --out REPORT.md
echo "done."
