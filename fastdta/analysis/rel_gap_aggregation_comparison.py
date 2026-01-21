#!/usr/bin/env python3
"""
Generate relative gap median comparison across aggregations.

Creates one plot per instance/network with three subplots showing
relative gap progress for aggregations 900s, 300s, and 60s side by side.

Usage:
    python rel_gap_aggregation_comparison.py --log <experiment.out> --csv <experiment.csv> [--out-dir <output>]
"""
from plots.styles import style_for_algo, get_all_algorithm_colors, get_display_label
from common import (
    build_model,
    DataModel,
    get_experiments_by_instance,
    get_relative_gaps,
    sanitize_for_filename,
)
import sys
import os
import argparse
from pathlib import Path
from typing import Dict, List, Tuple
from collections import defaultdict
import matplotlib.pyplot as plt
import numpy as np

# Add analysis directory to path for imports
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))


# Algorithm order for display
ALGORITHM_ORDER = [
    "cch",
    "dijkstra",
    "fastdta2",
    "fastdta_1_1",
    "fastdta_1_1_1",
    "fastdta_1_2_3_4",
]

# Aggregation order (left to right in plot)
AGGREGATIONS = [900, 300, 60]

# Network name mapping
NETWORK_NAMES = {
    "leopoldshafen": "Leopoldshafen",
    "rastatt": "Rastatt",
    "karlsruhe": "Karlsruhe",
}


def extract_network_name(prefix: str) -> str:
    """Extract readable network name from prefix."""
    prefix_lower = prefix.lower()
    for key, name in NETWORK_NAMES.items():
        if key in prefix_lower:
            return name
    return prefix  # Fallback to original


def group_by_aggregation(experiments, instance):
    """Group experiments by aggregation level."""
    by_agg = defaultdict(list)
    for exp in experiments:
        if instance.aggregation in AGGREGATIONS:
            by_agg[instance.aggregation].append(exp)
    return by_agg


def calculate_median_gaps(experiments) -> Tuple[List[int], List[float]]:
    """
    Calculate median relative gaps across repetitions for each iteration.
    Returns (iterations, median_values)
    """
    # Collect gaps per iteration across all experiments
    iter_gaps: Dict[int, List[float]] = defaultdict(list)

    for exp in experiments:
        gaps = get_relative_gaps(exp)
        for iteration, gap in gaps.items():
            iter_gaps[iteration].append(gap)

    if not iter_gaps:
        return [], []

    # Calculate median for each iteration
    iterations = sorted(iter_gaps.keys())
    medians = [np.median(iter_gaps[i]) for i in iterations]

    return iterations, medians


