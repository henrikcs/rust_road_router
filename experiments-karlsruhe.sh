#!/usr/bin/env sh

declare spack_env=$1
# if a flag "--debug" is provided, then set release_type to "debug"
# otherwise if no flag is provided then set to "release"

declare release_type="release"
if [ "$2" = "--debug" ]; then
    declare release_type="debug"


declare pwd=$(pwd)

echo "Path: "$pwd"/lib/InertialFlowCutter/build:"$pwd"/target/"$release_type":~/rust-nightly/bin:~/.local/bin:$PATH"

P=$(basename $(find ~/.user_spack/environments/"$spack_env"/.spack-env/._view -mindepth 1 -maxdepth 1 -type d))
export PATH="$pwd"/lib/InertialFlowCutter/build:"$pwd"/target/"$release_type":~/rust-nightly/bin:~/.local/bin:$PATH
export RL=$(spack location -i readline%gcc@14)
export NC=$(spack location -i ncurses%gcc@14)
export LX=$(spack location -i libx11%gcc@14)
export LIBRARY_PATH=$LX/lib:$RL/lib:$NC/lib:$LIBRARY_PATH
export CPATH=$RL/include:$NC/include:$LX/include
export LD_LIBRARY_PATH=~/.local/libnsl1/lib64:~/.user_spack/environments/"$spack_env"/.spack-env/._view/"$P"/lib:$NC/lib:$LD_LIBRARY_PATH

declare -a args=(
    "data/imported/sumo/karlsruhe-dbg import/sumo/karlsruhe karlsruhe 900"
)

# iterate over the array and run the command for each tuple
for arg in "${args[@]}"; do
    IFS=' ' read -r -a pair <<< "$arg"
    out_dir="$pwd/${pair[0]}"
    in_dir="$pwd/${pair[1]}"
    prefix="${pair[2]}"
    aggregation="${pair[3]}"
    echo "Processing output directory: $out_dir with input directory: $in_dir and prefix: $prefix with aggregation: $aggregation"
    net_file="$in_dir/$prefix.net.xml"
    trips_file="$in_dir/$prefix.trips.xml"

    mkdir -p "$out_dir"
    cd "$out_dir"

    python ~/rust_road_router/venvs/libsumo/lib/python3.11/site-packages/sumo/tools/assign/duaIterate.py \
    -n "$net_file" \
    -t "$trips_file" \
    --routing-algorithm CCH --mesosim --aggregation "$aggregation" --begin 0 --end 36000 \
    cch-preprocessor--input-prefix "$prefix" \
    cch-preprocessor--input-dir "$in_dir"

    mkdir -p "$out_dir-dijkstra"
    cd "$out_dir-dijkstra"

    # Run with Dijkstra
    python ~/rust_road_router/venvs/libsumo/lib/python3.11/site-packages/sumo/tools/assign/duaIterate.py \
    -n "$net_file" \
    -t "$trips_file" \
    --mesosim --aggregation "$aggregation" --begin 0 --end 36000
done
