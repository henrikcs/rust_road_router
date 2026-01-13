#!/usr/bin/env python3
"""
Generate multi-network comparison plot showing router duration by algorithm and aggregation.

Creates a grid of subplots (2-3 rows x 2 columns):
- Rows: Networks (Leopoldshafen, Rastatt, optionally Karlsruhe)
- Columns: Sequential vs Parallel
- Each subplot shows router duration boxplots grouped by algorithm
- Each algorithm group has 3 boxplots for different aggregations (60, 300, 900)
"""
from plots.styles import style_for_algo, get_all_algorithm_colors, get_display_label
from common import (
    build_model,
    DataModel,
    get_experiments_by_instance,
    get_routing_times,
)
import argparse
import os
import sys
from typing import Dict, List, Tuple, Optional
import matplotlib.pyplot as plt
import numpy as np
from matplotlib.patches import Rectangle

# Add analysis directory to path for imports
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))


# Fixed algorithm order
ALGORITHM_ORDER = [
    "fastdta2",
    "fastdta_1_1",
    "fastdta_1_1_1",
    "fastdta_1_2_3_4",
    "dijkstra-rust",
    "cch",
    "dijkstra",
    "astar",
    "ch",
]

# Aggregation values to expect (in order for display)
AGGREGATIONS = [900, 300, 60]

# Network names for row labels
NETWORK_NAMES = {
    "leopoldshafen": "Leopoldshafen",
    "rastatt": "Rastatt",
    "karlsruhe": "Karlsruhe",
}


def parse_data_for_subplot(dm: DataModel) -> Dict[Tuple[str, int], List[float]]:
    """
    Parse routing times from data model, grouped by algorithm and aggregation.
    Returns: {(algorithm, aggregation): [routing_times]}
    """
    result: Dict[Tuple[str, int], List[float]] = {}

    exp_by_instance = get_experiments_by_instance(dm)

    for instance_idx, exps in exp_by_instance.items():
        instance = dm.instances.get(instance_idx)
        if not instance:
            continue

        aggregation = int(instance.aggregation)

        for exp in exps:
            times = get_routing_times(exp, skip_first=True)
            if times:
                key = (exp.algorithm, aggregation)
                result.setdefault(key, []).extend(times)

    return result


def create_grouped_boxplot(ax, data: Dict[Tuple[str, int], List[float]],
                           algo_colors: Dict[str, str], title: str) -> Dict[Tuple[str, int], float]:
    """
    Create boxplot with algorithms grouped, each showing 3 aggregations.

    Args:
        ax: Matplotlib axis
        data: {(algorithm, aggregation): [routing_times]}
        algo_colors: {algorithm: color}
        title: Subplot title

    Returns:
        Dictionary of median values: {(algorithm, aggregation): median}
    """
    # Group width and spacing parameters
    group_width = 0.8  # Width for 3 boxplots within a group
    box_width = 0.25   # Width of individual box
    group_spacing = 0.6  # Space between algorithm groups

    positions = []
    plot_data = []
    colors = []
    labels = []
    tick_positions = []
    tick_labels = []
    medians = {}  # Store median values

    current_pos = 1.0

    for algo in ALGORITHM_ORDER:
        algo_data = []
        algo_found = False

        # Collect data for this algorithm across aggregations
        for agg in AGGREGATIONS:
            key = (algo, agg)
            if key in data and data[key]:
                algo_data.append(data[key])
                algo_found = True
            else:
                algo_data.append([])  # Empty data for missing aggregation

        if not algo_found:
            continue  # Skip algorithm if no data at all

        # Calculate positions for this algorithm's boxplots
        # Center the group at current_pos
        group_start = current_pos - group_width / 2

        for i, agg_data in enumerate(algo_data):
            if agg_data:  # Only add if there's data
                pos = group_start + i * box_width
                positions.append(pos)
                plot_data.append(agg_data)
                colors.append(algo_colors.get(algo, "#999999"))
                labels.append(f"{AGGREGATIONS[i]}")
                # Store median
                medians[(algo, AGGREGATIONS[i])] = np.median(agg_data)

        # Store tick position at center of group
        tick_positions.append(current_pos)
        tick_labels.append(get_display_label(algo))

        # Move to next group
        current_pos += group_width + group_spacing

    if not plot_data:
        ax.text(0.5, 0.5, "No data available",
                ha='center', va='center', transform=ax.transAxes)
        return {}

    # Create boxplots
    bp = ax.boxplot(plot_data, positions=positions, patch_artist=True,
                    widths=box_width, manage_ticks=False)

    # Color boxes
    for patch, color in zip(bp['boxes'], colors):
        patch.set_facecolor(color)
        patch.set_alpha(0.7)

    # Style whiskers, caps, medians
    for element in ['whiskers', 'caps', 'medians']:
        for item in bp[element]:
            item.set_color('black')
            item.set_linewidth(1)

    # Set x-axis ticks at algorithm group centers
    ax.set_xticks(tick_positions)
    ax.set_xticklabels(tick_labels, rotation=45, ha="right", fontsize=13)

    # Set labels and title
    ax.set_ylabel("Router Duration (s)", fontsize=15)
    ax.set_title(title, fontsize=12, fontweight='bold')
    ax.tick_params(axis='y', labelsize=13)
    ax.grid(True, axis="y", linestyle="--", alpha=0.3)

    return medians


def extract_network_name(filepath: str) -> Optional[str]:
    """
    Try to extract network name from filepath.
    Looks for patterns like 'leopoldshafen', 'rastatt', 'karlsruhe' in path.
    """
    filepath_lower = filepath.lower()
    for key in NETWORK_NAMES.keys():
        if key in filepath_lower:
            return key
    return None


