# plots/router_avg_points.py
from __future__ import annotations
import os
import matplotlib.pyplot as plt

from common import (
    DataModel, experiments_by_line, parse_duration_to_seconds, normalize_algo,
)
from .base import Plot, register_plot


def _ensure_outdir(out_dir: str):
    os.makedirs(out_dir, exist_ok=True)


@register_plot
class RouterAvgPoints(Plot):
    @staticmethod
    def key() -> str:
        return "router-avg-point"

    @staticmethod
    def display_name() -> str:
        return "Average router duration vs. max iteration (points)"

    @staticmethod
    def filename_suffix() -> str:
        return "router-avg-point"

    def run(self, dm: DataModel, out_dir: str) -> None:
        _ensure_outdir(out_dir)
        exp_by_line = experiments_by_line(dm)

        for line_idx, exps in exp_by_line.items():
            input_row = dm.inputs_by_line.get(line_idx)
            if not input_row:
                continue

            xs = []
            ys = []
            labels = []

            for exp in exps:
                algo = normalize_algo(exp.algorithm)
                k = min(
                    input_row.last_iter,
                    max((s.iteration for s in exp.steps if s.iteration >= 1), default=0),
                )
                if k < 1:
                    continue
                durations = []
                for s in exp.steps:
                    if 1 <= s.iteration <= k and s.router and s.router.duration:
                        sec = parse_duration_to_seconds(s.router.duration)
                        if sec is not None:
                            durations.append(sec)
                if not durations:
                    continue
                avg_sec = sum(durations) / len(durations)
                xs.append(k)
                ys.append(avg_sec)
                labels.append(algo)

            if not xs:
                continue

            plt.figure(figsize=(7, 4))
            plt.scatter(xs, ys, c="tab:blue")
            for xi, yi, lbl in zip(xs, ys, labels):
                plt.annotate(lbl, (xi, yi), textcoords="offset points",
                             xytext=(5, 5), fontsize=8)

            plt.title(
                f"Avg router duration vs. max iteration reached (line #{line_idx})")
            plt.xlabel("Iteration (positioned at k = max reached)")
            plt.ylabel("Average router duration (s)")
            plt.xlim(1, max(input_row.last_iter, max(xs)))
            plt.grid(True, linestyle="--", alpha=0.4)

            fname = f"{self.filename_base(input_row)}-{self.filename_suffix()}.png"
            plt.tight_layout()
            plt.savefig(os.path.join(out_dir, fname), dpi=200)
            plt.close()
