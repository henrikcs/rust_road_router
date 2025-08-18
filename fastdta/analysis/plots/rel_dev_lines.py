# plots/rel_dev_lines.py
from __future__ import annotations
import os
from typing import Dict, List, Tuple
import matplotlib.pyplot as plt

from common import (
    DataModel, experiments_by_line, to_float_or_none, normalize_algo,
)
from .base import Plot, register_plot


def _ensure_outdir(out_dir: str):
    os.makedirs(out_dir, exist_ok=True)


@register_plot
class RelDevLines(Plot):
    @staticmethod
    def key() -> str:
        return "rel-dev-lines"

    @staticmethod
    def display_name() -> str:
        return "Relative travel time deviation (lines)"

    @staticmethod
    def filename_suffix() -> str:
        return "rel-dev-lines"

    def run(self, dm: DataModel, out_dir: str) -> None:
        _ensure_outdir(out_dir)
        exp_by_line = experiments_by_line(dm)

        for line_idx, exps in exp_by_line.items():
            input_row = dm.inputs_by_line.get(line_idx)
            if not input_row:
                continue

            algo_to_points: Dict[str, Dict[int, float]] = {}
            last_iter = input_row.last_iter

            for exp in exps:
                algo = normalize_algo(exp.algorithm)
                for s in exp.steps:
                    if s.iteration < 1 or s.iteration > last_iter:
                        continue
                    y = to_float_or_none(s.relative_travel_time_deviation)
                    if y is None:
                        continue
                    # Keep last value per iteration if duplicates
                    algo_to_points.setdefault(algo, {})[s.iteration] = y

            if not algo_to_points:
                continue

            plt.figure(figsize=(7, 4))
            for algo, pts_map in sorted(algo_to_points.items()):
                xs = sorted(pts_map.keys())
                ys = [pts_map[x] for x in xs]
                plt.plot(xs, ys, marker="o", label=algo)

            plt.title(
                f"Relative travel time deviation by iteration (line #{line_idx})")
            plt.xlabel("Iteration")
            plt.ylabel("Relative travel time deviation")

            # make y-axis logarithmically scaled
            plt.yscale("log")

            plt.xlim(1, max(1, last_iter))
            plt.grid(True, linestyle="--", alpha=0.4)
            plt.legend(title="Algorithm", fontsize=8)

            fname = f"{self.filename_base(input_row)}-{self.filename_suffix()}.png"
            plt.tight_layout()
            plt.savefig(os.path.join(out_dir, fname), dpi=200)
            plt.close()
