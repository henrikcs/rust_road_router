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
from scipy import stats

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


def calculate_median_gaps(experiments, agg_method: str = "min") -> Tuple[List[int], List[float]]:
    """
    Calculate aggregated relative gaps across repetitions for each iteration.

    Args:
        experiments: List of experiments to aggregate
        agg_method: Aggregation method - "min", "mean", or "median"

    Returns (iterations, aggregated_values)
    """
    # Collect gaps per iteration across all experiments
    iter_gaps: Dict[int, List[float]] = defaultdict(list)

    for exp in experiments:
        gaps = get_relative_gaps(exp)
        for iteration, gap in gaps.items():
            iter_gaps[iteration].append(gap)

    if not iter_gaps:
        return [], []

    # Calculate aggregation for each iteration
    iterations = sorted(iter_gaps.keys())

    if agg_method == "min":
        aggregated = [np.min(iter_gaps[i]) for i in iterations]
    elif agg_method == "mean":
        aggregated = [np.mean(iter_gaps[i]) for i in iterations]
    elif agg_method == "median":
        aggregated = [np.median(iter_gaps[i]) for i in iterations]
    else:
        raise ValueError(f"Unknown aggregation method: {agg_method}")

    return iterations, aggregated


def compute_pairwise_statistics(plot_data: Dict[int, Dict[str, Tuple[List[int], List[float]]]], algos_to_plot: List[str], network_name: str):
    """Compute and print pairwise statistical comparisons between algorithms."""
    print(f"\n{'='*80}")
    print(f"Statistical Comparison for {network_name}")
    print(f"{'='*80}")

    for agg in AGGREGATIONS:
        if agg not in plot_data or len(plot_data[agg]) < 2:
            continue

        print(f"\nAggregation: {agg}s")
        print("-" * 80)

        # Get algorithms that have data for this aggregation
        available_algos = [
            algo for algo in algos_to_plot if algo in plot_data[agg]]

        if len(available_algos) < 2:
            print("  Not enough algorithms with data for comparison")
            continue

        # Pairwise comparisons
        for i, algo1 in enumerate(available_algos):
            for algo2 in available_algos[i+1:]:
                iter1, vals1 = plot_data[agg][algo1]
                iter2, vals2 = plot_data[agg][algo2]

                # Find common iterations
                common_iters = sorted(set(iter1) & set(iter2))
                if len(common_iters) < 2:
                    continue

                # Extract values for common iterations
                vals1_common = [vals1[iter1.index(it)] for it in common_iters]
                vals2_common = [vals2[iter2.index(it)] for it in common_iters]

                # Compute statistics
                mean_diff = np.mean(
                    np.array(vals1_common) - np.array(vals2_common))
                median_diff = np.median(
                    np.array(vals1_common) - np.array(vals2_common))

                # Mean absolute difference
                mad = np.mean(
                    np.abs(np.array(vals1_common) - np.array(vals2_common)))

                # Root mean square error
                rmse = np.sqrt(
                    np.mean((np.array(vals1_common) - np.array(vals2_common))**2))

                # Wilcoxon signed-rank test (paired, non-parametric)
                try:
                    wilcoxon_stat, wilcoxon_p = stats.wilcoxon(
                        vals1_common, vals2_common)
                except Exception as e:
                    wilcoxon_stat, wilcoxon_p = None, None

                # Paired t-test (assumes normality)
                try:
                    t_stat, t_p = stats.ttest_rel(vals1_common, vals2_common)
                except Exception as e:
                    t_stat, t_p = None, None

                # Cohen's d (effect size)
                pooled_std = np.sqrt(
                    (np.var(vals1_common) + np.var(vals2_common)) / 2)
                cohens_d = mean_diff / pooled_std if pooled_std > 0 else 0

                # Display labels
                label1 = get_display_label(algo1)
                label2 = get_display_label(algo2)

                print(f"\n  {label1} vs {label2}:")
                print(f"    N (common iterations):  {len(common_iters)}")
                print(f"    Mean difference:        {mean_diff:.6e}")
                print(f"    Median difference:      {median_diff:.6e}")
                print(f"    Mean absolute diff:     {mad:.6e}")
                print(f"    RMSE:                   {rmse:.6e}")
                if wilcoxon_p is not None:
                    print(f"    Wilcoxon p-value:       {wilcoxon_p:.6e}")
                if t_p is not None:
                    print(f"    Paired t-test p-value:  {t_p:.6e}")
                print(f"    Cohen's d:              {cohens_d:.4f}")

                # Interpretation
                if wilcoxon_p is not None:
                    if wilcoxon_p < 0.001:
                        sig_str = "highly significant (p < 0.001)"
                    elif wilcoxon_p < 0.01:
                        sig_str = "very significant (p < 0.01)"
                    elif wilcoxon_p < 0.05:
                        sig_str = "significant (p < 0.05)"
                    else:
                        sig_str = "not significant (p >= 0.05)"
                    print(f"    Significance:           {sig_str}")

    print(f"\n{'='*80}\n")