def create_aggregation_comparison_plot(dm: DataModel, prefix: str, out_dir: str, y_min: float = None):
    """Create comparison plot for one network across aggregations."""

    # Get all instances with this prefix (one per aggregation)
    instances_with_prefix = {
        idx: inst for idx, inst in dm.instances.items()
        if inst.prefix == prefix
    }

    if not instances_with_prefix:
        return

    # Get one instance for network name
    sample_instance = next(iter(instances_with_prefix.values()))

    # Group by aggregation and algorithm
    agg_algo_data: Dict[int, Dict[str, Tuple[List[int], List[float]]]] = {}

    for agg in AGGREGATIONS:
        agg_algo_data[agg] = {}

        # Find instances with this aggregation
        for inst_idx, inst in instances_with_prefix.items():
            if inst.aggregation != agg:
                continue

            # Get all experiments for this instance
            for exp in dm.experiments:
                if exp.instance_index != inst_idx:
                    continue

                algo = exp.algorithm
                if algo not in agg_algo_data[agg]:
                    agg_algo_data[agg][algo] = []

                # Collect gaps from this experiment
                gaps = get_relative_gaps(exp)
                agg_algo_data[agg][algo].append((exp, gaps))

    # Calculate medians for each algorithm and aggregation
    plot_data: Dict[int, Dict[str, Tuple[List[int], List[float]]]] = {}

    for agg in AGGREGATIONS:
        plot_data[agg] = {}
        for algo in ALGORITHM_ORDER:
            if algo not in agg_algo_data[agg]:
                continue

            # Collect all gaps by iteration
            iter_gaps: Dict[int, List[float]] = defaultdict(list)
            for exp, gaps in agg_algo_data[agg][algo]:
                for iteration, gap in gaps.items():
                    iter_gaps[iteration].append(gap)

            if iter_gaps:
                iterations = sorted(iter_gaps.keys())
                medians = [np.median(iter_gaps[i]) for i in iterations]
                plot_data[agg][algo] = (iterations, medians)

    # Check if we have data for any aggregation
    if not any(plot_data.values()):
        print(f"No data found for network {prefix}")
        return

    # Get consistent colors
    algo_colors = get_all_algorithm_colors(list(dm.algorithms))

    # Create figure with 3 subplots
    fig, axes = plt.subplots(1, 3, figsize=(11, 4.5), sharey=True)

    # Overall title
    network_name = extract_network_name(prefix)
    fig.suptitle(network_name, fontsize=16, fontweight='bold')

    # Plot each aggregation
    print(f"\nMinimum values for {network_name}:")
    for idx, agg in enumerate(AGGREGATIONS):
        ax = axes[idx]
        ax.set_title(f"{agg}s", fontsize=16)

        max_iter = 0
        print(f"  Aggregation {agg}s:")

        # Plot each algorithm
        for algo in ALGORITHM_ORDER:
            if algo not in plot_data[agg]:
                continue

            iterations, medians = plot_data[agg][algo]
            if not iterations:
                continue

            max_iter = max(max_iter, max(iterations))

            # Track and print minimum value
            min_value = min(medians)
            min_iter = iterations[medians.index(min_value)]
            display_label = get_display_label(algo)
            print(
                f"    {display_label:20s}: {min_value:.6e} (at iteration {min_iter})")

            algo_style = style_for_algo(algo)
            color = algo_style["color"]
            marker = algo_style["marker"]

            ax.plot(
                iterations, medians,
                label=display_label,
                color=color,
                marker=marker,
                markersize=5,
                linewidth=1.2,
                alpha=0.85,
                markevery=max(1, len(iterations) // 13)
            )

        # Style
        ax.set_yscale('log')
        ax.set_xlim(0, max(1, max_iter))
        ax.tick_params(axis='both', labelsize=13)
        ax.grid(True, linestyle="--", alpha=0.4)

        # Set x-axis to show equidistant ticks
        if max_iter > 0:
            # Create 5 equidistant ticks from 0 to max_iter
            num_ticks = 5
            tick_values = np.linspace(0, max_iter, num_ticks)
            # Round to integers for cleaner display
            tick_values = [int(round(t)) for t in tick_values]
            # Remove duplicates while preserving order
            seen = set()
            tick_values = [t for t in tick_values if not (
                t in seen or seen.add(t))]
            ax.set_xticks(tick_values)

        # Set y-axis limits and ticks
        if y_min is not None:
            ax.set_ylim(bottom=y_min)

        # Only show y-label on leftmost subplot
        if idx == 0:
            ax.set_ylabel("Relative Gap", fontsize=15)

        # X-label for middle subplot only
        if idx == 1:
            ax.set_xlabel("Iteration", fontsize=15)

    # Create single legend below all subplots
    handles, labels = axes[0].get_legend_handles_labels()
    if not handles:
        # Try other subplots if first one is empty
        for ax in axes[1:]:
            handles, labels = ax.get_legend_handles_labels()
            if handles:
                break

    if handles:
        fig.legend(
            handles, labels,
            loc='lower center',
            bbox_to_anchor=(0.5, -0.1),
            ncol=3,
            fontsize=14,
            frameon=False
        )

    # Adjust layout
    plt.tight_layout(rect=[0, 0.05, 1, 0.95])

    # Save
    os.makedirs(out_dir, exist_ok=True)
    filename = f"{sanitize_for_filename(prefix)}-rel-gap-agg-comparison.pdf"
    filepath = os.path.join(out_dir, filename)
    fig.savefig(filepath, bbox_inches='tight', dpi=150)
    plt.close(fig)

    print(f"Created: {filepath}")


def main():
    parser = argparse.ArgumentParser(
        description="Generate relative gap aggregation comparison plots",
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
    parser.add_argument(
        "--y-min",
        type=float,
        default=None,
        help="Minimum y-axis value for relative gap (optional)"
    )

    args = parser.parse_args()

    # Validate input files
    if not os.path.exists(args.log):
        print(f"Error: Log file not found: {args.log}", file=sys.stderr)
        sys.exit(1)

    if not os.path.exists(args.csv):
        print(f"Error: CSV file not found: {args.csv}", file=sys.stderr)
        sys.exit(1)

    # Build data model
    print("Parsing log and CSV files...")
    dm = build_model(args.log, args.csv)

    print(
        f"Found {len(dm.instances)} instances, {len(dm.experiments)} experiments")
    print(f"Algorithms: {', '.join(sorted(dm.algorithms))}")

    # Group experiments by base instance (ignoring aggregation)
    # We want to create one plot per network, showing different aggregations
    instances_by_prefix: Dict[str, List[int]] = defaultdict(list)

    for inst_idx, instance in dm.instances.items():
        # Use prefix as grouping key (this should be the network name)
        instances_by_prefix[instance.prefix].append(inst_idx)

    # Create plots
    print("\nGenerating plots...")
    for prefix, inst_indices in instances_by_prefix.items():
        # Create plot for this network prefix across all aggregations
        if inst_indices:
            create_aggregation_comparison_plot(
                dm, prefix, args.out_dir, y_min=args.y_min)

    print(f"\nPlots saved to: {args.out_dir}")


if __name__ == "__main__":
    main()
