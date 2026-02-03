#!/usr/bin/env python3
"""
Generate a single relative gap plot for CATCHUp on Leopoldshafen (60s aggregation).

Creates a minimal plot showing relative gap vs iteration:
- Logarithmic y-axis
- No title, no legend
- Only axis labels and tick labels
- Width is 3x height for effective vertical space usage

Usage:
    python rel_gap_single_plot.py --log <experiment.out> --csv <experiment.csv> [--out-dir <output>]
"""
from plots.styles import style_for_algo
from common import (
    build_model,
    DataModel,
    get_relative_gaps,
)
import sys
import os
import argparse
from typing import Dict, List, Tuple
from collections import defaultdict
import matplotlib.pyplot as plt
import numpy as np

# Add analysis directory to path for imports
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))


# Target configuration
TARGET_NETWORK = "leopoldshafen"
TARGET_AGGREGATION = 60
TARGET_ALGORITHM = "cch"


def find_best_run(dm: DataModel) -> Tuple[List[int], List[float]]:
    """
    Find the best run of CATCHUp on Leopoldshafen with 60s aggregation.

    Returns (iterations, relative_gaps) for the run that achieved the lowest final gap.
    """
    # Find the instance with leopoldshafen prefix and 60s aggregation
    target_instance_idx = None
    for idx, inst in dm.instances.items():
        if TARGET_NETWORK in inst.prefix.lower() and inst.aggregation == TARGET_AGGREGATION:
            target_instance_idx = idx
            break

    if target_instance_idx is None:
        print(
            f"No instance found with network '{TARGET_NETWORK}' and aggregation {TARGET_AGGREGATION}s")
        return [], []

    # Find all experiments for this instance with the target algorithm
    matching_experiments = []
    for exp in dm.experiments:
        if exp.instance_index == target_instance_idx and exp.algorithm == TARGET_ALGORITHM:
            matching_experiments.append(exp)

    if not matching_experiments:
        print(
            f"No experiments found for algorithm '{TARGET_ALGORITHM}' on instance {target_instance_idx}")
        return [], []

    # Find the best run (lowest minimum relative gap)
    best_exp = None
    best_min_gap = float('inf')

    for exp in matching_experiments:
        gaps = get_relative_gaps(exp)
        if gaps:
            min_gap = min(gaps.values())
            if min_gap < best_min_gap:
                best_min_gap = min_gap
                best_exp = exp

    if best_exp is None:
        print("No valid relative gap data found")
        return [], []

    # Extract iterations and gaps
    gaps = get_relative_gaps(best_exp)
    iterations = sorted(gaps.keys())
    values = [gaps[i] for i in iterations]

    print(f"Found best run: repetition {best_exp.repetition}")
    print(f"  Iterations: {len(iterations)}")
    print(f"  Min gap: {best_min_gap:.6e}")

    return iterations, values


def create_minimal_plot(iterations: List[int], values: List[float], out_dir: str):
    """
    Create a minimal plot with no title, no legend.
    Width is 3x height.
    """
    if not iterations or not values:
        print("No data to plot")
        return

    # Figure size: width = 3 * height
    height = 2.0
    width = 3 * height

    fig, ax = plt.subplots(figsize=(width, height))

    # Get style for CATCHUp
    algo_style = style_for_algo(TARGET_ALGORITHM)
    color = algo_style["color"]
    marker = algo_style["marker"]

    # Plot the data
    ax.plot(
        iterations, values,
        color=color,
        marker=marker,
        markersize=3,
        linewidth=1.0,
        markevery=max(1, len(iterations) // 10)
    )

    # Logarithmic y-axis
    ax.set_yscale('log')

    # Axis labels
    ax.set_xlabel("Iteration", fontsize=10)
    ax.set_ylabel("Relative Gap", fontsize=10)

    # Set x-axis limits
    ax.set_xlim(0, max(iterations))

    # Configure ticks
    ax.tick_params(axis='both', labelsize=9)

    # Set x-axis to show equidistant ticks
    max_iter = max(iterations)
    if max_iter > 0:
        num_ticks = 11
        tick_values = np.linspace(0, max_iter, num_ticks)
        tick_values = [int(round(t)) for t in tick_values]
        # Remove duplicates while preserving order
        seen = set()
        tick_values = [t for t in tick_values if not (
            t in seen or seen.add(t))]
        ax.set_xticks(tick_values)

    # Light grid
    ax.grid(True, linestyle="--", alpha=0.3)

    # Tight layout for effective space usage
    plt.tight_layout()

    # Save
    os.makedirs(out_dir, exist_ok=True)
    filename = f"{TARGET_NETWORK}-{TARGET_ALGORITHM}-{TARGET_AGGREGATION}s-rel-gap.png"
    filepath = os.path.join(out_dir, filename)
    fig.savefig(filepath, bbox_inches='tight', dpi=300)
    plt.close(fig)

    print(f"Created: {filepath}")


def main():
    parser = argparse.ArgumentParser(
        description="Generate minimal relative gap plot for CATCHUp on Leopoldshafen (60s)",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--log", "-l",
        required=True,
        help="Path to the .out log file from run_experiments.sh"
    )
    parser.add_argument(
        "--csv", "-c",
        required=True,
        help="Path to the .csv input file used for experiments"
    )
    parser.add_argument(
        "--out-dir", "-o",
        default="plots_out",
        help="Output directory for generated plots (default: plots_out)"
    )

    args = parser.parse_args()

    # Validate input files
    if not os.path.exists(args.log):
        print(f"Error: Log file not found: {args.log}")
        sys.exit(1)

    if not os.path.exists(args.csv):
        print(f"Error: CSV file not found: {args.csv}")
        sys.exit(1)

    # Build data model
    print("Parsing log and CSV files...")
    dm = build_model(args.log, args.csv)

    print(
        f"Found {len(dm.instances)} instances, {len(dm.experiments)} experiments")
    print(f"Algorithms: {', '.join(sorted(dm.algorithms))}")

    # Find the best run
    print(
        f"\nLooking for best {TARGET_ALGORITHM} run on {TARGET_NETWORK} ({TARGET_AGGREGATION}s)...")
    iterations, values = find_best_run(dm)

    # Create the plot
    if iterations and values:
        print("\nGenerating plot...")
        create_minimal_plot(iterations, values, args.out_dir)
        print(f"\nPlot saved to: {args.out_dir}")
    else:
        print("No data found to plot")
        sys.exit(1)


if __name__ == "__main__":
    main()
