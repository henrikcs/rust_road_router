#!/usr/bin/env python3
"""
Generate routing phase breakdown comparison plots grouped by network.

Creates one plot per network (Leopoldshafen, Rastatt, Karlsruhe) with three subplots
showing stacked bar charts of routing phases for different aggregations (900s, 300s, 60s).

Usage:
    python routing_phase_breakdown_by_network.py --log <experiment.out> --csv <experiment.csv> [--out-dir <output>]
"""
from plots.styles import get_display_label
from common import (
    build_model,
    DataModel,
    Experiment,
    get_experiments_by_instance,
)
import argparse
import os
import sys
import matplotlib.pyplot as plt
import numpy as np
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from collections import defaultdict

# Add analysis directory to path for imports
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))


# Algorithm order for display
ALGORITHM_ORDER = [
    "cch",
    "dijkstra-rust",
    "fastdta2",
    "fastdta_1_1",
    "fastdta_1_1_1",
    "fastdta_1_2_3_4",
]

# Define consistent colors for phases across all algorithms
PHASE_COLORS = {
    # Common phases
    "preprocessing": "#E8E8E8",
    "postprocessing": "#D0D0D0",

    # FastDTA specific phases
    "calibration": "#CC79A7",
    "sample": "#D55E00",
    "choice model": "#9467BD",
    "adjust weights": "#5FAD56",  # Green shade

    # Reading phases
    "read edge ids": "#B0B0B0",
    "read queries": "#A0A0A0",
}

# Colors for phase families (multiple bars, same color)
ROUTING_COLOR = "#56B4E9"
CUSTOMIZATION_COLOR = "#E69F00"

DEFAULT_PHASE_COLOR = "#999999"

# Aggregation order (left to right in plot)
AGGREGATIONS = [900, 300, 60]

# Network name mapping
NETWORK_NAMES = {
    "leopoldshafen": "Leopoldshafen",
    "rastatt": "Rastatt",
    "karlsruhe": "Karlsruhe",
}


def normalize_phase_name(phase_name: str) -> str:
    """Normalize phase name for consistent coloring."""
    phase_lower = phase_name.lower()

    # Handle routing phases
    if "routing" in phase_lower:
        return "routing"

    # Handle customization phases
    if "customization" in phase_lower:
        return "customization"

    # Handle known specific phases
    known_phases = [
        "preprocessing", "postprocessing", "calibration",
        "sample", "choice model", "adjust weights",
        "read edge ids", "read queries"
    ]

    for known in known_phases:
        if known in phase_lower:
            return known

    return phase_name


def get_phase_color(phase_name: str) -> str:
    """Get color for a phase."""
    normalized = normalize_phase_name(phase_name)

    if normalized == "routing":
        return ROUTING_COLOR
    elif normalized == "customization":
        return CUSTOMIZATION_COLOR
    elif normalized in PHASE_COLORS:
        return PHASE_COLORS[normalized]
    else:
        return DEFAULT_PHASE_COLOR


def get_phase_legend_name(phase_name: str) -> str:
    """Get legend label for a phase."""
    normalized = normalize_phase_name(phase_name)

    if normalized == "routing":
        return "Routing"
    elif normalized == "customization":
        return "Customization"
    elif normalized == "calibration":
        return "Calibration"
    elif normalized == "sample":
        return "Sample"
    elif normalized == "choice model":
        return "Choice Model"
    elif normalized == "adjust weights":
        return "Adjust Weights"
    elif normalized == "preprocessing":
        return "Preprocessing"
    elif normalized == "postprocessing":
        return "Postprocessing"
    elif normalized == "read edge ids":
        return "Read Edge IDs"
    elif normalized == "read queries":
        return "Read Queries"
    else:
        return phase_name.title()


def get_phase_times_for_experiment(exp: Experiment, skip_first: bool = True) -> Dict[str, List[float]]:
    """Get phase times grouped by phase name for an experiment."""
    phase_times: Dict[str, List[float]] = defaultdict(list)

    for step in exp.steps:
        if skip_first and step.iteration == 0:
            continue

        for pd in step.phase_details:
            if pd.duration_seconds is not None:
                normalized_name = normalize_phase_name(pd.phase_name)
                phase_times[normalized_name].append(pd.duration_seconds)

    return dict(phase_times)


def calculate_average_phase_times(experiments: List[Experiment]) -> Dict[str, float]:
    """Calculate average time for each phase across all experiments."""
    all_phase_times: Dict[str, List[float]] = defaultdict(list)

    for exp in experiments:
        phase_times = get_phase_times_for_experiment(exp, skip_first=True)
        for phase_name, times in phase_times.items():
            all_phase_times[phase_name].extend(times)

    # Calculate averages
    avg_times = {}
    for phase_name, times in all_phase_times.items():
        if times:
            avg_times[phase_name] = sum(times) / len(times)

    return avg_times


