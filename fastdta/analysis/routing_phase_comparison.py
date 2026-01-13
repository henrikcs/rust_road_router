#!/usr/bin/env python3
"""
Generate routing phase breakdown comparison plots grouped by aggregation level.

Creates one plot per aggregation (60, 300, 900) showing stacked bar charts
of routing phases for different algorithms. Each algorithm has its own subplot
with a separate y-axis.

Usage:
    python routing_phase_comparison.py --log <experiment.out> --csv <experiment.csv> [--out-dir <output>]
"""
from plots.styles import get_display_label
from common import (
    build_model,
    DataModel,
    Experiment,
    get_experiments_by_instance,
)
import sys
import os
import argparse
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


def normalize_phase_name(phase_name: str) -> str:
    """
    Normalize phase names, combining specific phases into categories.
    Routing and customization phases are kept separate but will get the same color.

    For example:
    - "get preferred paths" -> "choice model"
    - "first routing", "second routing", "routing sample 1" -> kept as-is (same color via get_phase_color)
    """
    phase_lower = phase_name.lower().strip()

    # Phases that should be grouped into choice model
    choice_model_phases = [
        "choice model",
        "get preferred paths",
        "add fastdta2 alternatives",
        "calculate travel times on original"
    ]

    for cm_phase in choice_model_phases:
        if cm_phase in phase_lower:
            return "choice model"

    # Keep routing and customization phases as-is (they'll get colored the same)
    # Return original (lowercased) for all other phases
    return phase_lower


def get_phase_color(phase_name: str) -> str:
    """
    Get the color for a phase. Routing and customization variants get the same color.
    """
    phase_lower = phase_name.lower()

    # Check if it's a routing phase
    if "routing" in phase_lower:
        return ROUTING_COLOR

    # Check if it's a customization phase
    if "customization" in phase_lower or "customize" in phase_lower:
        return CUSTOMIZATION_COLOR

    # Check if it's an adjust weights phase (from fastdta samples)
    if "adjust weights" in phase_lower:
        return PHASE_COLORS.get("adjust weights", DEFAULT_PHASE_COLOR)

    # Look up in standard colors
    return PHASE_COLORS.get(phase_name, DEFAULT_PHASE_COLOR)


def get_phase_legend_name(phase_name: str) -> str:
    """
    Get the legend name for a phase. Groups routing and customization variants.
    """
    phase_lower = phase_name.lower()

    if "routing" in phase_lower:
        return "routing phases"

    if "customization" in phase_lower or "customize" in phase_lower:
        return "customization phases"

    # Group all "adjust weights (sample X)" under one legend entry
    if "adjust weights" in phase_lower:
        return "adjust weights"
        return "customization phases"

    return phase_name


def get_phase_times_for_experiment(exp: Experiment, skip_first: bool = True) -> Dict[str, List[float]]:
    """
    Extract phase times from an experiment, preserving order of first appearance.
    Returns: {phase_name: [time_per_iteration]}
    """
    phase_order = []  # Track order of first appearance
    phase_times: Dict[str, List[float]] = defaultdict(list)

    for step in exp.steps:
        if skip_first and step.iteration == 0:
            continue

        # Track phases in order of appearance
        for pd in step.phase_details:
            # Skip "fastdta routing" phase specifically
            if pd.phase_name.lower().strip() == "fastdta routing":
                continue

            phase_name = normalize_phase_name(pd.phase_name)
            if phase_name not in phase_order:
                phase_order.append(phase_name)

            # Add time for this phase in this iteration
            if pd.duration_seconds is not None:
                phase_times[phase_name].append(pd.duration_seconds)

    # Move postprocessing to the end if present
    if "postprocessing" in phase_order:
        phase_order.remove("postprocessing")
        phase_order.append("postprocessing")

    # Maintain order
    ordered_result = {phase: phase_times[phase]
                      for phase in phase_order if phase in phase_times}
    return ordered_result


def calculate_average_phase_times(experiments: List[Experiment]) -> Dict[str, float]:
    """
    Calculate average phase times across all experiments and iterations.
    Returns: {phase_name: average_time}
    """
    all_phase_times: Dict[str, List[float]] = defaultdict(list)
    phase_order = []

    for exp in experiments:
        exp_phases = get_phase_times_for_experiment(exp, skip_first=True)
        for phase_name, times in exp_phases.items():
            if phase_name not in phase_order:
                phase_order.append(phase_name)
            all_phase_times[phase_name].extend(times)

    # Move postprocessing to the end if present
    if "postprocessing" in phase_order:
        phase_order.remove("postprocessing")
        phase_order.append("postprocessing")

    # Calculate averages in order
    avg_times = {}
    for phase in phase_order:
        if phase in all_phase_times and all_phase_times[phase]:
            avg_times[phase] = np.mean(all_phase_times[phase])

    return avg_times


