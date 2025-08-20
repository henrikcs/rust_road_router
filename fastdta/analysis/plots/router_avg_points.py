# plots/router_avg_points.py
from __future__ import annotations
import os
import matplotlib.pyplot as plt

from common import (
    DataModel, experiments_by_line, parse_duration_to_seconds, normalize_algo,
)
from .base import Plot, register_plot
from .styles import style_for_algo, S_SCATTER  # NEW: shared styles


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

            # Collect one or more points per algorithm: (k, avg_sec)
            points_by_algo = {}

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
                points_by_algo.setdefault(algo, []).append((k, avg_sec))

            if not points_by_algo:
                continue

            plt.figure(figsize=(7, 4))
            for algo, pts in sorted(points_by_algo.items()):
                st = style_for_algo(algo)
                xs = [p[0] for p in pts]
                ys = [p[1] for p in pts]
                plt.scatter(
                    xs, ys,
                    label=algo,
                    color=st["color"],
                    marker=st["marker"],
                    s=S_SCATTER,
                )

            plt.title(
                f"Avg router duration vs. max iteration reached (line #{line_idx})")
            plt.xlabel("Iteration (positioned at k = max reached)")
            plt.ylabel("Average router duration (s)")
            all_x = [x for pts in points_by_algo.values() for (x, _) in pts]
            plt.xlim(1, max(input_row.last_iter, max(all_x)))
            plt.grid(True, linestyle="--", alpha=0.4)
            plt.legend(title="Algorithm", fontsize=8)

            fname = f"{self.filename_base(input_row)}-{self.filename_suffix()}.pdf"
            plt.tight_layout()
            plt.savefig(os.path.join(out_dir, fname))
            plt.close()