def create_broken_axis_subplot(
    ax,
    algorithms: List[str],
    phase_data: Dict[str, Dict[str, float]],
    break_lower: float,
    break_upper: float,
    show_ylabel: bool = True,
    title: str = ""
):
    """
    Create a stacked bar chart with broken y-axis on a single subplot.
    Uses divider to split the axis into two parts.
    """
    from mpl_toolkits.axes_grid1 import make_axes_locatable
    import matplotlib.patches as mpatches

    # Calculate max value to determine upper limit
    max_val = 0.0
    for algo_data in phase_data.values():
        total = sum(algo_data.values())
        max_val = max(max_val, total)

    # Add some padding to max value
    upper_limit = max_val * 1.05

    # Hide the main axis
    ax.axis('off')

    # Create two sub-axes manually within the main axis
    # Upper subplot (for higher values)
    ax_upper = ax.figure.add_axes(ax.get_position(), frameon=True)
    ax_lower = ax.figure.add_axes(ax.get_position(), frameon=True)

    # Calculate the relative heights
    total_range = (break_lower - 0) + (upper_limit - break_upper)
    lower_fraction = (break_lower - 0) / total_range
    upper_fraction = (upper_limit - break_upper) / total_range

    # Adjust positions
    pos = ax.get_position()
    gap = 0.02  # Gap between the two axes

    # Lower axis at bottom
    lower_height = pos.height * lower_fraction * (1 - gap)
    ax_lower.set_position([pos.x0, pos.y0, pos.width, lower_height])

    # Upper axis at top
    upper_height = pos.height * upper_fraction * (1 - gap)
    upper_y0 = pos.y0 + lower_height + pos.height * gap
    ax_upper.set_position([pos.x0, upper_y0, pos.width, upper_height])

    # Create the same stacked bars on both axes (no y-labels on individual axes)
    legend_handles_lower, legend_labels_lower = create_stacked_bar_chart_on_axis(
        ax_lower,
        algorithms,
        phase_data,
        show_ylabel=False
    )

    legend_handles_upper, legend_labels_upper = create_stacked_bar_chart_on_axis(
        ax_upper,
        algorithms,
        phase_data,
        show_ylabel=False
    )

    # Set y-axis limits
    ax_lower.set_ylim(0, break_lower)
    ax_upper.set_ylim(break_upper, upper_limit)

    # Add centered y-axis label if needed
    if show_ylabel:
        # Calculate the vertical center between the two axes
        lower_pos = ax_lower.get_position()
        upper_pos = ax_upper.get_position()

        # Center y position between the two axes
        center_y = (lower_pos.y0 + upper_pos.y1) / 2

        # Add label to the figure at the centered position
        ax.figure.text(
            lower_pos.x0 - 0.05,  # x position (left of the axes)
            center_y,  # y position (centered)
            "Average Time (s)",
            fontsize=15,
            rotation=90,
            va='center',
            ha='right'
        )

    # Remove x-axis labels from upper plot
    ax_upper.set_xlabel('')
    ax_upper.set_xticklabels([])
    ax_upper.tick_params(axis='x', length=0)

    # Set title on the upper axis
    if title:
        ax_upper.set_title(title, fontsize=16, pad=10)

    # Hide spines where break occurs
    ax_upper.spines['bottom'].set_visible(False)
    ax_upper.tick_params(axis='x', which='both', bottom=False)
    ax_lower.spines['top'].set_visible(False)

    # Prevent tick label clipping
    ax_upper.tick_params(axis='y', pad=5)
    ax_lower.tick_params(axis='y', pad=5)

    # Add diagonal lines to indicate broken axis
    d = 0.015  # size of diagonal lines
    kwargs = dict(transform=ax_upper.transAxes,
                  color='k', clip_on=False, linewidth=1)
    ax_upper.plot((-d, +d), (-d, +d), **kwargs)  # bottom-left
    ax_upper.plot((1 - d, 1 + d), (-d, +d), **kwargs)  # bottom-right

    kwargs.update(transform=ax_lower.transAxes)
    ax_lower.plot((-d, +d), (1 - d, 1 + d), **kwargs)  # top-left
    ax_lower.plot((1 - d, 1 + d), (1 - d, 1 + d), **kwargs)  # top-right

    return legend_handles_lower, legend_labels_lower


