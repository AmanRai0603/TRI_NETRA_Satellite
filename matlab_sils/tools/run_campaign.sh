#!/usr/bin/env bash
# Run a Monte Carlo campaign with GNU Octave on N worker processes, then collect.
#   tools/run_campaign.sh mc_nadir_ais 4
# MATLAB users: run_campaign('mc_nadir_ais') uses parfor when available.
# Copyright (c) 2026 Agastya. All rights reserved.
set -euo pipefail
id=$1; n=${2:-4}
cd "$(dirname "$0")/.."
total=$(python3 -c "import json;print(json.load(open('data/campaigns/$id.json'))['runs'])")
for w in $(seq 1 $n); do
  octave-cli --no-gui -q --eval "startup_asils; addpath tools; run_campaign('$id', $w:$n:$total);" > "store/results/${id}_worker$w.log" 2>&1 &
done
wait
octave-cli --no-gui -q --eval "startup_asils; addpath tools; run_campaign('$id');"
