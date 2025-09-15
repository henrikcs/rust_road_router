#!/usr/bin/env bash

# --- Default values ---
declare spack_env=""
declare experiment=""
declare trip_file_name=""
declare release_type="release"
declare timestamp
timestamp=$(date +'%Y-%m-%d-%H-%M')

# --- Function to display usage ---
usage() {
    echo "Usage: $0 --spack-env <env> --experiment <name> [options]"
    echo ""
    echo "Required arguments:"
    echo "  --spack-env <env>      Specify the spack environment to use."
    echo "  --experiment <file>    Specify the experiment file to run."
    echo "                Format: [<net_dir>;<net_prefix>;<trip_file_name>;<dta_dir>\n]"
    echo "  --debug                Build and use the debug target instead of release."
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
        --experiment)
        experiment="$2"
        shift 2
        ;;
        --debug)
        release_type="debug"
        shift
        ;;
        *)
        echo "Unknown option: $1"
        usage
        ;;
    esac
done

# --- Validate arguments ---
if [ -z "$spack_env" ] || [ -z "$experiment" ] || [ -z "$experiment" ]; then
    echo "Error: Missing required arguments."
    usage
fi

if [ ! -f "$experiment" ]; then
    echo "Error: Experiment file not found: $experiment"
    exit 1
fi

# --- Set up environment ---
spack env activate "$spack_env"

declare pwd=$(pwd)
P=$(basename $(find ~/.user_spack/environments/"$spack_env"/.spack-env/._view -mindepth 1 -maxdepth 1 -type d))
export PATH="$pwd"/lib/InertialFlowCutter/build:"$pwd"/target/"$release_type":~/rust-nightly/bin:~/.local/bin:$PATH
export RL=$(spack location -i readline%gcc@14)
export NC=$(spack location -i ncurses%gcc@14)
export LX=$(spack location -i libx11%gcc@14)
export LIBRARY_PATH=$LX/lib:$RL/lib:$NC/lib:$LIBRARY_PATH
export CPATH=$RL/include:$NC/include:$LX/include
export LD_LIBRARY_PATH=~/.local/libnsl1/lib64:~/.user_spack/environments/"$spack_env"/.spack-env/._view/"$P"/lib:$NC/lib:$LD_LIBRARY_PATH

# --- Compile the project ---
if [ "$release_type" = "debug" ]; then
    cargo build
else
    cargo build --release
fi


# --- Run calculations ---
line_index=0
while IFS=';' read -r net_dir net_prefix trip_file_name dta_dir || [[ -n "$net_dir" ]]; do
    # Skip empty or commented lines
    [[ -z "$net_dir" || "$net_dir" =~ ^#.* ]] && continue

    trips_file="$net_dir/$trip_file_name"

    sumo-relative-gap-calculator --net-dir "$net_dir" --net-prefix "$net_prefix" --trips-file "$trips_file" --dta-dir "$dta_dir"

    ((line_index++))
done < "$experiment"

echo "All calculations finished."