def group_experiments_by_algorithm_and_aggregation(
    dm: DataModel
) -> Dict[Tuple[str, int], List[Experiment]]:
    """
    Group experiments by algorithm and aggregation level.
    Returns: {(algorithm, aggregation): [experiments]}
    """
    result: Dict[Tuple[str, int], List[Experiment]] = defaultdict(list)

    for exp in dm.experiments:
        instance = dm.instances.get(exp.instance_index)
        if instance:
            key = (exp.algorithm, int(instance.aggregation))
            result[key].append(exp)

    return result


def create_stacked_bar_chart(
    ax,
    algorithms: List[str],
    phase_data: Dict[str, Dict[str, float]],
    title: str,
    y_limits: Optional[Tuple[float, float]] = None,
    show_ylabel: bool = True
):
    """
    Create stacked bar chart for multiple algorithms.

    Args:
        ax: Matplotlib axis
        algorithms: List of algorithm names in display order
        phase_data: {algorithm: {phase_name: average_time}}
        title: Plot title
        y_limits: Optional tuple (y_min, y_max) for y-axis limits
        show_ylabel: Whether to show the y-axis label
    """
    # Prepare data
    bar_positions = np.arange(len(algorithms))
    bar_width = 0.6

    # Collect all unique phases in order of appearance across all algorithms
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

    # Create stacked bars
    bottoms = np.zeros(len(algorithms))

    # Track legend entries to avoid duplicates
    legend_entries = {}  # legend_name -> handle

    for phase in all_phases:
        heights = []
        for algo in algorithms:
            if algo in phase_data and phase in phase_data[algo]:
                heights.append(phase_data[algo][phase])
            else:
                heights.append(0.0)

        color = get_phase_color(phase)
        legend_name = get_phase_legend_name(phase)

        # Only add to legend if not already present
        if legend_name not in legend_entries:
            bar_container = ax.bar(bar_positions, heights, bar_width, bottom=bottoms,
                                   label=legend_name, color=color, edgecolor='white', linewidth=0.5)
            legend_entries[legend_name] = bar_container
        else:
            # Don't add label (already in legend)
            ax.bar(bar_positions, heights, bar_width, bottom=bottoms,
                   color=color, edgecolor='white', linewidth=0.5)

        bottoms += heights

    # Style
    if show_ylabel:
        ax.set_ylabel("Average Time (s)", fontsize=16)
    # Title removed as requested
    ax.set_xticks(bar_positions)
    ax.set_xticklabels([get_display_label(a)
                       for a in algorithms], rotation=45, ha='right', fontsize=14)
    ax.tick_params(axis='y', labelsize=14)
    ax.grid(True, axis='y', linestyle='--', alpha=0.3)

    # Set y-axis limits if specified
    if y_limits:
        ax.set_ylim(y_limits)

    # Show legend
    if legend_entries:
        ax.legend(loc='upper left', fontsize=12, framealpha=0.9)


def create_stacked_bar_chart_with_broken_axis(
    fig,
    algorithms: List[str],
    phase_data: Dict[str, Dict[str, float]],
    title: str,
    break_lower: float,
    break_upper: float
):
    """
    Create stacked bar chart with broken y-axis.

    Args:
        fig: Matplotlib figure
        algorithms: List of algorithm names in display order
        phase_data: {algorithm: {phase_name: average_time}}
        title: Plot title
        break_lower: Upper limit of lower y-axis segment
        break_upper: Lower limit of upper y-axis segment
    """
    # Calculate max value to determine upper limit
    max_val = 0.0
    for algo_data in phase_data.values():
        total = sum(algo_data.values())
        max_val = max(max_val, total)

    # Add some padding to max value
    upper_limit = max_val * 1.05

    # Create two subplots with shared x-axis
    # Upper subplot (for higher values)
    ax_upper = fig.add_subplot(2, 1, 1)
    # Lower subplot (for lower values)
    ax_lower = fig.add_subplot(2, 1, 2)

    # Adjust spacing between subplots
    fig.subplots_adjust(hspace=0.05)

    # Create the same stacked bars on both axes
    # Only the lower axis should have a y-label
    for idx, (ax, y_limits) in enumerate([(ax_upper, (break_upper, upper_limit)),
                                          (ax_lower, (0, break_lower))]):
        should_show_ylabel = (idx == 1)  # Only show for lower axis
        create_stacked_bar_chart(
            ax, algorithms, phase_data,
            title if ax == ax_upper else "",
            y_limits,
            show_ylabel=should_show_ylabel
        )

        # Remove x-axis labels from upper plot
        if ax == ax_upper:
            ax.set_xlabel('')
            ax.set_xticklabels([])
            ax.tick_params(axis='x', length=0)

        # Hide spines where break occurs
        if ax == ax_upper:
            ax.spines['bottom'].set_visible(False)
            ax.tick_params(axis='x', which='both', bottom=False)
        else:
            ax.spines['top'].set_visible(False)
            legend = ax.legend()
            if legend:
                legend.set_visible(False)  # Only show legend in upper plot

    # Synchronize y-axis tick spacing
    # Get tick locations from lower axis
    lower_ticks = ax_lower.get_yticks()
    if len(lower_ticks) >= 2:
        # Calculate tick spacing from lower axis
        tick_spacing = lower_ticks[1] - lower_ticks[0]

        # Generate ticks for upper axis with same spacing
        # Start from first multiple of tick_spacing >= break_upper
        import math
        first_upper_tick = math.ceil(break_upper / tick_spacing) * tick_spacing
        upper_ticks = np.arange(
            first_upper_tick, upper_limit + tick_spacing/2, tick_spacing)
        ax_upper.set_yticks(upper_ticks)

    # Add diagonal lines to indicate broken axis
    d = 0.015  # size of diagonal lines
    kwargs = dict(transform=ax_upper.transAxes,
                  color='k', clip_on=False, linewidth=1)
    ax_upper.plot((-d, +d), (-d, +d), **kwargs)  # bottom-left
    ax_upper.plot((1 - d, 1 + d), (-d, +d), **kwargs)  # bottom-right

    kwargs.update(transform=ax_lower.transAxes)
    ax_lower.plot((-d, +d), (1 - d, 1 + d), **kwargs)  # top-left
    ax_lower.plot((1 - d, 1 + d), (1 - d, 1 + d), **kwargs)  # top-right


