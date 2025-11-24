# SUMO Sample Router

A new binary for Dynamic Traffic Assignment (DTA) that uses SUMO simulation directly instead of traffic models.

## Overview

The `sumo-sample-router` performs sample-based routing where each sample is simulated using SUMO to obtain realistic travel times. Unlike `sumo-fastdta-router` which estimates densities and velocities using traffic models, this router invokes SUMO directly for each batch of sampled queries.

## Workflow

1. **Sample Generation**: Queries are divided into batches according to the specified sample sizes
2. **Batch Processing**: For each batch:
   - Calculate shortest paths for the sampled queries using CCH on the current graph
   - Combine these paths with all previously routed paths
   - Write all routes to a `.rou.xml` file
   - Generate an additional file for SUMO edgeData output
   - Run SUMO simulation with all accumulated routes
   - Read the resulting dump file (meandata)
   - Update graph weights based on the simulation results
3. **Iteration Complete**: After all batches, prepare the next iteration with choice model and relative gap calculation

## Key Features

- **Direct SUMO Simulation**: Uses actual SUMO simulation instead of analytical traffic models
- **Batch-by-Batch Updates**: Graph weights are updated after each batch, reflecting cumulative traffic
- **Dump File Preservation**: All dump files are preserved for analysis (`dump_<aggregation>_<iteration>_<batch>.xml`)
- **Incremental Routing**: Each batch builds upon previous batches' routes

## Usage

```bash
sumo-sample-router \
    --input-dir <directory> \
    --input-prefix <prefix> \
    --iteration <number> \
    --net-file <path-to-sumo-net.xml> \
    --aggregation <seconds> \
    --samples "<sample1> <sample2> ..." \
    --route-choice-method <gawron|logit> \
    --max-alternatives <number> \
    [--seed <number>]
```

### Required Arguments

- `--input-dir`: Directory containing preprocessed graph data
- `--input-prefix`: Prefix for output files
- `--iteration`: Current iteration number
- `--net-file`: Path to SUMO network file (.net.xml)
- `--aggregation`: Aggregation interval for SUMO edgeData (in seconds, e.g., 60)
- `--samples`: Space-separated sample fractions (e.g., "0.1 0.2 0.3 0.4")

### Optional Arguments

- `--route-choice-method`: Choice algorithm (default: gawron)
- `--max-alternatives`: Maximum number of alternative paths to maintain
- `--seed`: Random seed for reproducibility
- `--no-write-sumo-alternatives`: Skip writing alternative routes file

## Files Generated

### Per Batch

- `routes_batch_<batch>.rou.xml`: SUMO routes file for the batch
- `additional_batch_<batch>.xml`: SUMO additional file configuration
- `dump_<aggregation>_<iteration>_<batch>.xml`: SUMO edgeData output (preserved)

### Per Iteration

- `<prefix>_<iteration>.rou.xml`: Final routes for next iteration
- `<prefix>_<iteration>.rou.alt.xml`: Alternative routes (if enabled)
- DTA data for alternative paths and choices

## Additional File Format

The additional file generated for each batch has this structure:

```xml
<a>
    <edgeData id="dump_<aggregation>" freq="<aggregation>" file="dump_<aggregation>_<iteration>_<batch>.xml" excludeEmpty="true" minSamples="1"/>
</a>
```

This configures SUMO to output edge statistics at the specified aggregation interval.

## Implementation Details

### New Modules

1. **`sumo_runner.rs`**: Library for invoking SUMO and generating additional files
2. **`sumo_routes_writer.rs`**: Functions to serialize routes as SUMO XML
3. **`sumo_sample_routing.rs`**: Main routing logic with SUMO integration

### Key Function

`get_paths_by_samples_with_sumo()` implements the core workflow:

- Iterates through samples
- Routes each sample using CCH
- Accumulates all routed paths
- Runs SUMO simulation
- Updates graph from dump file

## Comparison with FastDTA

| Feature                | sumo-fastdta-router                | sumo-sample-router       |
| ---------------------- | ---------------------------------- | ------------------------ |
| Travel Time Estimation | Traffic model (Modified Lee)       | Direct SUMO simulation   |
| Speed                  | Faster (analytical)                | Slower (simulation)      |
| Accuracy               | Model-dependent                    | SUMO-accurate            |
| Calibration            | Requires traffic model calibration | No calibration needed    |
| Batch Output           | Not preserved                      | All dump files preserved |

## Example

```bash
# Run iteration 0 with 4 batches
sumo-sample-router \
    --input-dir ./data/imported/sumo/capa \
    --input-prefix capa \
    --iteration 0 \
    --net-file ./import/sumo/capa/capa.net.xml \
    --aggregation 60 \
    --samples "0.25 0.25 0.25 0.25" \
    --route-choice-method logit \
    --logitbeta 1.0 \
    --logitgamma 1.0 \
    --logittheta 1.0 \
    --max-alternatives 5
```

This will:

1. Split queries into 4 equal batches (25% each)
2. Route and simulate each batch incrementally
3. Preserve 4 dump files: `dump_60_000_000.xml` through `dump_60_000_003.xml`
4. Generate routes for iteration 1 based on logit choice model
