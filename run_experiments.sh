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
declare message=""
declare fastdta_samples=""
declare sumo_samples=""
declare repetitions=1
declare use_logit=false

# --- Function to display usage ---
usage() {
    echo "Usage: $0 --spack-env <env> --experiment <name> [options]"
    echo ""
    echo "Required arguments:"
    echo "  --spack-env <env>      Specify the spack environment to use."
    echo "  --experiment <file>    Specify the experiment file to run."
    echo "                         Format: [<input_dir>;<prefix>;<trip_file_name>;<begin>;<end>;<aggregation>;<begin>;<end>;<convergence_deviation>;<convergence_relative-gap>;<last_iter>\n]"
    echo ""
    echo "Routing algorithm options (at least one is required):"
    echo "  --cch                                  Run all experiments with CCH routing."
    echo "  --ch                                   Run all experiments with CH routing."
    echo "  --dijkstra                             Run all experiments with Dijkstra routing."
    echo "  --dijkstra-rust                        Run all experiments with Dijkstra (Rust) routing."
    echo "  --fast-dta \"((<number> )+;)+\"        Run all experiments with Fast DTA routing with the given list of samples."
    echo "                                         Example: --fast-dta \"1.0 1.0; 1 2 3 4; 1 1 1 1;\""
    echo "  --sumo \"(<number>+;)+\"               Run all experiments with SUMO Sampled routing (same as fast dta, only using SUMO as the traffic model)."
    echo "                                         Example: --sumo \"1.0 1.0; 1 2 3 4; 1 1 1 1;\""
    echo "  --a-star                               Run all experiments with A* routing."
    echo "  --fast-dta2                            Run all experiments with Fast DTA v2 routing."
    echo ""
    echo "Other options:"
    echo "  --output <path>        Specify the base output directory (default: current directory). \"[TIME]\" is replaced with timestamp."
    echo "  --message, -m <text>   Add a custom message to the experiment README.md file."
    echo "  --debug                Build and use the debug target instead of release."
    echo "  --repeat <N >= 1>           Repeat each experiment N times using a different seed. (default: 1)"
    echo "  --logit                Use logit choice model with default parameters."
    exit 1
}

# parse_sample_string is a function which is given a string in the format (<number>+;?)+ 
# where number is a positive float or integer and numbers are separated by spaces and groups are separated by semicolons
# Populates the array variable passed as the second argument with the parsed samples
# Example input: "1.0 1.0; 1 2 3 4; 1 1 1 1;"
# Example output in array: ("1.0 1.0" "1 2 3 4" "1 1 1 1") 
parse_samples() {
    local input_string="$1"
    local -n result_array="$2"
    
    IFS=';' read -ra groups <<< "$input_string"
    result_array=()
    for group in "${groups[@]}"; do
        # Trim leading/trailing whitespace
        group=$(echo "$group" | xargs)
        if [ -n "$group" ]; then
            result_array+=("$group")
        fi
    done
}

call_duaIterate() {
    local out_dir="$1"
    shift
    mkdir -p "$out_dir"
    # go to subshell to not affect current shell's working directory
    (
        cd "$out_dir" || exit
        echo "Calling duaIterate.py with output directory: $out_dir and arguments: $*"
        python "$SUMO_HOME"/tools/assign/duaIterate.py "$@"
    )
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
        --message|-m)
        message="$2"
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
        fastdta_samples="$2"
        shift 2
        ;;
        --fast-dta2)
        routing_algorithms+=("ksp")
        shift
        ;;
        --sumo)
        routing_algorithms+=("sumo-sample")
        sumo_samples="$2"
        shift 2
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
         --repeat)
        repetitions="$2"
        shift 2
        ;;
        --logit)
        use_logit=true
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
# Create base output directory, replacing [TIME] with timestamp if present
output_dir="${output_dir//\[TIME\]/$timestamp}"
# the following syntax removes trailing slash if present
declare base_output_dir="${output_dir%/}"


