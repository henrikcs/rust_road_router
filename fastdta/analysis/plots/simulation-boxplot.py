# plots/sim_boxplot.py
from __future__ import annotations
import os
from typing import Dict, List, Tuple
import matplotlib.pyplot as plt

from common import (
    DataModel, experiments_by_line, parse_duration_to_seconds, normalize_algo,
)
from .base import Plot, register_plot
from .styles import style_for_algo  # use the shared color mapping


def _ensure_outdir(out_dir: str):
    os.makedirs(out_dir, exist_ok=True)


@register_plot
class SimulationBoxplotByAlgoAgg(Plot):
    @staticmethod
    def key() -> str:
        return "simulation-boxplot"

    @staticmethod
    def display_name() -> str:
        return "Simulation duration by (algorithm, aggregation) (boxplot)"

    @staticmethod
    def filename_suffix() -> str:
        return "simulation-boxplot"

    def run(self, dm: DataModel, out_dir: str) -> None:
        _ensure_outdir(out_dir)
        exp_by_line = experiments_by_line(dm)

        for line_idx, exps in exp_by_line.items():
            input_row = dm.inputs_by_line.get(line_idx)
            if not input_row:
                continue

            # Collect simulation durations (seconds) per (algorithm, aggregation)
            by_algo_agg: Dict[str, Dict[str, List[float]]] = {}
            for exp in exps:
                algo = normalize_algo(exp.algorithm)
                agg = str(exp.aggregation or "")
                for s in exp.steps:
                    if s.iteration < 1 or s.iteration > input_row.last_iter:
                        continue
                    if not s.simulation or not s.simulation.duration:
                        continue
                    sec = parse_duration_to_seconds(s.simulation.duration)
                    if sec is None:
                        continue
                    by_algo_agg.setdefault(algo, {}).setdefault(
                        agg, []).append(sec)

            if not by_algo_agg:
                continue

            # Build data, labels, positions with larger gaps between algorithms
            def agg_sort_key(a: str):
                try:
                    return (0, int(a))
                except Exception:
                    return (1, a)

            algos = sorted(by_algo_agg.keys())
            gap_within = 0.5   # small gap between aggregations of the same algorithm
            gap_between = 1.8  # larger gap between different algorithms
            pos = 1.0

            data: List[List[float]] = []
            labels: List[str] = []
            positions: List[float] = []
            colors: List[str] = []

            for ai, algo in enumerate(algos):
                aggs = sorted(by_algo_agg[algo].keys(), key=agg_sort_key)
                for j, agg in enumerate(aggs):
                    data.append(by_algo_agg[algo][agg])
                    labels.append(f"{algo} | {agg}")
                    positions.append(pos)
                    colors.append(style_for_algo(algo)["color"])
                    # advance by within-group increment
                    pos += gap_within
                # after each algorithm group, add extra gap (beyond the last within-gap already added)
                pos += (gap_between - gap_within)

            plt.figure(figsize=(max(6, len(labels) * 1.2), 4))
            bp = plt.boxplot(
                data,
                positions=positions,
                widths=0.45,
                patch_artist=True,
            )

            # Color each box by algorithm color
            for patch, c in zip(bp["boxes"], colors):
                patch.set_facecolor(c)
                patch.set_alpha(0.75)
            # Tidy up other elements
            for med in bp["medians"]:
                med.set_color("black")
                med.set_linewidth(1.2)
            for whisk in bp["whiskers"]:
                whisk.set_color("black")
                whisk.set_linewidth(1.0)
            for cap in bp["caps"]:
                cap.set_color("black")
                cap.set_linewidth(1.0)

            plt.title(
                f"Simulation duration by (algorithm, aggregation) (line #{line_idx})")
            plt.xlabel("(algorithm, aggregation)")
            plt.ylabel("Simulation duration (s)")
            plt.grid(True, axis="y", linestyle="--", alpha=0.4)
            plt.xticks(positions, labels, rotation=30, ha="right")

            fname = f"{self.filename_base(input_row)}-{self.filename_suffix()}.pdf"
            plt.tight_layout()
            plt.savefig(os.path.join(out_dir, fname))
            plt.close()