def create_aggregation_comparison_plot(dm: DataModel, prefix: str, out_dir: str, y_min: float = None, algorithms: List[str] = None, agg_method: str = "min"):
    """Create comparison plot for one network across aggregations.

    Args:
        dm: Data model
        prefix: Network prefix
        out_dir: Output directory
        y_min: Minimum y-axis value
        algorithms: List of algorithms to plot
        agg_method: Aggregation method for repetitions - "min", "mean", "median", "best", or a number (0 to n-1) to select specific repetition
    """

    # Use specified algorithms or default to all
    algos_to_plot = algorithms if algorithms else ALGORITHM_ORDER

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

    # For "best" method, we need to find the best repetition for each algo/agg combination
    if agg_method == "best":
        best_repetitions: Dict[Tuple[int, str],
                               int] = {}  # (agg, algo) -> rep_index

        for agg in AGGREGATIONS:
            for algo in algos_to_plot:
                if algo not in agg_algo_data[agg]:
                    continue

                # Find which repetition reached the lowest relative gap
                n_reps = len(agg_algo_data[agg][algo])
                if n_reps == 0:
                    continue

                best_rep = 0
                best_min_gap = float('inf')

                for rep_idx in range(n_reps):
                    exp, gaps = agg_algo_data[agg][algo][rep_idx]
                    if gaps:
                        min_gap = min(gaps.values())
                        if min_gap < best_min_gap:
                            best_min_gap = min_gap
                            best_rep = rep_idx

                best_repetitions[(agg, algo)] = best_rep

    for agg in AGGREGATIONS:
        plot_data[agg] = {}
        for algo in algos_to_plot:
            if algo not in agg_algo_data[agg]:
                continue

            # Collect all gaps by iteration
            iter_gaps: Dict[int, List[float]] = defaultdict(list)
            for exp, gaps in agg_algo_data[agg][algo]:
                for iteration, gap in gaps.items():
                    iter_gaps[iteration].append(gap)

            if iter_gaps:
                iterations = sorted(iter_gaps.keys())

                if agg_method == "best":
                    # Use the best repetition for this algo/agg combination
                    best_rep = best_repetitions.get((agg, algo), 0)
                    n_reps = len(iter_gaps[iterations[0]]) if iterations else 0
                    if n_reps > 0:
                        aggregated = [iter_gaps[i][best_rep]
                                      for i in iterations]
                    else:
                        aggregated = []
                else:
                    # Check if agg_method is an integer (specific repetition index)
                    try:
                        rep_index = int(agg_method)
                        # Use specific repetition (with modulo to handle out-of-range indices)
                        n_reps = len(
                            iter_gaps[iterations[0]]) if iterations else 0
                        if n_reps > 0:
                            actual_index = rep_index % n_reps
                            aggregated = [iter_gaps[i][actual_index]
                                          for i in iterations]
                        else:
                            aggregated = []
                    except (ValueError, TypeError):
                        # agg_method is a string, use aggregation function
                        if agg_method == "min":
                            aggregated = [np.min(iter_gaps[i])
                                          for i in iterations]
                        elif agg_method == "mean":
                            aggregated = [np.mean(iter_gaps[i])
                                          for i in iterations]
                        elif agg_method == "median":
                            aggregated = [np.median(iter_gaps[i])
                                          for i in iterations]
                        else:
                            raise ValueError(
                                f"Unknown aggregation method: {agg_method}")
                plot_data[agg][algo] = (iterations, aggregated)

    # Check if we have data for any aggregation
    if not any(plot_data.values()):
        print(f"No data found for network {prefix}")
        return

    # Compute and print statistical comparisons
    network_name = extract_network_name(prefix)
    compute_pairwise_statistics(plot_data, algos_to_plot, network_name)

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
        for algo in algos_to_plot:
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
    filename = f"{sanitize_for_filename(prefix)}-rel-gap-agg-comparison-{agg_method}.pdf"
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
    parser.add_argument(
        "--algorithms",
        nargs="+",
        default=None,
        help="List of algorithms to plot (e.g., dijkstra-rust cch). If not specified, all algorithms are plotted."
    )
    parser.add_argument(
        "--agg-method",
        type=str,
        default="min",
        help="Method to aggregate repetitions: 'min', 'mean', 'median', 'best', or a number (0 to n-1) to select a specific repetition (default: min)"
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
    print(f"Using aggregation method: {args.agg_method}")
    if args.algorithms:
        print(f"Filtering algorithms: {', '.join(args.algorithms)}")
    for prefix, inst_indices in instances_by_prefix.items():
        # Create plot for this network prefix across all aggregations
        if inst_indices:
            create_aggregation_comparison_plot(
                dm, prefix, args.out_dir, y_min=args.y_min, algorithms=args.algorithms, agg_method=args.agg_method)

    print(f"\nPlots saved to: {args.out_dir}")


if __name__ == "__main__":
    main()