echo "Base output directory: $base_output_dir"
mkdir -p "$base_output_dir"

# Copy the experiment file into the output directory for reproducibility
cp "$experiment" "$base_output_dir/"

# --- Write README.md with experiment info ---
(
    # Get git info
    git_hash=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
    git_branch=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
    # Get CPU info with Ghz, Cores, Threads and Model name
    cpu_info=$(lscpu | grep 'Model name' | head -n1 | sed 's/Model name:[ ]*//')
    cpu_max_mhz=$(lscpu | grep 'CPU max MHz' | head -n1 | sed 's/CPU max MHz:[ ]*//' | sed 's/,/\./')
    cpu_ghz=$(awk "BEGIN {printf \"%.2f\", $cpu_max_mhz / 1000}" 2>/dev/null || echo "N/A")
    cpu_cores=$(lscpu | grep 'Core(s) per socket' | head -n1 | sed 's/Core(s) per socket:[ ]*//')
    cpu_sockets=$(lscpu | grep 'Socket(s)' | head -n1 | sed 's/Socket(s):[ ]*//')
    cpu_threads=$(lscpu | grep 'Thread(s) per core' | head -n1 | sed 's/Thread(s) per core:[ ]*//')
    cpu_info="${cpu_info} | ${cpu_ghz} GHz | Cores: $((cpu_cores * cpu_sockets)) | Threads: $((cpu_cores * cpu_sockets * cpu_threads))" 
    
    # Get RAM info (total in GB)
    ram_info=$(free -g | awk '/^Mem:/ {print $2 " GB"}')
    # List algorithms
    algos="${routing_algorithms[*]}"
    ## if there are samples in sumo or in fastdta, append that info to algos
    if [ -n "$fastdta_samples" ]; then
        algos+=" (fastdta samples: $fastdta_samples)"
    fi
    if [ -n "$sumo_samples" ]; then
        algos+=" (sumo samples: $sumo_samples)"
    fi
    
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

${message:+**Message:**
$message
}
EOF
)

