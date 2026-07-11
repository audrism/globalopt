#!/usr/bin/env bash
# usage: launch_bbob_shard.sh <shard> <pool>   (runs all methods except skopt_gp)
cd /home/audris/swsc/globalopt
export PATH=/home/audris/micromamba/envs/globalopt/bin:$PATH
export OMP_NUM_THREADS=1 OPENBLAS_NUM_THREADS=1 MKL_NUM_THREADS=1
export BBOB_METHODS=$(python -c "
import sys; sys.path.insert(0,'benchmarks')
from run_bench import METHODS
print(','.join(k for k in METHODS if k != 'skopt_gp'))")
exec nice python benchmarks/run_bbob.py --shard "$1" --pool "$2"
