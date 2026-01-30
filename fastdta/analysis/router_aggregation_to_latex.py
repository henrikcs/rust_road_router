#!/usr/bin/env python3
"""
Generate LaTeX tables showing median router duration by algorithm, aggregation, and parallelization.

Creates two tables:
- SUMO algorithms (dijkstra-rust, cch, dijkstra, astar, ch)
- FastDTA algorithms (fastdta2, fastdta_1_1, fastdta_1_1_1, fastdta_1_2_3_4, cch, dijkstra-rust)

Rows: Networks and aggregations (e.g., Leopoldshafen 900s, Leopoldshafen 300s, ...)
Columns: Algorithms with sequential/parallel subcolumns
Cells: Median router duration (s), with bold for fastest per row in seq/par
"""

from typing import Dict, List, Tuple, Optional
import numpy as np
import argparse
import os
import sys
from common import (
    build_model,
    DataModel,
    get_experiments_by_instance,
    get_routing_times,
)
from plots.styles import get_display_label


# Add analysis directory to path for imports
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))


# Algorithm order for SUMO plot (traditional routing algorithms)
SUMO_ALGORITHM_ORDER = [
    "dijkstra-rust",
    "cch",
    "dijkstra",
    "astar",
    "ch",
]

# Algorithm order for FastDTA plot (FastDTA variants + baselines)
FASTDTA_ALGORITHM_ORDER = [
    "fastdta2",
    "fastdta_1_1",
    "fastdta_1_1_1",
    "fastdta_1_2_3_4",
    "dijkstra-rust",
    "cch",
]

# Aggregation values to expect (in order for display)
AGGREGATIONS = [900, 300, 60]

# Network names for row labels (short names to avoid clipping)
NETWORK_NAMES = {
    "leopoldshafen": "Leopoldshafen",
    "rastatt": "Rastatt",
    "karlsruhe": "Karlsruhe",
}


def is_fastdta_algorithm(algorithm: str) -> bool:
    """Check if algorithm is a FastDTA variant."""
    return algorithm.startswith("fastdta")


def parse_data_for_subplot(dm: DataModel, subtract_pre_post: bool = True) -> Dict[Tuple[str, int], List[float]]:
    """
    Parse routing times from data model, grouped by algorithm and aggregation.

    For FastDTA algorithms (fastdta*): sums individual phase times (excluding pre/post)
    to ensure consistency with breakdown plots, since not all phases may be logged.

    For other algorithms: uses router duration, optionally subtracting pre/post times.

    Args:
        dm: DataModel to parse
        subtract_pre_post: If True, subtract preprocessing and postprocessing times
                          (only applies to non-FastDTA algorithms)

    Returns: {(algorithm, aggregation): [routing_times]}
    """
    result: Dict[Tuple[str, int], List[float]] = {}

    exp_by_instance = get_experiments_by_instance(dm)

    for instance_idx, exps in exp_by_instance.items():
        instance = dm.instances.get(instance_idx)
        if not instance:
            continue

        aggregation = int(instance.aggregation)
        # For single-iteration experiments (last_iter=1), include step 0
        skip_first = instance.last_iter > 1

        for exp in exps:
            adjusted_times = []

            # For FastDTA algorithms, sum phases directly (excluding pre/post)
            # This ensures consistency with breakdown plots
            if is_fastdta_algorithm(exp.algorithm):
                for step in exp.steps:
                    if skip_first and step.iteration == 0:
                        continue

                    # Sum all phases except preprocessing and postprocessing
                    phase_sum = 0.0
                    for phase in step.phase_details:
                        phase_name_lower = phase.phase_name.lower()
                        if "preprocessing" not in phase_name_lower and "postprocessing" not in phase_name_lower and "sample" != phase_name_lower:
                            phase_sum += phase.duration_seconds

                    adjusted_times.append(phase_sum)
            else:
                # For non-FastDTA algorithms, use router duration
                times = get_routing_times(exp, skip_first=skip_first)
                step_idx = 1 if skip_first else 0

                for time in times:
                    if subtract_pre_post and step_idx < len(exp.steps):
                        step = exp.steps[step_idx]

                        # Sum up preprocessing and postprocessing times for this step
                        pre_post_time = 0.0
                        for phase in step.phase_details:
                            phase_name_lower = phase.phase_name.lower()
                            if "preprocessing" in phase_name_lower or "postprocessing" in phase_name_lower:
                                pre_post_time += phase.duration_seconds

                        # Subtract from total time (ensure non-negative)
                        adjusted_time = max(0.0, time - pre_post_time)
                        adjusted_times.append(adjusted_time)
                    else:
                        # Don't subtract, use original time
                        adjusted_times.append(time)

                    step_idx += 1

            if adjusted_times:
                key = (exp.algorithm, aggregation)
                result.setdefault(key, []).extend(adjusted_times)

    return result