def group_experiments_by_network_and_aggregation(
    dm: DataModel
) -> Dict[Tuple[str, int], Dict[str, List[Experiment]]]:
    """
    Group experiments by network (prefix) and aggregation, then by algorithm.
    Returns: {(network, aggregation): {algorithm: [experiments]}}
    """
    result: Dict[Tuple[str, int], Dict[str, List[Experiment]]
                 ] = defaultdict(lambda: defaultdict(list))

    exp_by_instance = get_experiments_by_instance(dm)

    for instance_idx, exps in exp_by_instance.items():
        instance = dm.instances.get(instance_idx)
        if not instance:
            continue

        network = instance.prefix
        aggregation = int(instance.aggregation)

        for exp in exps:
            result[(network, aggregation)][exp.algorithm].append(exp)

    return result


def create_stacked_bar_chart_on_axis(
    ax,
    algorithms: List[str],
    phase_data: Dict[str, Dict[str, float]],
    show_ylabel: bool = True
):
    """
    Create a stacked bar chart on a given axis.

    Args:
        ax: Matplotlib axis
        algorithms: List of algorithm names in order
        phase_data: {algorithm: {phase_name: avg_time}}
        show_ylabel: Whether to show the y-axis label
    """
    # Collect all unique phases across all algorithms, preserving order
    all_phases = []
    for algo in algorithms:
        if algo in phase_data:
            for phase in phase_data[algo].keys():
                if phase not in all_phases:
                    all_phases.append(phase)

    # Move postprocessing to the end if present
    if "postprocessing" in all_phases:
        all_phases.remove("postprocessing")
        all_phases.append("postprocessing")

    # Prepare data for stacked bars
    x_positions = np.arange(len(algorithms))
    bar_width = 0.6

    # Track bottom position for stacking
    bottoms = np.zeros(len(algorithms))

    # Plot each phase as a stacked segment
    legend_handles = []
    legend_labels = []
    seen_phases = set()

    for phase in all_phases:
        heights = []
        for algo in algorithms:
            heights.append(phase_data.get(algo, {}).get(phase, 0.0))

        color = get_phase_color(phase)
        legend_name = get_phase_legend_name(phase)

        bars = ax.bar(
            x_positions,
            heights,
            bar_width,
            bottom=bottoms,
            color=color,
            label=legend_name,
            alpha=0.85,
            edgecolor='white',
            linewidth=0.5
        )

        # Add to legend only if not seen before and has non-zero data
        if legend_name not in seen_phases and any(h > 0 for h in heights):
            legend_handles.append(bars[0])
            legend_labels.append(legend_name)
            seen_phases.add(legend_name)

        bottoms += heights

    # Style the axis
    if show_ylabel:
        ax.set_ylabel("Average Time (s)", fontsize=15)
    ax.set_xticks(x_positions)
    ax.set_xticklabels([get_display_label(a) for a in algorithms],
                       rotation=45, ha="right", fontsize=13)
    ax.tick_params(axis='y', labelsize=13)
    ax.grid(True, axis="y", linestyle="--", alpha=0.3)

    return legend_handles, legend_labels


def create_network_comparison_plot(
    dm: DataModel,
    network: str,
    exp_by_network_agg: Dict[Tuple[str, int], Dict[str, List[Experiment]]],
    out_dir: str,
    y_axis_breaks: Dict[int, Optional[Tuple[float, float]]] = None
):
    """Create comparison plot for one network across aggregations."""

    if y_axis_breaks is None:
        y_axis_breaks = {}

    # Create figure with 3 subplots (one per aggregation)
    fig, axes = plt.subplots(1, 3, figsize=(
        16, 5), sharey=False)

    # Get network display name
    network_name = NETWORK_NAMES.get(network.lower(), network)
    fig.suptitle(network_name, fontsize=16, fontweight='bold', y=1.02)

    all_legend_handles = []
    all_legend_labels = []

    for idx, aggregation in enumerate(AGGREGATIONS):
        ax = axes[idx]
        key = (network, aggregation)

        if key not in exp_by_network_agg:
            ax.set_title(f"{aggregation}s", fontsize=16,
                         fontweight='bold', pad=20)
            ax.text(0.5, 0.5, "No data", ha='center', va='center',
                    transform=ax.transAxes, fontsize=12)
            continue

        # Get experiments for this network+aggregation
        algo_exps = exp_by_network_agg[key]

        # Calculate average phase times for each algorithm
        phase_data: Dict[str, Dict[str, float]] = {}
        present_algorithms = []

        for algo in ALGORITHM_ORDER:
            if algo in algo_exps:
                experiments = algo_exps[algo]
                avg_times = calculate_average_phase_times(experiments)
                if avg_times:
                    phase_data[algo] = avg_times
                    present_algorithms.append(algo)

        if not present_algorithms:
            ax.set_title(f"{aggregation}s", fontsize=16,
                         fontweight='bold', pad=20)
            ax.text(0.5, 0.5, "No data", ha='center', va='center',
                    transform=ax.transAxes, fontsize=12)
            continue

        # Check if we need to break the y-axis for this aggregation
        y_breaks = y_axis_breaks.get(aggregation)
        show_ylabel = (idx == 0)  # Only show y-label on leftmost subplot

        if y_breaks:
            # Create broken axis for this subplot
            legend_handles, legend_labels = create_broken_axis_subplot(
                ax,
                present_algorithms,
                phase_data,
                y_breaks[0],
                y_breaks[1],
                show_ylabel=show_ylabel,
                title=f"{aggregation}s"
            )
        else:
            # Create regular stacked bar chart
            legend_handles, legend_labels = create_stacked_bar_chart_on_axis(
                ax,
                present_algorithms,
                phase_data,
                show_ylabel=show_ylabel
            )
            # Set subplot title for non-broken axis
            ax.set_title(f"{aggregation}s", fontsize=16, pad=10)

        # Collect legend items from first subplot
        if idx == 0:
            all_legend_handles = legend_handles
            all_legend_labels = legend_labels

    # Add single legend at the top using middle subplot for positioning
    # This ensures the legend takes up proper space in the layout
    if all_legend_handles:
        # Use the middle subplot to anchor the legend
        axes[1].legend(
            all_legend_handles,
            all_legend_labels,
            loc='lower center',
            bbox_to_anchor=(0.5, 1.0),
            bbox_transform=fig.transFigure,
            ncol=4,
            fontsize=14,
            frameon=False,
            fancybox=False,
            shadow=False
        )

    # Adjust layout
    plt.tight_layout(rect=[0, 0, 1, 0.97])

    # Save plot
    output_filename = f"routing_phase_breakdown_{network.lower()}.pdf"
    output_path = os.path.join(out_dir, output_filename)
    fig.savefig(output_path, bbox_inches='tight', dpi=150)
    plt.close(fig)

    print(f"Saved: {output_path}")


