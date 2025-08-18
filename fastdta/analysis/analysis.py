# cli.py
import argparse
import os
from common import build_model
from plots import discover_plots


def main():
    parser = argparse.ArgumentParser(
        description="Generate DTA experiment plots from JSON/CSV")
    parser.add_argument("--json", required=True,
                        help="Path to test.json (summary)")
    parser.add_argument("--csv", required=True,
                        help="Path to test.csv (inputs)")
    parser.add_argument("--out", default="plots_out",
                        help="Output directory for PNGs")
    parser.add_argument(
        "--plots",
        nargs="*",
        default=None,
        help="Optional list of plot keys to run (e.g., rel-dev-lines router-avg-point router-boxplot)",
    )
    args = parser.parse_args()

    dm = build_model(args.json, args.csv)

    plots = discover_plots()
    if args.plots:
        keys = set(args.plots)
        plots = [p for p in plots if p.key() in keys]

    if not plots:
        print("No plots selected or discovered.")
        return

    print("Running plots:", ", ".join(p.key() for p in plots))
    for p in plots:
        p.run(dm, args.out)

    print(f"Done. Plots saved in: {os.path.abspath(args.out)}")


if __name__ == "__main__":
    main()