# spack env activate fast-dta
# declare release_type="release"
# declare spack_env="fast-dta"
declare pwd=$(pwd)
P=$(basename $(find ~/.user_spack/environments/fast-dta/.spack-env/._view -mindepth 1 -maxdepth 1 -type d))
export PATH="$pwd"/lib/InertialFlowCutter/build:"$pwd"/target/"$release_type":~/rust-nightly/bin:~/.local/bin:$PATH
export RL=$(spack location -i readline%gcc@14)
export NC=$(spack location -i ncurses%gcc@14)
export LX=$(spack location -i libx11%gcc@14)
export LIBRARY_PATH=$LX/lib:$RL/lib:$NC/lib:$LIBRARY_PATH
export CPATH=$RL/include:$NC/include:$LX/include:$CPATH
export LD_LIBRARY_PATH=~/.local/libnsl1/lib64:~/.user_spack/environments/fast-dta/.spack-env/._view/"$P"/lib:$NC/lib:$LD_LIBRARY_PATH
export SUMO_HOME=~/rust_road_router/venvs/libsumo/lib/python3.11/site-packages/sumo
export PATH=$SUMO_HOME/bin:$PATH

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
while IFS=';' read -r in_dir prefix trip_file_name begin end aggregation convergence_deviation convergence_relgap last_iter || [[ -n "$in_dir" ]]; do
    # Skip empty or commented lines
    [[ -z "$in_dir" || "$in_dir" =~ ^#.* ]] && continue

    
    net_file="$in_dir/$prefix.net.xml"
    trips_file="$in_dir/$trip_file_name"
    

    for algorithm in "${routing_algorithms[@]}"; do

        repetion_count=0

        while [ $repetion_count -lt $repetitions ]; do
            ((repetion_count++))

            experiment_out_dir="$base_output_dir/$line_index"
            seed=$repetion_count

            # Use lowercase for directory name
            algo_dir_name=$(echo "$algorithm" | tr '[:upper:]' '[:lower:]')
            out_dir="$experiment_out_dir/$algo_dir_name"

            # Common arguments for duaIterate.py
            declare -a dua_args=(
                -n "$net_file"
                -t "$trips_file"
                --mesosim --aggregation "$aggregation" --begin $begin --end $end -l $last_iter
                --routing-algorithm "$algorithm"
                --max-convergence-deviation "$convergence_deviation"
                --relative-gap "$convergence_relgap"
            )
            
            # Add logit args, if --logit is provided
            if [ "$use_logit" = true ]; then
                dua_args+=(
                    --logit
                    --logitbeta
                    1.0
                    --logitgamma
                    1.0
                    --logittheta
                    1.0
                    -s
                )
            fi
            
            dua_args+=(
                duarouter--weights.interpolate
                duarouter--seed $seed
                sumo--ignore-route-errors
                sumo--aggregate-warnings 5
                sumo--time-to-teleport.disconnected 0
                sumo--seed $seed
                sumo--step-length 0.1
                sumo--threads $(nproc)
            )

            # Add preprocessor args only for CCH
            if [ "$algorithm" = "CCH" ]; then
                dua_args+=(
                    cch-preprocessor--input-prefix "$prefix"
                    cch-preprocessor--input-dir "$in_dir"
                    cch-router--seed $seed
                )
            fi
            # Add preprocessor args only for fastdta2 (ksp)
            if [ "$algorithm" = "ksp" ]; then
                dua_args+=(
                    ksp-preprocessor--input-prefix "$prefix"
                    ksp-preprocessor--input-dir "$in_dir"
                    ksp-router--seed $seed
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

            # any other algorithm uses relative-gap-calculator: 
            if [ "$algorithm" != "fastdta" ] && [ "$algorithm" != "CCH" ] && [ "$algorithm" != "dijkstra-rust" ]; then
                dua_args+=(
                    relative-gap--net-prefix "$prefix"
                    relative-gap--net-dir "$in_dir"
                )
            fi

            # Add preprocessor args only for fastdta
            if [ "$algorithm" = "fastdta" ]; then

                dua_args+=(
                    fastdta-preprocessor--input-prefix "$prefix"
                    fastdta-preprocessor--input-dir "$in_dir"
                    fastdta-router--seed $seed
                )

                declare -a fastdta_sample_sets
                parse_samples "$fastdta_samples" fastdta_sample_sets
                for sample_set in "${fastdta_sample_sets[@]}"; do
                    # out_dir should be suffixed with sample_set such that out dirs do not overlap
                    # and to mark that this is a new algorithm
                    # sample_set's spaces are replaced with underscores
                    sample_out_dir="${out_dir}_$(echo "$sample_set" | tr ' ' '_')"
                    call_duaIterate "$sample_out_dir/$repetion_count" "${dua_args[@]}" fastdta-router--samples "$sample_set"
                done

                continue
            fi 

            # Add preprocessor args only for sumo-sample
            if [ "$algorithm" = "sumo-sample" ]; then
                dua_args+=(
                    sample-preprocessor--input-prefix "$prefix"
                    sample-preprocessor--input-dir "$in_dir"
                    sample-router--seed $seed
                    sample-router--aggregation "$aggregation"
                )
                
                declare -a sumo_sample_sets
                parse_samples "$sumo_samples" sumo_sample_sets
                for sample_set in "${sumo_sample_sets[@]}"; do
                    # out_dir should be suffixed with sample_set such that out dirs do not overlap 
                    # and to mark that this is a new algorithm
                    # sample_set's spaces are replaced with underscores
                    sample_out_dir="${out_dir}_$(echo "$sample_set" | tr ' ' '_')"

                    call_duaIterate "$sample_out_dir/$repetion_count" "${dua_args[@]}" sample-router--samples "$sample_set"
                done

                continue

            fi
            
            call_duaIterate "$out_dir/$repetion_count" "${dua_args[@]}"

        done
    done
    ((line_index++))
done < "$experiment"

echo "All experiments finished."