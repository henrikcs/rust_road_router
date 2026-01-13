# plots/router_boxplot_by_repetition.py
"""
For each instance + algorithm: boxplot of routing times per repetition.
First iteration is ignored in routing time measurements.
"""
from __future__ import annotations
from typing import Dict, List
import matplotlib.pyplot as plt

from common import (
    DataModel,
    get_experiments_by_instance_and_algorithm,
)
from .base import Plot, register_plot, ensure_outdir
from .styles import style_for_algo


def get_routing_times_from_exp(exp, skip_first: bool = True) -> List[float]:
    """Get all routing times from an experiment."""
    times = []
    for step in exp.steps:
        if skip_first and step.iteration == 0:
            continue
        if step.router and step.router.duration_seconds is not None:
            times.append(step.router.duration_seconds)
    return times


@register_plot
class RouterBoxplotByRepetition(Plot):
    @staticmethod
    def key() -> str:
        return "router-boxplot-by-rep"

    @staticmethod
    def display_name() -> str:
        return "Router duration by repetition (boxplot per instance+algorithm)"

    @staticmethod
    def filename_suffix() -> str:
        return "router-boxplot-by-rep"

    def run(self, dm: DataModel, out_dir: str) -> None:
        ensure_outdir(out_dir)
        exp_by_inst_algo = get_experiments_by_instance_and_algorithm(dm)

        for (instance_idx, algorithm), exps in exp_by_inst_algo.items():
            instance = dm.instances.get(instance_idx)
            if not instance:
                continue

            # Collect routing times per repetition
            rep_times: Dict[int, List[float]] = {}
            for exp in exps:
                times = get_routing_times_from_exp(exp, skip_first=True)
                if times:
                    rep_times.setdefault(exp.repetition, []).extend(times)

            if not rep_times:
                continue

            # Sort by repetition number
            reps = sorted(rep_times.keys())
            data = [rep_times[r] for r in reps]
            labels = [f"Rep {r}" for r in reps]

            # Get algorithm color
            algo_style = style_for_algo(algorithm)
            color = algo_style["color"]

            # Create figure
            fig, ax = plt.subplots(figsize=(max(6, len(reps) * 1.2), 5))

            bp = ax.boxplot(data, patch_artist=True, widths=0.6)

            # Color all boxes with algorithm color
            for patch in bp['boxes']:
                patch.set_facecolor(color)
                patch.set_alpha(0.7)

            # Style
            ax.set_ylabel("Router Duration (s)", fontsize=14)
            ax.set_xticks(range(1, len(labels) + 1))
            ax.set_xticklabels(labels)
            ax.tick_params(axis='both', labelsize=12)
            ax.grid(True, axis="y", linestyle="--", alpha=0.4)

            # Save
            filename = self.filename_for_instance_algo(
                instance, algorithm, self.filename_suffix()) + ".pdf"
            self.save_plot(fig, out_dir, filename)
