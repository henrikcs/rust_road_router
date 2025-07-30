#!/usr/bin/env sh

declare spack_env=$1



declare release_type="release"
if [ "$2" = "--debug" ]; then
    declare release_type="debug"
fi

declare pwd=$(pwd)

P=$(basename $(find ~/.user_spack/environments/"$spack_env"/.spack-env/._view -mindepth 1 -maxdepth 1 -type d))
export PATH="$pwd"/lib/InertialFlowCutter/build:"$pwd"/target/"$release_type":~/rust-nightly/bin:~/.local/bin:$PATH
export RL=$(spack location -i readline%gcc@14)
export NC=$(spack location -i ncurses%gcc@14)
export LX=$(spack location -i libx11%gcc@14)
export LIBRARY_PATH=$LX/lib:$RL/lib:$NC/lib:$LIBRARY_PATH
export CPATH=$RL/include:$NC/include:$LX/include
export LD_LIBRARY_PATH=~/.local/libnsl1/lib64:~/.user_spack/environments/"$spack_env"/.spack-env/._view/"$P"/lib:$NC/lib:$LD_LIBRARY_PATH

declare -a args=(
    "data/imported/sumo/example-data-0 import/sumo/example-data example-data 900"
    "data/imported/sumo/example-data-1 import/sumo/example-data example-data 100"
)

# copy duaIterate.py from fastdta to the venv directory
cp ~/rust_road_router/fastdta/duaIterate.py ~/rust_road_router/venvs/libsumo/lib/python3.11/site-packages/sumo/tools/assign

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
    --mesosim --aggregation "$aggregation" --clean-alt --begin 0 --end 86400 -f 0 -l 30 --routing-algorithm CCH  \
    cch-preprocessor--input-prefix "$prefix" \
    cch-preprocessor--input-dir "$in_dir" \
    cch-router--no-write-sumo-alternatives

    mkdir -p "$out_dir-dijkstra"
    cd "$out_dir-dijkstra"

    # Run with Dijkstra
    python ~/rust_road_router/venvs/libsumo/lib/python3.11/site-packages/sumo/tools/assign/duaIterate.py \
    -n "$net_file" \
    -t "$trips_file" \
    --mesosim --aggregation "$aggregation" --clean-alt --begin 0 --end 86400 -f 0 -l 30 --routing-algorithm CCH  \
done
