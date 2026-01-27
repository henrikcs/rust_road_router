#!/usr/bin/env python3
"""
Generate LaTeX table showing average simulation time by network and aggregation.

Creates a table with:
- Columns: Networks spanning their aggregation values (e.g., Leopoldshafen 900s, 300s, 60s)
- Row: Average simulation time in seconds
"""

from common import (
    build_model,
    DataModel,
    get_experiments_by_instance,
)
import argparse
import os
import sys
import numpy as np
from typing import Dict, Tuple, List, Optional

# Add analysis directory to path for imports
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))


# Aggregation values in order for display
AGGREGATIONS = [900, 300, 60]

# Network names for column labels
NETWORK_NAMES = {
    "leopoldshafen": "Leopoldshafen",
    "rastatt": "Rastatt",
    "karlsruhe": "Karlsruhe",
}


def parse_simulation_times(dm: DataModel) -> Dict[Tuple[str, int], List[float]]:
    """
    Parse simulation times from data model, grouped by network and aggregation.

    Args:
        dm: DataModel to parse

    Returns: {(network, aggregation): [simulation_times]}
    """
    result: Dict[Tuple[str, int], List[float]] = {}

    exp_by_instance = get_experiments_by_instance(dm)

    for instance_idx, exps in exp_by_instance.items():
        instance = dm.instances.get(instance_idx)
        if not instance:
            continue

        network = instance.prefix
        aggregation = int(instance.aggregation)

        for exp in exps:
            # Collect simulation times from all steps
            for step in exp.steps:
                if step.simulation and step.simulation.duration_seconds is not None:
                    sim_time = step.simulation.duration_seconds
                    key = (network, aggregation)
                    result.setdefault(key, []).append(sim_time)

    return result


def compute_averages(data: Dict[Tuple[str, int], List[float]]) -> Dict[Tuple[str, int], float]:
    """
    Compute average for each (network, aggregation) pair.

    Returns: {(network, aggregation): average_time}
    """
    averages = {}
    for key, times in data.items():
        if times:
            averages[key] = np.mean(times)
    return averages


def format_value(value: Optional[float]) -> str:
    """
    Format a value for LaTeX table.
    """
    if value is None:
        return "---"
    return f"{value:.2f}"


def generate_latex_table(
    networks: List[str],
    data: Dict[str, Dict[Tuple[str, int], float]],
    karlsruhe_value: Optional[float] = None
) -> str:
    """
    Generate LaTeX table code.

    Args:
        networks: List of network names (e.g., ['leopoldshafen', 'rastatt', 'karlsruhe'])
        data: {mode: {(network, aggregation): average}}
        karlsruhe_value: Optional manual value for Karlsruhe (60s)

    Returns:
        LaTeX table code as string
    """
    # Determine which aggregations to show for each network
    network_aggregations = {}
    for network in networks:
        if network == "karlsruhe":
            network_aggregations[network] = [60]  # Only 60s for Karlsruhe
        else:
            network_aggregations[network] = AGGREGATIONS  # All aggregations

    # Build column specification
    # One column for each network's aggregation values
    col_parts = []
    for network in networks:
        col_parts.extend(["r"] * len(network_aggregations[network]))
    col_spec = "|".join(col_parts)

    latex_lines = []
    latex_lines.append("\\small")
    latex_lines.append("\\begin{tabular}{" + col_spec + "}")
    latex_lines.append("\\toprule")

    # Header row 1: Network names (spanning columns for their aggregations)
    header1_parts = []
    for network in networks:
        network_display = NETWORK_NAMES.get(network, network)
        num_aggs = len(network_aggregations[network])
        header1_parts.append(
            f"\\multicolumn{{{num_aggs}}}{{c|}}{{{network_display}}}")
    header1 = " & ".join(header1_parts)
    latex_lines.append(header1 + " \\\\")

    # Header row 2: Aggregation values
    header2_parts = []
    for network in networks:
        for agg in network_aggregations[network]:
            header2_parts.append(f"{agg}s")
    header2 = " & ".join(header2_parts)
    latex_lines.append(header2 + " \\\\")
    latex_lines.append("\\midrule")

    # Data row: average simulation times
    # Combine data from both sequential and parallel (they should have same simulation times)
    combined_data = {}
    for mode_data in data.values():
        for key, value in mode_data.items():
            if key not in combined_data:
                combined_data[key] = []
            combined_data[key].append(value)

    # Average across modes (if we have both seq and par)
    averaged_data = {}
    for key, values in combined_data.items():
        averaged_data[key] = np.mean(values)

    row_parts = []
    for network in networks:
        for agg in network_aggregations[network]:
            # Special handling for Karlsruhe if manual value provided
            if network == "karlsruhe" and karlsruhe_value is not None:
                row_parts.append(format_value(karlsruhe_value))
            else:
                value = averaged_data.get((network, agg))
                row_parts.append(format_value(value))

    row = " & ".join(row_parts)
    latex_lines.append(row + " \\\\")

    latex_lines.append("\\bottomrule")
    latex_lines.append("\\end{tabular}")

    return "\n".join(latex_lines)


