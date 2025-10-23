#!/usr/bin/env bash

# --- Default values ---
declare spack_env=""
declare experiment=""
declare trip_file_name=""
declare release_type="release"
declare -a routing_algorithms=()
declare output_dir
output_dir=$(pwd)
declare timestamp
timestamp=$(date +'%Y-%m-%d-%H-%M')
declare start_time
start_time=$(date '+%Y-%m-%d %H:%M:%S')

# --- Function to display usage ---
usage() {
    echo "Usage: $0 --spack-env <env> --experiment <name> [options]"
    echo ""
    echo "Required arguments:"
    echo "  --spack-env <env>      Specify the spack environment to use."
    echo "  --experiment <file>    Specify the experiment file to run."
    echo "                         Format: [<input_dir>;<prefix>;<trip_file_name>;<aggregation>;<begin>;<end>;<convergence_deviation>;<convergence_relative-gap>;<last_iter>;<seed>\n]"
    echo ""
    echo "Routing algorithm options (at least one is required):"
    echo "  --cch                  Run all experiments with CCH routing."
    echo "  --ch                   Run all experiments with CH routing."
    echo "  --dijkstra             Run all experiments with Dijkstra routing."
    echo "  --dijkstra-rust        Run all experiments with Dijkstra (Rust) routing."
    echo "  --fast-dta             Run all experiments with Fast DTA routing."
    echo "  --a-star               Run all experiments with A* routing."
    echo ""
    echo "Other options:"
    echo "  --output <path>        Specify the base output directory (default: current directory)."
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
        --output)
        output_dir="$2"
        shift 2
        ;;
        --debug)
        release_type="debug"
        shift
        ;;
        --cch)
        routing_algorithms+=("CCH")
        shift
        ;;
        --fast-dta)
        routing_algorithms+=("fastdta")
        shift
        ;;
        --ch)
        routing_algorithms+=("CH")
        shift
        ;;
        --dijkstra)
        routing_algorithms+=("dijkstra")
        shift
        ;;
        --dijkstra-rust)
        routing_algorithms+=("dijkstra-rust")
        shift
        ;;
        --a-star)
        routing_algorithms+=("astar")
        shift
        ;;
        *)
        echo "Unknown option: $1"
        usage
        ;;
    esac
done

# --- Validate arguments ---
if [ -z "$spack_env" ] || [ -z "$experiment" ] || [ -z "$experiment" ] || [ ${#routing_algorithms[@]} -eq 0 ]; then
    echo "Error: Missing required arguments."
    usage
fi

if [ ! -f "$experiment" ]; then
    echo "Error: Experiment file not found: $experiment"
    exit 1
fi

# --- Set up environment ---
spack env activate "$spack_env"
declare base_output_dir="${output_dir%/}/$timestamp"


echo "Base output directory: $base_output_dir"
mkdir -p "$base_output_dir"

# Copy the experiment file into the output directory for reproducibility
cp "$experiment" "$base_output_dir/"

# --- Write README.md with experiment info ---
(
    
    # Get git info
    git_hash=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
    git_branch=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
    # Get CPU info (first line of lscpu output)
    cpu_info=$(lscpu | grep 'Model name' | head -n1 | sed 's/Model name:[ ]*//')
    # Get RAM info (total in GB)
    ram_info=$(free -g | awk '/^Mem:/ {print $2 " GB"}')
    # List algorithms
    algos="${routing_algorithms[*]}"
    
    cd "$base_output_dir" || exit 1
    # Write README.md
    cat > README.md << EOF
# Experiment Batch Information

**Git Information**
- Commit hash: $git_hash
- Branch: $git_branch

**Start time:** $start_time

**CPU:** ${cpu_info:-unknown}
**RAM:** ${ram_info:-unknown}

**Algorithms run:** $algos
EOF
)

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

# --- Compile the project ---
if [ "$release_type" = "debug" ]; then
    cargo build
else
    cargo build --release
fi

# copy duaIterate.py from fastdta to the venv directory
cp ~/rust_road_router/fastdta/duaIterate.py "$SUMO_HOME"/tools/assign

# --- Run experiments ---
line_index=0
while IFS=';' read -r in_dir prefix trip_file_name aggregation convergence_deviation convergence_relgap last_iter seed || [[ -n "$in_dir" ]]; do
    # Skip empty or commented lines
    [[ -z "$in_dir" || "$in_dir" =~ ^#.* ]] && continue

    net_file="$in_dir/$prefix.net.xml"
    trips_file="$in_dir/$trip_file_name"
    experiment_out_dir="$base_output_dir/$line_index"

    for algorithm in "${routing_algorithms[@]}"; do
        # Use lowercase for directory name
        algo_dir_name=$(echo "$algorithm" | tr '[:upper:]' '[:lower:]')
        out_dir="$experiment_out_dir/$algo_dir_name"

        echo "Processing with algorithm: $algorithm, aggregation: $aggregation"
        echo "Output directory: $out_dir"

        mkdir -p "$out_dir"
        # run commands in a subshell to not change the script's working directory
        (
            cd "$out_dir" || exit

            # Common arguments for duaIterate.py
            declare -a dua_args=(
                -n "$net_file"
                -t "$trips_file"
                --mesosim --aggregation "$aggregation" --begin 0 --end 86400 -l $last_iter
                --routing-algorithm "$algorithm"
                --max-convergence-deviation "$convergence_deviation"
                --relative-gap "$convergence_relgap"
                duarouter--weights.interpolate
                duarouter--seed $seed
                duarouter--precision 6
                sumo--ignore-route-errors
                sumo--aggregate-warnings 5
                sumo--seed $seed
                sumo--default.speeddev 0
                sumo--precision 6
                relative-gap--net-prefix "$prefix"
                relative-gap--net-dir "$in_dir"
            )

            # Add preprocessor args only for CCH
            if [ "$algorithm" = "CCH" ]; then
                dua_args+=(
                    cch-preprocessor--input-prefix "$prefix"
                    cch-preprocessor--input-dir "$in_dir"
                    cch-router--seed $seed
                )
            fi
            # Add preprocessor args only for dijkstra-rust
            if [ "$algorithm" = "dijkstra-rust" ]; then
                dua_args+=(
                    dijkstra-preprocessor--input-prefix "$prefix"
                    dijkstra-preprocessor--input-dir "$in_dir"
                    dijkstra-router--seed $seed
                )
            fi
            # Add preprocessor args only for fastdta
            if [ "$algorithm" = "fastdta" ]; then
                dua_args+=(
                    fastdta-preprocessor--input-prefix "$prefix"
                    fastdta-preprocessor--input-dir "$in_dir"
                    fastdta-router--seed $seed
                    fastdta-router--vdf ptv
                    fastdta-router--samples 0.9 0.1
                )
            fi

            python duaIterate.py "${dua_args[@]}"
        )
    done
    ((line_index++))
done < "$experiment"

echo "All experiments finished."