def compute_means(data: Dict[Tuple[str, int], List[float]]) -> Dict[Tuple[str, int], float]:
    """
    Compute median for each (algorithm, aggregation) pair.
    Returns: {(algorithm, aggregation): median_time}
    """
    medians = {}
    for key, times in data.items():
        if times:
            medians[key] = np.mean(times)
    return medians


def format_value(value: Optional[float], bold: bool = False) -> str:
    """
    Format a value for LaTeX table.
    Returns formatted string with optional bold.
    """
    if value is None:
        return "---"

    formatted = f"{value:.2f}"
    if bold:
        formatted = f"\\textbf{{{formatted}}}"
    return formatted


def generate_latex_table(
    networks: List[str],
    algorithm_order: List[str],
    data: Dict[Tuple[str, str, str], Dict[Tuple[str, int], float]],
    hide_ch: bool = False
) -> str:
    """
    Generate LaTeX table code (transposed format).

    Args:
        networks: List of network names (e.g., ['leopoldshafen', 'rastatt'])
        algorithm_order: List of algorithms to include
        data: {(network, mode, algorithm): {(algorithm, aggregation): median}}
        hide_ch: Whether to hide CH algorithm

    Returns:
        LaTeX table code as string
    """
    # Filter algorithms
    algos = [a for a in algorithm_order if not (hide_ch and a.lower() == 'ch')]

    # Determine which aggregations to show for each network
    # Karlsruhe only has 60s data, others have all three
    network_aggregations = {}
    for network in networks:
        if network == "karlsruhe":
            network_aggregations[network] = [60]  # Only 60s for Karlsruhe
        else:
            network_aggregations[network] = AGGREGATIONS  # All aggregations

    # Build column specification
    # First column: Algorithm (multirow), Second column: Seq/Par
    # Then for each network: columns for its aggregations
    col_parts = ["l", "l"]
    for network in networks:
        col_parts.extend(["r"] * len(network_aggregations[network]))
    col_spec = "|".join(col_parts)

    latex_lines = []
    latex_lines.append("\\small")  # Reduce font size by one
    latex_lines.append("\\begin{tabular}{" + col_spec + "}")
    latex_lines.append("\\toprule")

    # Header row 1: Network names (spanning columns for their aggregations)
    header1 = " & "  # Empty first two columns
    for network in networks:
        network_display = NETWORK_NAMES.get(network, network)
        num_aggs = len(network_aggregations[network])
        header1 += f" & \\multicolumn{{{num_aggs}}}{{c|}}{{{network_display}}}"
    latex_lines.append(header1 + " \\\\")

    # Header row 2: Aggregation values
    header2 = "Algorithm & Mode"
    for network in networks:
        for agg in network_aggregations[network]:
            header2 += f" & {agg}"
    latex_lines.append(header2 + " \\\\")
    latex_lines.append("\\midrule")

    # Data rows - for each algorithm: 2 rows (Sequential and Parallel)
    first_algo = True
    for algo in algos:
        if not first_algo:
            latex_lines.append("\\midrule")
        first_algo = False

        algo_display = get_display_label(algo)

        # For each mode (Sequential, Parallel)
        for mode_idx, mode in enumerate(["Sequential", "Parallel"]):
            mode_label = "Seq" if mode == "Sequential" else "Par"

            # Collect all values for this row
            row_values = {}  # {(network, agg): value}
            for network in networks:
                for agg in network_aggregations[network]:
                    key = (network, mode, algo)
                    if key in data and (algo, agg) in data[key]:
                        row_values[(network, agg)] = data[key][(algo, agg)]

            # Find minimum for each network+aggregation column
            col_minimums = {}  # {(network, agg): min_value}
            for network in networks:
                for agg in network_aggregations[network]:
                    # Find minimum across all algorithms for this network+agg+mode
                    col_values = []
                    for a in algos:
                        key = (network, mode, a)
                        if key in data and (a, agg) in data[key]:
                            col_values.append(data[key][(a, agg)])
                    if col_values:
                        col_minimums[(network, agg)] = min(col_values)

            # Build row
            if mode_idx == 0:
                # First mode for this algorithm: show algorithm name spanning 2 rows
                row = f"\\multirow{{2}}{{*}}{{{algo_display}}} & {mode_label}"
            else:
                # Second mode: empty algorithm column
                row = f" & {mode_label}"

            # Add data cells for each network and its aggregations
            for network in networks:
                for agg in network_aggregations[network]:
                    val = row_values.get((network, agg))
                    min_val = col_minimums.get((network, agg))
                    bold = (
                        val is not None and min_val is not None and val == min_val)
                    row += f" & {format_value(val, bold)}"

            latex_lines.append(row + " \\\\")

    latex_lines.append("\\bottomrule")
    latex_lines.append("\\end{tabular}")

    return "\n".join(latex_lines)