def main():
    parser = argparse.ArgumentParser(
        description="Generate routing phase breakdown comparison by aggregation",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Example:
  %(prog)s --log experiment.out --csv experiment.csv --out-dir phase_plots/ --suffix _seq --y-axis-breaks-60 100 300
        """
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
        default="phase_comparison_plots",
        help="Output directory for plots (default: phase_comparison_plots)"
    )
    parser.add_argument(
        "--suffix",
        default="",
        help="Suffix to append to output filenames (e.g., '_seq' results in routing_phase_comparison_agg60_seq.pdf)"
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
    y_axis_breaks_map = {
        60: args.y_axis_breaks_60,
        300: args.y_axis_breaks_300,
        900: args.y_axis_breaks_900
    }

    for agg, breaks in y_axis_breaks_map.items():
        if breaks and breaks[0] >= breaks[1]:
            print(
                f"Error: First y-axis break value must be less than second value for aggregation {agg}", file=sys.stderr)
            sys.exit(1)

    # Validate input files
    if not os.path.isfile(args.log):
        print(f"Error: Log file not found: {args.log}", file=sys.stderr)
        sys.exit(1)
    if not os.path.isfile(args.csv):
        print(f"Error: CSV file not found: {args.csv}", file=sys.stderr)
        sys.exit(1)

    # Create output directory
    os.makedirs(args.out_dir, exist_ok=True)

    # Parse data
    print("Parsing log and CSV files...")
    dm = build_model(args.log, args.csv)

    # Group experiments
    exp_by_algo_agg = group_experiments_by_algorithm_and_aggregation(dm)

    # Get all aggregations
    aggregations = sorted(set(agg for (_, agg) in exp_by_algo_agg.keys()))

    if not aggregations:
        print("No experiments found in data.", file=sys.stderr)
        sys.exit(1)

    print(f"Found aggregations: {aggregations}")
    print(
        f"Found algorithms: {sorted(set(algo for (algo, _) in exp_by_algo_agg.keys()))}")

    # Create one plot per aggregation
    for aggregation in aggregations:
        # Get y-axis breaks for this aggregation
        y_axis_breaks_current = y_axis_breaks_map.get(aggregation)
        print(f"\nGenerating plot for aggregation {aggregation}...")

        # Collect data for this aggregation
        phase_data: Dict[str, Dict[str, float]] = {}
        present_algorithms = []

        for algo in ALGORITHM_ORDER:
            key = (algo, aggregation)
            if key in exp_by_algo_agg:
                experiments = exp_by_algo_agg[key]
                avg_times = calculate_average_phase_times(experiments)
                if avg_times:
                    phase_data[algo] = avg_times
                    present_algorithms.append(algo)
                    print(
                        f"  {algo}: {len(experiments)} experiments, {len(avg_times)} phases")

        if not present_algorithms:
            print(f"  Skipping aggregation {aggregation} (no data)")
            continue

        # Create figure
        title = f"Routing Phase Breakdown (Aggregation {aggregation}s)"

        if y_axis_breaks_current:
            # Create figure with broken axis
            fig = plt.figure(
                figsize=(max(10, len(present_algorithms) * 1.5), 8))
            create_stacked_bar_chart_with_broken_axis(
                fig,
                present_algorithms,
                phase_data,
                title,
                y_axis_breaks_current[0],
                y_axis_breaks_current[1]
            )
        else:
            # Create regular figure
            fig, ax = plt.subplots(
                figsize=(max(10, len(present_algorithms) * 1.5), 6))
            create_stacked_bar_chart(
                ax,
                present_algorithms,
                phase_data,
                title
            )

        # Save plot with suffix
        output_filename = f"routing_phase_comparison_agg{aggregation}{args.suffix}.pdf"
        output_path = os.path.join(args.out_dir, output_filename)
        fig.tight_layout()
        fig.savefig(output_path, bbox_inches='tight', dpi=150)
        plt.close(fig)

        print(f"  Saved: {output_path}")

    print(f"\nAll plots saved to: {args.out_dir}")


if __name__ == "__main__":
    main()
