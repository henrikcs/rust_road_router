#!/usr/bin/env python3
# analysis.py
"""
Generate DTA experiment plots from .out log files and .csv input files.
This script directly parses log files (no need for sumo-log-parser.rs).

Usage:
    python analysis.py --log <experiment.out> --csv <input.csv> [--out-dir <plots_dir>] [--plots <plot1> <plot2> ...]

Example:
    python analysis.py --log experiment.out --csv experiment.csv --out-dir plots/
    python analysis.py --log experiment.out --csv experiment.csv --plots rel-gap-averaged router-boxplot-by-algo
"""
import argparse
import os
import sys

from common import build_model
from plots import discover_plots


def main():
    parser = argparse.ArgumentParser(
        description="Generate DTA experiment plots from .out log files and .csv input files",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s --log experiment.out --csv experiment.csv
  %(prog)s --log experiment.out --csv experiment.csv --out-dir plots/
  %(prog)s --log experiment.out --csv experiment.csv --plots rel-gap-averaged simulation-boxplot
  %(prog)s --log experiment.out --csv experiment.csv --ignore cch fastdta_1_1

Available plots:
  router-boxplot-by-algo    - Router duration boxplot per algorithm (for each instance)
  router-boxplot-by-rep     - Router duration boxplot per repetition (for each instance+algorithm)
  rel-gap-by-rep            - Relative gap lines per repetition (for each instance+algorithm)
  rel-gap-averaged          - Relative gap averaged over repetitions (for each instance)
  rel-dev-lines             - Relative travel time deviation (for each instance)
  simulation-boxplot        - Simulation duration boxplot per algorithm (for each instance)
  routing-breakdown         - Routing phase breakdown for sampled algorithms
        """
    )
    parser.add_argument(
        "--log", "-l",
        required=False,
        help="Path to the .out log file from run_experiments.sh"
    )
    parser.add_argument(
        "--csv", "-c",
        required=False,
        help="Path to the .csv input file used for experiments"
    )
    parser.add_argument(
        "--out-dir", "-o",
        default="plots_out",
        help="Output directory for generated plots (default: plots_out)"
    )
    parser.add_argument(
        "--plots", "-p",
        nargs="*",
        default=None,
        help="Optional list of plot keys to generate (e.g., rel-gap-averaged router-boxplot-by-algo). "
             "If not specified, all available plots are generated."
    )
    parser.add_argument(
        "--ignore", "-i",
        nargs="*",
        default=None,
        help="List of algorithm names to ignore (exclude from analysis and plots). "
             "Names must match exactly, including sample sizes (e.g., 'fastdta_1_1' 'cch')."
    )
    parser.add_argument(
        "--list-plots",
        action="store_true",
        help="List all available plot types and exit"
    )

    args = parser.parse_args()

    # Discover available plots
    plots = discover_plots()

    # List plots and exit if requested
    if args.list_plots:
        print("Available plots:")
        for p in plots:
            print(f"  {p.key():25} - {p.display_name()}")
        return 0

    # Validate required arguments when not listing plots
    if not args.log or not args.csv:
        parser.error("--log and --csv are required when generating plots")

    # Validate input files
    if not os.path.isfile(args.log):
        print(f"Error: Log file not found: {args.log}", file=sys.stderr)
        return 1
    if not os.path.isfile(args.csv):
        print(f"Error: CSV file not found: {args.csv}", file=sys.stderr)
        return 1

    # Build data model (parse log and CSV)
    print(f"Parsing log file: {args.log}")
    print(f"Parsing CSV file: {args.csv}")

    try:
        dm = build_model(args.log, args.csv)
    except Exception as e:
        print(f"Error parsing files: {e}", file=sys.stderr)
        return 1

    # Filter out ignored algorithms
    if args.ignore:
        ignored_set = set(args.ignore)
        original_count = len(dm.experiments)
        dm.experiments = [
            exp for exp in dm.experiments if exp.algorithm not in ignored_set]
        filtered_count = original_count - len(dm.experiments)
        if filtered_count > 0:
            print(
                f"Filtered out {filtered_count} experiments matching ignored algorithms: {', '.join(args.ignore)}")
        # Update algorithms list
        dm.algorithms = sorted(
            list(set(exp.algorithm for exp in dm.experiments)))

    # Print summary
    print(f"\nFound {len(dm.experiments)} experiments")
    print(f"Found {len(dm.instances)} instances")
    print(f"Algorithms: {', '.join(dm.algorithms)}")

    # Filter plots if specific ones requested
    if args.plots:
        keys = set(args.plots)
        selected_plots = [p for p in plots if p.key() in keys]
        unknown_keys = keys - {p.key() for p in plots}
        if unknown_keys:
            print(
                f"Warning: Unknown plot keys ignored: {', '.join(unknown_keys)}", file=sys.stderr)
    else:
        selected_plots = plots

    if not selected_plots:
        print("No plots to generate.", file=sys.stderr)
        return 1

    # Generate plots
    print(f"\nGenerating {len(selected_plots)} plot types...")
    print(f"Output directory: {os.path.abspath(args.out_dir)}")

    for p in selected_plots:
        print(f"  - {p.key()}: {p.display_name()}")
        try:
            p.run(dm, args.out_dir)
        except Exception as e:
            print(f"    Error: {e}", file=sys.stderr)

    print(f"\nDone. Plots saved in: {os.path.abspath(args.out_dir)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