def main():
    parser = argparse.ArgumentParser(
        description="Generate routing phase breakdown plots grouped by network",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--log", "-l",
        required=True,
        help="Path to the .out log file"
    )
    parser.add_argument(
        "--csv", "-c",
        required=True,
        help="Path to the .csv input file"
    )
    parser.add_argument(
        "--out-dir", "-o",
        default="plots_out",
        help="Output directory for plots (default: plots_out)"
    )
    parser.add_argument(
        "--y-axis-breaks-60",
        nargs=2,
        type=float,
        metavar=('LOWER_MAX', 'UPPER_MIN'),
        help="Break y-axis for aggregation 60. Example: --y-axis-breaks-60 100 300"
    )
    parser.add_argument(
        "--y-axis-breaks-300",
        nargs=2,
        type=float,
        metavar=('LOWER_MAX', 'UPPER_MIN'),
        help="Break y-axis for aggregation 300. Example: --y-axis-breaks-300 50 200"
    )
    parser.add_argument(
        "--y-axis-breaks-900",
        nargs=2,
        type=float,
        metavar=('LOWER_MAX', 'UPPER_MIN'),
        help="Break y-axis for aggregation 900. Example: --y-axis-breaks-900 25 180"
    )

    args = parser.parse_args()

    # Validate y-axis breaks if provided
    y_axis_breaks = {
        60: tuple(args.y_axis_breaks_60) if args.y_axis_breaks_60 else None,
        300: tuple(args.y_axis_breaks_300) if args.y_axis_breaks_300 else None,
        900: tuple(args.y_axis_breaks_900) if args.y_axis_breaks_900 else None
    }

    for agg, breaks in y_axis_breaks.items():
        if breaks and breaks[0] >= breaks[1]:
            print(
                f"Error: First y-axis break value must be less than second value for aggregation {agg}",
                file=sys.stderr
            )
            sys.exit(1)

    # Ensure output directory exists
    os.makedirs(args.out_dir, exist_ok=True)

    # Build data model
    print("Parsing log and CSV files...")
    dm = build_model(args.log, args.csv)
    print(f"Loaded {len(dm.experiments)} experiments")

    # Group experiments by network and aggregation
    exp_by_network_agg = group_experiments_by_network_and_aggregation(dm)

    if not exp_by_network_agg:
        print("No experiments found!")
        return

    # Get unique networks
    networks = sorted(set(network for (network, _)
                      in exp_by_network_agg.keys()))

    print(f"Found networks: {networks}")
    print(
        f"Found aggregations: {sorted(set(agg for (_, agg) in exp_by_network_agg.keys()))}")

    # Create one plot per network
    for network in networks:
        print(f"\nGenerating plot for network: {network}")
        create_network_comparison_plot(
            dm, network, exp_by_network_agg, args.out_dir, y_axis_breaks)

    print(f"\nAll plots saved to: {args.out_dir}")


if __name__ == "__main__":
    main()
