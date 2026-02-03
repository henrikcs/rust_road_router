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

# Simulation bar color (RGB: 0.635, 0.133, 0.137)
SIMULATION_COLOR = "#a22223"

# Simulation time values by (network, aggregation)
SIMULATION_TIMES = {
    ("leopoldshafen", 900): 2.08,
    ("leopoldshafen", 300): 2.36,
    ("leopoldshafen", 60): 3.17,
    ("rastatt", 900): 34.29,
    ("rastatt", 300): 37.06,
    ("rastatt", 60): 46.13,
    ("karlsruhe", 60): 14400.0,
}

DEFAULT_PHASE_COLOR = "#999999"

# Aggregation order (left to right in plot)
AGGREGATIONS = [900, 300, 60]

# Network name mapping
NETWORK_NAMES = {
    "leopoldshafen": "Leopoldshafen",
    "rastatt": "Rastatt",
    "karlsruhe": "Karlsruhe",
}

# Phases to ignore (will not be included in plots)
IGNORED_PHASES = {
    "get preferred paths",
    "calculate travel times on original",
    "add fastdta2 alternatives",
    "sample"
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

    # Handle adjust weights phases (including "adjust weights (sample X)")
    if "adjust weights" in phase_lower:
        return "adjust weights"

    # Handle known specific phases
    known_phases = [
        "preprocessing", "postprocessing", "calibration",
        "choice model", "sample",
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


def darken_color(hex_color: str, factor: float = 0.7) -> str:
    """Darken a hex color by a factor (0-1, lower is darker)."""
    hex_color = hex_color.lstrip('#')
    r, g, b = int(hex_color[0:2], 16), int(
        hex_color[2:4], 16), int(hex_color[4:6], 16)
    r, g, b = int(r * factor), int(g * factor), int(b * factor)
    return f'#{r:02x}{g:02x}{b:02x}'


def get_phase_legend_name(phase_name: str) -> str:
    """Get legend label for a phase."""
    normalized = normalize_phase_name(phase_name)

    if normalized == "routing":
        return "Querying"
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


def get_phase_times_for_experiment(exp: Experiment, skip_first: bool = False) -> Dict[str, List[float]]:
    """Get phase times grouped by phase name for an experiment."""
    phase_times: Dict[str, List[float]] = defaultdict(list)

    for step in exp.steps:
        if skip_first and step.iteration == 0:
            continue

        for pd in step.phase_details:
            # Skip ignored phases (normalize for comparison)
            if pd.phase_name.lower() in IGNORED_PHASES:
                continue
            if pd.duration_seconds is not None:
                normalized_name = normalize_phase_name(pd.phase_name)
                phase_times[normalized_name].append(pd.duration_seconds)

    return dict(phase_times)


def calculate_average_phase_times(experiments: List[Experiment]) -> Dict[str, float]:
    """
    Calculate average time for each phase across all experiments.

    For each step, sums all phases with the same normalized name (e.g., 'first routing'
    and 'second routing' both become 'routing'), then averages across all steps.
    This ensures that phases occurring multiple times per step are correctly summed.

    Skips the first iteration (iteration 0) unless the experiment only has a single
    iteration, which is the case for single-iteration experiments like Karlsruhe.
    """
    # Collect per-step sums for each normalized phase
    # Structure: {phase_name: [sum_for_step1, sum_for_step2, ...]}
    phase_sums_per_step: Dict[str, List[float]] = defaultdict(list)

    for exp in experiments:
        # Determine if we should skip the first iteration
        # Skip first iteration unless there's only one step (single-iteration experiment)
        is_single_iteration = len(exp.steps) == 1

        for step in exp.steps:
            # Skip iteration 0 unless it's a single-iteration experiment
            if not is_single_iteration and step.iteration == 0:
                continue

            # Sum phases by normalized name for this step
            step_phase_sums: Dict[str, float] = defaultdict(float)

            for pd in step.phase_details:
                # Skip ignored phases
                if pd.phase_name.lower() in IGNORED_PHASES:
                    continue
                if pd.duration_seconds is not None:
                    normalized_name = normalize_phase_name(pd.phase_name)
                    step_phase_sums[normalized_name] += pd.duration_seconds

            # Add this step's sums to the overall collection
            for phase_name, phase_sum in step_phase_sums.items():
                phase_sums_per_step[phase_name].append(phase_sum)

    # Calculate averages across steps
    avg_times = {}
    for phase_name, step_sums in phase_sums_per_step.items():
        if step_sums:
            avg_times[phase_name] = sum(step_sums) / len(step_sums)

    return avg_times


def create_broken_axis_subplot(
    ax,
    algorithms: List[str],
    phase_data: Dict[str, Dict[str, float]],
    break_lower: float,
    break_upper: float,
    show_ylabel: bool = True,
    title: str = "",
    show_route_time: bool = False,
    simulation_time: Optional[float] = None
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

    # Include simulation time in max calculation if present
    if simulation_time is not None:
        max_val = max(max_val, simulation_time)

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
        show_ylabel=False,
        show_route_time=show_route_time,
        simulation_time=simulation_time
    )

    legend_handles_upper, legend_labels_upper = create_stacked_bar_chart_on_axis(
        ax_upper,
        algorithms,
        phase_data,
        show_ylabel=False,
        show_route_time=show_route_time,
        simulation_time=simulation_time
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
        ax_upper.set_title(title, fontsize=16)

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
    show_ylabel: bool = True,
    show_route_time: bool = False,
    simulation_time: Optional[float] = None
):
    """
    Create a stacked bar chart on a given axis.

    Args:
        ax: Matplotlib axis
        algorithms: List of algorithm names in order
        phase_data: {algorithm: {phase_name: avg_time}}
        show_ylabel: Whether to show the y-axis label
        simulation_time: Optional simulation time to add as rightmost bar
    """
    # Collect all unique phases across all algorithms, preserving order
    all_phases = []
    for algo in algorithms:
        if algo in phase_data:
            for phase in phase_data[algo].keys():
                if phase not in all_phases:
                    all_phases.append(phase)

    # Move preprocessing and postprocessing to the end if present
    # preprocessing should be second to last, postprocessing should be last
    if "preprocessing" in all_phases:
        all_phases.remove("preprocessing")
    if "postprocessing" in all_phases:
        all_phases.remove("postprocessing")

    # Add them back in the correct order
    if "preprocessing" in [phase for algo in algorithms if algo in phase_data for phase in phase_data[algo].keys()]:
        all_phases.append("preprocessing")
    if "postprocessing" in [phase for algo in algorithms if algo in phase_data for phase in phase_data[algo].keys()]:
        all_phases.append("postprocessing")

    # Prepare data for stacked bars
    # Add extra position for simulation bar if needed
    num_bars = len(algorithms) + (1 if simulation_time is not None else 0)
    x_positions = np.arange(len(algorithms))
    x_labels = [get_display_label(a) for a in algorithms]

    if simulation_time is not None:
        x_labels.append("Simulation")

    bar_width = 0.6

    # Track bottom position for stacking
    bottoms = np.zeros(len(algorithms))

    # Track positions of routing phase top for annotations
    routing_top_positions = np.zeros(len(algorithms))

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

        # Track top of routing phase for annotations
        if normalize_phase_name(phase) == "routing":
            routing_top_positions = bottoms.copy()

    # Add time annotations for customization and routing phases if requested
    if show_route_time:
        for i, algo in enumerate(algorithms):
            algo_data = phase_data.get(algo, {})
            customization_time = algo_data.get("customization", 0.0)
            routing_time = algo_data.get("routing", 0.0)

            if routing_time > 0:
                # Place annotations right after the routing bar
                y_offset = routing_top_positions[i]
                annotations = []

                # If algorithm has customization, show both labels
                if customization_time > 0:
                    # Customization label first (lower)
                    cust_color = darken_color(CUSTOMIZATION_COLOR, 0.6)
                    annotations.append(
                        (f"{customization_time:.2f}s", cust_color))

                # Routing/Querying label (upper or only)
                route_color = darken_color(ROUTING_COLOR, 0.6)
                annotations.append((f"{routing_time:.2f}s", route_color))

                # Place annotations right after routing bar
                line_height = 0.07 * ax.get_ylim()[1]  # Dynamic line spacing
                for j, (text, color) in enumerate(annotations):
                    ax.text(
                        x_positions[i],
                        y_offset + j * line_height,
                        text,
                        ha='center',
                        va='bottom',
                        fontsize=9,
                        color=color,
                        fontweight='bold'
                    )

    # Add simulation bar if requested
    if simulation_time is not None:
        sim_x = len(algorithms)  # Position after all algorithm bars
        sim_bar = ax.bar(
            sim_x,
            simulation_time,
            bar_width,
            color=SIMULATION_COLOR,
            label="Simulation",
            alpha=0.85,
            edgecolor='white',
            linewidth=0.5
        )
        # Add to legend
        if "Simulation" not in seen_phases:
            legend_handles.append(sim_bar[0])
            legend_labels.append("Simulation")
            seen_phases.add("Simulation")

    # Style the axis
    if show_ylabel:
        ax.set_ylabel("Average Time (s)", fontsize=15)
    all_x_positions = np.arange(num_bars)
    ax.set_xticks(all_x_positions)
    ax.set_xticklabels(x_labels,
                       rotation=45, ha="right", fontsize=13)
    ax.tick_params(axis='y', labelsize=13)
    ax.grid(True, axis="y", linestyle="--", alpha=0.3)

    return legend_handles, legend_labels


def create_network_comparison_plot(
    dm: DataModel,
    network: str,
    exp_by_network_agg: Dict[Tuple[str, int], Dict[str, List[Experiment]]],
    out_dir: str,
    y_axis_breaks: Dict[int, Optional[Tuple[float, float]]] = None,
    no_io: bool = False,
    algorithms: Optional[List[str]] = None,
    show_route_time: bool = False,
    png: bool = False,
    simulation: bool = False
):
    """Create comparison plot for one network across aggregations."""

    if y_axis_breaks is None:
        y_axis_breaks = {}

    # When simulation is set, only show 60s aggregation
    if simulation:
        aggregations_to_plot = [60]
        fig, ax_single = plt.subplots(1, 1, figsize=(4.5, 4.5), sharey=False)
        axes = [ax_single]
    # For Karlsruhe, only show 60s aggregation
    elif network.lower() == 'karlsruhe':
        aggregations_to_plot = [60]
        fig, ax_single = plt.subplots(1, 1, figsize=(4.5, 4.5), sharey=False)
        axes = [ax_single]  # Wrap in list for consistent indexing
    else:
        aggregations_to_plot = AGGREGATIONS
        # Create figure with 3 subplots (one per aggregation)
        fig, axes = plt.subplots(1, 3, figsize=(11, 4.5), sharey=False)

    # Overall title (skip if simulation mode)
    network_name = NETWORK_NAMES.get(network.lower(), network)
    if not simulation:
        fig.suptitle(network_name, fontsize=16, fontweight='bold')

    all_legend_handles = []
    all_legend_labels = []

    for idx, aggregation in enumerate(aggregations_to_plot):
        ax = axes[idx] if len(axes) > 1 else axes[0]
        key = (network, aggregation)

        if key not in exp_by_network_agg:
            ax.set_title(f"{aggregation}s", fontsize=16)
            ax.text(0.5, 0.5, "No data", ha='center', va='center',
                    transform=ax.transAxes, fontsize=12)
            continue

        # Get experiments for this network+aggregation
        algo_exps = exp_by_network_agg[key]

        # Calculate average phase times for each algorithm
        phase_data: Dict[str, Dict[str, float]] = {}
        present_algorithms = []

        # Filter algorithm order based on provided list
        algorithms_to_include = ALGORITHM_ORDER if algorithms is None else [
            algo for algo in ALGORITHM_ORDER if algo in algorithms
        ]

        for algo in algorithms_to_include:
            if algo in algo_exps:
                experiments = algo_exps[algo]
                avg_times = calculate_average_phase_times(experiments)
                # Filter out IO phases if --no-io flag is set
                if no_io:
                    avg_times = {
                        phase: time for phase, time in avg_times.items()
                        if phase.lower() not in ['preprocessing', 'postprocessing']
                    }
                if avg_times:
                    phase_data[algo] = avg_times
                    present_algorithms.append(algo)

        if not present_algorithms:
            ax.set_title(f"{aggregation}s", fontsize=16)
            ax.text(0.5, 0.5, "No data", ha='center', va='center',
                    transform=ax.transAxes, fontsize=12)
            continue

        # Print phase breakdown to console
        print(f"\n{network_name} - Aggregation {aggregation}s:")
        print("=" * 60)
        for algo in present_algorithms:
            total_time = sum(phase_data[algo].values())
            print(f"\n{get_display_label(algo)} (Total: {total_time:.2f}s):")
            for phase_name, avg_time in sorted(phase_data[algo].items(), key=lambda x: -x[1]):
                percentage = (avg_time / total_time *
                              100) if total_time > 0 else 0
                print(
                    f"  {get_phase_legend_name(phase_name):20s}: {avg_time:8.3f}s ({percentage:5.1f}%)")

        # Check if we need to break the y-axis for this aggregation
        y_breaks = y_axis_breaks.get(aggregation)
        show_ylabel = (idx == 0)  # Only show y-label on leftmost subplot

        # Get simulation time for this network+aggregation if requested
        sim_time = None
        if simulation:
            sim_time = SIMULATION_TIMES.get((network.lower(), aggregation))

        if y_breaks:
            # Create broken axis for this subplot
            legend_handles, legend_labels = create_broken_axis_subplot(
                ax,
                present_algorithms,
                phase_data,
                y_breaks[0],
                y_breaks[1],
                show_ylabel=show_ylabel,
                title="" if simulation else f"{aggregation}s",
                show_route_time=show_route_time,
                simulation_time=sim_time
            )
        else:
            # Create regular stacked bar chart
            legend_handles, legend_labels = create_stacked_bar_chart_on_axis(
                ax,
                present_algorithms,
                phase_data,
                show_ylabel=show_ylabel,
                show_route_time=show_route_time,
                simulation_time=sim_time
            )
            # Set subplot title for non-broken axis (skip if simulation mode)
            if not simulation:
                ax.set_title(f"{aggregation}s", fontsize=16)

        # Collect legend items from first subplot
        if idx == 0:
            all_legend_handles = legend_handles
            all_legend_labels = legend_labels

        # X-label for middle subplot (or single subplot for Karlsruhe) - skip if simulation mode
        if not simulation and (idx == 1 or (network.lower() == 'karlsruhe' and idx == 0)):
            ax.set_xlabel("Algorithm", fontsize=15)

    # Create single legend below all subplots (skip if simulation mode)
    if all_legend_handles and not simulation:
        fig.legend(
            all_legend_handles,
            all_legend_labels,
            loc='lower center',
            bbox_to_anchor=(0.5, -0.1),
            ncol=3,
            fontsize=14,
            frameon=False
        )

    # Adjust layout
    plt.tight_layout(rect=[0, 0.05, 1, 0.95])

    # Save plot
    ext = "png" if png else "pdf"
    output_filename = f"routing_phase_breakdown_{network.lower()}.{ext}"
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
    parser.add_argument(
        "--no-io",
        action="store_true",
        help="Exclude preprocessing and postprocessing times from plots"
    )
    parser.add_argument(
        "--algorithms",
        nargs="+",
        help="List of algorithms to include in plots (e.g., --algorithms cch dijkstra-rust)"
    )
    parser.add_argument(
        "--show-route-time",
        action="store_true",
        help="Annotate bars with customization and querying times (both for algorithms with customization, querying only for others)"
    )
    parser.add_argument(
        "--png",
        action="store_true",
        help="Output PNG instead of PDF"
    )
    parser.add_argument(
        "--simulation",
        action="store_true",
        help="Add simulation time bar as rightmost bar"
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
            dm, network, exp_by_network_agg, args.out_dir, y_axis_breaks,
            no_io=args.no_io, algorithms=args.algorithms,
            show_route_time=args.show_route_time, png=args.png,
            simulation=args.simulation)

    print(f"\nAll plots saved to: {args.out_dir}")


if __name__ == "__main__":
    main()
