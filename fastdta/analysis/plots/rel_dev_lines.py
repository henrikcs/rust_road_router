# plots/rel_dev_lines.py
"""
For each instance: plot relative travel time deviation over iterations,
one line per algorithm (averaged over repetitions).
"""
from __future__ import annotations
from typing import Dict, List
from collections import defaultdict
import matplotlib.pyplot as plt
import numpy as np

from common import (
    DataModel,
    get_experiments_by_instance,
)
from .base import Plot, register_plot, ensure_outdir
from .styles import style_for_algo, get_all_algorithm_colors, get_display_label, MS, LW


def get_rel_deviations(exp) -> Dict[int, float]:
    """Get relative travel time deviations by iteration."""
    devs = {}
    for step in exp.steps:
        if step.relative_travel_time_deviation is not None:
            devs[step.iteration] = step.relative_travel_time_deviation
    return devs


@register_plot
class RelDevLines(Plot):
    @staticmethod
    def key() -> str:
        return "rel-dev-lines"

    @staticmethod
    def display_name() -> str:
        return "Relative travel time deviation (averaged over repetitions)"

    @staticmethod
    def filename_suffix() -> str:
        return "rel-dev-lines"

    def run(self, dm: DataModel, out_dir: str) -> None:
        ensure_outdir(out_dir)
        exp_by_instance = get_experiments_by_instance(dm)

        # Get consistent colors for all algorithms
        algo_colors = get_all_algorithm_colors(dm.algorithms)

        for instance_idx, exps in exp_by_instance.items():
            instance = dm.instances.get(instance_idx)
            if not instance:
                continue

            # Collect deviations per algorithm -> iteration -> list of values (for averaging)
            algo_iter_devs: Dict[str, Dict[int, List[float]]
                                 ] = defaultdict(lambda: defaultdict(list))

            for exp in exps:
                devs = get_rel_deviations(exp)
                for iteration, dev in devs.items():
                    algo_iter_devs[exp.algorithm][iteration].append(dev)

            if not algo_iter_devs:
                continue

            # Create figure
            fig, ax = plt.subplots(figsize=(8, 5))

            max_iter = 0
            for algorithm in sorted(algo_iter_devs.keys()):
                iter_devs = algo_iter_devs[algorithm]

                # Calculate average for each iteration
                iters = sorted(iter_devs.keys())
                avg_values = [np.mean(iter_devs[i]) for i in iters]

                max_iter = max(max_iter, max(iters) if iters else 0)

                algo_style = style_for_algo(algorithm)
                color = algo_style["color"]
                marker = algo_style["marker"]

                ax.plot(
                    iters, avg_values,
                    label=get_display_label(algorithm),
                    color=color,
                    marker=marker,
                    markersize=MS,
                    linewidth=LW,
                )

            # Use log scale for y-axis
            ax.set_yscale("log")

            # Style
            ax.set_ylabel("Relative Travel Time Deviation", fontsize=14)
            ax.set_xlim(0, max(1, max_iter))
            ax.tick_params(axis='both', labelsize=12)
            ax.grid(True, linestyle="--", alpha=0.4)
            ax.legend(title="Algorithm", fontsize=8,
                      loc='upper center', bbox_to_anchor=(0.5, -0.15), ncol=3)

            # Save
            filename = self.filename_for_instance(
                instance, self.filename_suffix()) + ".pdf"
            self.save_plot(fig, out_dir, filename)