def main():
    parser = argparse.ArgumentParser(
        description='Generate LaTeX tables with router duration data',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Example:
  %(prog)s \\
    --lh-seq lh_seq.out lh_seq.csv \\
    --lh-par lh_par.out lh_par.csv \\
    --ra-seq ra_seq.out ra_seq.csv \\
    --ra-par ra_par.out ra_par.csv \\
    --out tables.tex
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
                        help='Output file path (e.g., tables.tex)')

    parser.add_argument('--hide-ch', action='store_true',
                        help='Hide CH algorithm from tables')

    args = parser.parse_args()

    # Validate that we have at least Leopoldshafen and Rastatt
    if not (args.lh_seq and args.lh_par and args.ra_seq and args.ra_par):
        parser.error(
            "Leopoldshafen and Rastatt data (both seq and par) are required")

    # Determine which networks we have
    has_karlsruhe = bool(args.ka_seq and args.ka_par)

    networks = ["leopoldshafen", "rastatt"]
    if has_karlsruhe:
        networks.append("karlsruhe")

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

    # Parse all data models and compute medians
    # Parse twice: once for SUMO (keeping pre/post times), once for FastDTA (subtracting them)
    print("Loading data...")
    # {(network, mode, algorithm): {(algorithm, aggregation): median}}
    sumo_medians = {}
    fastdta_medians = {}

    for network, mode, (log_file, csv_file) in configs:
        print(f"  Parsing {network} {mode}...")
        try:
            dm = build_model(log_file, csv_file)

            # Parse for SUMO table: keep preprocessing/postprocessing times
            parsed_data_sumo = parse_data_for_subplot(
                dm, subtract_pre_post=False)
            medians_sumo = compute_means(parsed_data_sumo)

            # Parse for FastDTA table: subtract preprocessing/postprocessing times
            parsed_data_fastdta = parse_data_for_subplot(
                dm, subtract_pre_post=True)
            medians_fastdta = compute_means(parsed_data_fastdta)

            # Store SUMO medians indexed by (network, mode, algorithm)
            for (algo, agg), median_val in medians_sumo.items():
                key = (network, mode, algo)
                if key not in sumo_medians:
                    sumo_medians[key] = {}
                sumo_medians[key][(algo, agg)] = median_val

            # Store FastDTA medians indexed by (network, mode, algorithm)
            for (algo, agg), median_val in medians_fastdta.items():
                key = (network, mode, algo)
                if key not in fastdta_medians:
                    fastdta_medians[key] = {}
                fastdta_medians[key][(algo, agg)] = median_val

        except Exception as e:
            print(f"  Error parsing {network} {mode}: {e}")
            continue

    # Generate LaTeX tables
    print("\n=== Generating SUMO algorithms table ===")
    sumo_table = generate_latex_table(
        networks,
        SUMO_ALGORITHM_ORDER,
        sumo_medians,
        hide_ch=args.hide_ch
    )

    print("\n=== Generating FastDTA algorithms table ===")
    fastdta_table = generate_latex_table(
        networks,
        FASTDTA_ALGORITHM_ORDER,
        fastdta_medians,
        hide_ch=args.hide_ch
    )

    # Write to output file
    with open(args.out, 'w') as f:
        f.write("% SUMO Algorithms Table\n")
        f.write(
            "% Add \\usepackage{booktabs}, \\usepackage{graphicx}, and \\usepackage{multirow} to your LaTeX preamble\n")
        f.write("% Usage: \\input{" + args.out + "}\n\n")
        f.write(sumo_table)
        f.write("\n\n")
        f.write("% FastDTA Algorithms Table\n\n")
        f.write(fastdta_table)
        f.write("\n")

    print(f"\nLaTeX tables written to: {args.out}")
    print("\nTo use in your LaTeX document:")
    print(
        "1. Add \\usepackage{booktabs}, \\usepackage{graphicx}, and \\usepackage{multirow} to your preamble")
    print(f"2. Use \\input{{{args.out}}} to include the tables")
    print("\nDone!")


if __name__ == "__main__":
    main()
