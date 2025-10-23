#!/usr/bin/env bash

# --- Default values ---
declare spack_env=""

# --- Function to display usage ---
usage() {
    echo "Usage: $0 --spack-env <env>"
    echo ""
    echo "Required arguments:"
    echo "  --spack-env <env>      Specify the spack environment to use."
    exit 1
}

# --- Parse command line arguments ---
while [[ $# -gt 0 ]]; do
    key="$1"
    case $key in
        --spack-env)
        spack_env="$2"
        shift 2
        ;;
        *)
        echo "Unknown option: $1"
        usage
        ;;
    esac
done

# --- Validate arguments ---
if [ -z "$spack_env" ]; then
    echo "Error: Missing required arguments."
    usage
fi

# --- Set up environment ---
spack env activate "$spack_env"


declare pwd=$(pwd)
P=$(basename $(find ~/.user_spack/environments/fast-dta/.spack-env/._view -mindepth 1 -maxdepth 1 -type d))
export PATH="$pwd"/lib/InertialFlowCutter/build:"$pwd"/target/"$release_type":~/rust-nightly/bin:~/.local/bin:$PATH
export RL=$(spack location -i readline%gcc@14)
export NC=$(spack location -i ncurses%gcc@14)
export LX=$(spack location -i libx11%gcc@14)
export LIBRARY_PATH=$LX/lib:$RL/lib:$NC/lib:$LIBRARY_PATH
export CPATH=$RL/include:$NC/include:$LX/include
export LD_LIBRARY_PATH=~/.local/libnsl1/lib64:~/.user_spack/environments/fast-dta/.spack-env/._view/"$P"/lib:$NC/lib:$LD_LIBRARY_PATH
export SUMO_HOME=~/rust_road_router/venvs/libsumo/lib/python3.11/site-packages/sumo
export PATH=$SUMO_HOME/tools/assign:$SUMO_HOME/bin:$PATH

python fastdta/create_random_instances.py -n 2 --min-nodes 300 --max-nodes 500 --min-trips 35000 --max-trips 50000 --seed 42 --output-folder /nfs/work/hcsoere/fast-dta/input/random