def main():
    parser = argparse.ArgumentParser(
        description='Generate multi-network router duration comparison plot',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Example:
  %(prog)s \\
    --lh-seq lh_seq.out lh_seq.csv \\
    --lh-par lh_par.out lh_par.csv \\
    --ra-seq ra_seq.out ra_seq.csv \\
    --ra-par ra_par.out ra_par.csv \\
    --out comparison.pdf
        """
    )

    # Leopoldshafen inputs
    parser.add_argument('--lh-seq', nargs=2, metavar=('LOG', 'CSV'),
                        help='Leopoldshafen sequential: log and csv files')
    parser.add_argument('--lh-par', nargs=2, metavar=('LOG', 'CSV'),
                        help='Leopoldshafen parallel: log and csv files')

    # Rastatt inputs
    parser.add_argument('--ra-seq', nargs=2, metavar=('LOG', 'CSV'),
                        help='Rastatt sequential: log and csv files')
    parser.add_argument('--ra-par', nargs=2, metavar=('LOG', 'CSV'),
                        help='Rastatt parallel: log and csv files')

    # Karlsruhe inputs (optional)
    parser.add_argument('--ka-seq', nargs=2, metavar=('LOG', 'CSV'),
                        help='Karlsruhe sequential: log and csv files (optional)')
    parser.add_argument('--ka-par', nargs=2, metavar=('LOG', 'CSV'),
                        help='Karlsruhe parallel: log and csv files (optional)')

    parser.add_argument('--out', '-o', required=True,
                        help='Output file path (e.g., comparison.pdf)')

    parser.add_argument('--hide-ch', action='store_true',
                        help='Hide CH algorithm from plots (useful to avoid outliers)')

    args = parser.parse_args()

    # Validate that we have at least Leopoldshafen and Rastatt
    if not (args.lh_seq and args.lh_par and args.ra_seq and args.ra_par):
        parser.error(
            "Leopoldshafen and Rastatt data (both seq and par) are required")

    # Determine number of rows
    has_karlsruhe = bool(args.ka_seq and args.ka_par)
    num_rows = 3 if has_karlsruhe else 2

    # Load all data
    configs = [
        ("leopoldshafen", "Sequential", args.lh_seq),
        ("leopoldshafen", "Parallel", args.lh_par),
        ("rastatt", "Sequential", args.ra_seq),
        ("rastatt", "Parallel", args.ra_par),
    ]

    if has_karlsruhe:
        configs.extend([
            ("karlsruhe", "Sequential", args.ka_seq),
            ("karlsruhe", "Parallel", args.ka_par),
        ])

    # Parse all data models
    print("Loading data...")
    subplot_data = {}
    all_algorithms = set()

    for network, mode, (log_file, csv_file) in configs:
        print(f"  Parsing {network} {mode}...")
        try:
            dm = build_model(log_file, csv_file)
            data = parse_data_for_subplot(dm)

            # Filter out CH if requested
            if args.hide_ch:
                data = {k: v for k, v in data.items() if k[0].lower() != 'ch'}

            subplot_data[(network, mode)] = data

            # Collect all algorithms for color consistency
            for (algo, agg) in data.keys():
                all_algorithms.add(algo)
        except Exception as e:
            print(f"    Error loading {network} {mode}: {e}")
            sys.exit(1)

    # Get consistent colors for all algorithms
    algo_colors = get_all_algorithm_colors(sorted(all_algorithms))

    # Create figure with subplots
    fig_width = 16
    fig_height = 6 * num_rows
    fig, axes = plt.subplots(num_rows, 2, figsize=(fig_width, fig_height))

    if num_rows == 1:
        axes = axes.reshape(1, -1)

    # Plot each subplot and collect medians
    print("\n=== Median Values ===")
    row_idx = 0
    for network in ["leopoldshafen", "rastatt", "karlsruhe"]:
        if network == "karlsruhe" and not has_karlsruhe:
            continue

        network_display = NETWORK_NAMES[network]

        # Sequential (left column)
        data_seq = subplot_data.get((network, "Sequential"), {})
        title_seq = f"{network_display} - Sequential"
        medians_seq = create_grouped_boxplot(
            axes[row_idx, 0], data_seq, algo_colors, title_seq)

        print(f"\n{title_seq}:")
        for (algo, agg), median in sorted(medians_seq.items()):
            print(f"  {algo:20s} agg={agg:3d}s: {median:.4f}s")

        # Parallel (right column)
        data_par = subplot_data.get((network, "Parallel"), {})
        title_par = f"{network_display} - Parallel"
        medians_par = create_grouped_boxplot(
            axes[row_idx, 1], data_par, algo_colors, title_par)

        print(f"\n{title_par}:")
        for (algo, agg), median in sorted(medians_par.items()):
            print(f"  {algo:20s} agg={agg:3d}s: {median:.4f}s")

        # Share y-axis limits within row for easy comparison
        y_mins = []
        y_maxs = []
        for ax in [axes[row_idx, 0], axes[row_idx, 1]]:
            ylim = ax.get_ylim()
            y_mins.append(ylim[0])
            y_maxs.append(ylim[1])

        if y_mins and y_maxs:
            common_ymin = min(y_mins)
            common_ymax = max(y_maxs)
            axes[row_idx, 0].set_ylim(common_ymin, common_ymax)
            axes[row_idx, 1].set_ylim(common_ymin, common_ymax)

        row_idx += 1

    # Adjust layout
    plt.tight_layout()

    # Save figure
    print(f"\nSaving plot to: {args.out}")
    os.makedirs(os.path.dirname(args.out) or '.', exist_ok=True)
    fig.savefig(args.out, bbox_inches='tight', dpi=150)
    plt.close(fig)

    print("Done!")


if __name__ == "__main__":
    main()