def main():
    parser = argparse.ArgumentParser(
        description='Generate LaTeX table with average simulation times',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Example:
  %(prog)s \\
    --lh-seq lh_seq.out lh_seq.csv \\
    --lh-par lh_par.out lh_par.csv \\
    --ra-seq ra_seq.out ra_seq.csv \\
    --ra-par ra_par.out ra_par.csv \\
    --ka-value 1234.56 \\
    --out simulation_times.tex
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

    # Karlsruhe manual value
    parser.add_argument('--ka-value', type=float,
                        help='Manual Karlsruhe average simulation time (60s aggregation)')

    parser.add_argument('--out', '-o', required=True,
                        help='Output file path (e.g., simulation_times.tex)')

    args = parser.parse_args()

    # Validate that we have at least Leopoldshafen and Rastatt
    if not (args.lh_seq and args.lh_par and args.ra_seq and args.ra_par):
        parser.error(
            "Leopoldshafen and Rastatt data (both seq and par) are required")

    # Determine which networks we have
    has_karlsruhe = args.ka_value is not None

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

    # Parse all data models and compute averages
    print("Loading data...")
    # {mode: {(network, aggregation): average}}
    all_averages = {}

    for network, mode, (log_file, csv_file) in configs:
        print(f"  Parsing {network} {mode}...")
        try:
            dm = build_model(log_file, csv_file)
            parsed_data = parse_simulation_times(dm)
            averages = compute_averages(parsed_data)

            if mode not in all_averages:
                all_averages[mode] = {}
            all_averages[mode].update(averages)

        except Exception as e:
            print(f"  Error parsing {network} {mode}: {e}")
            continue

    # Print average times by mode
    print("\n=== Average Simulation Times ===")
    for mode in ["Sequential", "Parallel"]:
        if mode in all_averages:
            print(f"\n{mode}:")
            for network in networks:
                if network == "karlsruhe":
                    aggs = [60]
                else:
                    aggs = AGGREGATIONS
                for agg in aggs:
                    key = (network, agg)
                    if key in all_averages[mode]:
                        avg_time = all_averages[mode][key]
                        print(f"  {network:15s} {agg:4d}s: {avg_time:8.2f}s")

    # Generate LaTeX table
    print("\n=== Generating simulation time table ===")
    table = generate_latex_table(
        networks,
        all_averages,
        karlsruhe_value=args.ka_value
    )

    # Write to output file
    with open(args.out, 'w') as f:
        f.write("% Average Simulation Time Table\n")
        f.write("% Add \\usepackage{booktabs} to your LaTeX preamble\n")
        f.write("% Usage: \\input{" + args.out + "}\n\n")
        f.write(table)
        f.write("\n")

    print(f"\nLaTeX table written to: {args.out}")
    print("\nTo use in your LaTeX document:")
    print("1. Add \\usepackage{booktabs} to your preamble")
    print(f"2. Use \\input{{{args.out}}} to include the table")
    print("\nDone!")


if __name__ == "__main__":
    main()
