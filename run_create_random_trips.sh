#!/usr/bin/env bash

# --- Default values ---
declare spack_env=""
declare num_instances=""
declare min_nodes=""
declare max_nodes=""
declare min_trips=""
declare max_trips=""
declare seed="42"
declare output_folder=""

# --- Function to display usage ---
usage() {
    echo "Usage: $0 --spack-env <env> -n <num> --min-nodes <num> --max-nodes <num> --min-trips <num> --max-trips <num> --output-folder <path> [options]"
    echo ""
    echo "Required arguments:"
    echo "  --spack-env <env>           Specify the spack environment to use."
    echo "  -n, --num-instances <num>   Number of instances to create."
    echo "  --min-nodes <num>           Minimum number of nodes in each instance."
    echo "  --max-nodes <num>           Maximum number of nodes in each instance."
    echo "  --min-trips <num>           Minimum number of trips in each instance."
    echo "  --max-trips <num>           Maximum number of trips in each instance."
    echo "  --output-folder <path>      Folder to save the instances."
    echo ""
    echo "Optional arguments:"
    echo "  --seed <num>                Random seed for reproducibility (default: 42)."
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
        -n|--num-instances)
        num_instances="$2"
        shift 2
        ;;
        --min-nodes)
        min_nodes="$2"
        shift 2
        ;;
        --max-nodes)
        max_nodes="$2"
        shift 2
        ;;
        --min-trips)
        min_trips="$2"
        shift 2
        ;;
        --max-trips)
        max_trips="$2"
        shift 2
        ;;
        --seed)
        seed="$2"
        shift 2
        ;;
        --output-folder)
        output_folder="$2"
        shift 2
        ;;
        *)
        echo "Unknown option: $1"
        usage
        ;;
    esac
done

# --- Validate arguments ---
if [ -z "$spack_env" ] || [ -z "$num_instances" ] || [ -z "$min_nodes" ] || [ -z "$max_nodes" ] || [ -z "$min_trips" ] || [ -z "$max_trips" ] || [ -z "$output_folder" ]; then
    echo "Error: Missing required arguments."
    usage
fi

# --- Set up environment ---
echo "Using spack environment: $spack_env"
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



echo "Calling create_random_instances.py"

python "$pwd"/fastdta/create_random_instances.py -n "$num_instances" --min-nodes "$min_nodes" --max-nodes "$max_nodes" --min-trips "$min_trips" --max-trips "$max_trips" --seed "$seed" --output-folder "$output_folder"
