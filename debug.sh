#!/usr/bin/env bash

declare -a experiments=(
    "15 900"
)

base_dir="/home/henrik/rust_road_router"

export PATH="$base_dir:$base_dir/lib/InertialFlowCutter/build:$base_dir/target/debug/:$base_dir/lib/libsumo/bin/:$PATH"

for experiment in "${experiments[@]}"; do
    IFS=' ' read -r -a params <<< "$experiment"
    aggregation="${params[0]}"
    end_time="${params[1]}"

    out_dir=$base_dir/data/imported/sumo/grid.$aggregation-e$end_time.bak
    mkdir -p $out_dir
    cd $out_dir

    echo "Running experiment with aggregation: $aggregation and end time: $end_time"

    python "$base_dir/lib/libsumo/lib/python3.12/site-packages/sumo/tools/assign/duaIterate.py" \
        -n $base_dir/import/sumo/grid/grid.net.xml \
        -t $base_dir/import/sumo/grid/grid.trips.xml \
        --begin 0 \
        --aggregation "$aggregation" \
        --end "$end_time" \
        --routing-algorithm CCH \
        --mesosim \
        --router-verbose \
        sumo--aggregate-warnings 5 \
        sumo--ignore-route-errors \
        sumo--time-to-teleport.disconnected 1 \
        cch-preprocessor--input-prefix grid \
        cch-preprocessor--input-dir $base_dir/import/sumo/grid
done