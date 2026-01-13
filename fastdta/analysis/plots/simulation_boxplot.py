# plots/simulation_boxplot.py
"""
For each instance: boxplot of simulation times over all repetitions/iterations,
one boxplot per algorithm.
"""
from __future__ import annotations
from typing import Dict, List
import matplotlib.pyplot as plt

from common import (
    DataModel,
    get_experiments_by_instance,
    get_simulation_times,
)
from .base import Plot, register_plot, ensure_outdir
from .styles import style_for_algo, get_all_algorithm_colors


@register_plot
class SimulationBoxplot(Plot):
    @staticmethod
    def key() -> str:
        return "simulation-boxplot"

    @staticmethod
    def display_name() -> str:
        return "Simulation duration by algorithm (boxplot per instance)"

    @staticmethod
    def filename_suffix() -> str:
        return "simulation-boxplot"

    def run(self, dm: DataModel, out_dir: str) -> None:
        ensure_outdir(out_dir)
        exp_by_instance = get_experiments_by_instance(dm)

        # Get consistent colors for all algorithms
        algo_colors = get_all_algorithm_colors(dm.algorithms)

        for instance_idx, exps in exp_by_instance.items():
            instance = dm.instances.get(instance_idx)
            if not instance:
                continue

            # Collect simulation times per algorithm (all repetitions combined)
            algo_times: Dict[str, List[float]] = {}
            for exp in exps:
                times = get_simulation_times(exp, skip_first=True)
                if times:
                    algo_times.setdefault(exp.algorithm, []).extend(times)

            if not algo_times:
                continue

            # Sort algorithms for consistent ordering
            algos = sorted(algo_times.keys())
            data = [algo_times[a] for a in algos]
            colors = [algo_colors.get(a, "#999999") for a in algos]

            # Create figure
            fig, ax = plt.subplots(figsize=(max(8, len(algos) * 1.5), 5))

            bp = ax.boxplot(data, patch_artist=True, widths=0.6)

            # Color boxes
            for patch, color in zip(bp['boxes'], colors):
                patch.set_facecolor(color)
                patch.set_alpha(0.7)

            # Style
            ax.set_ylabel("Simulation Duration (s)", fontsize=14)
            ax.set_xticks(range(1, len(algos) + 1))
            ax.set_xticklabels(algos, rotation=45, ha="right")
            ax.tick_params(axis='both', labelsize=12)
            ax.grid(True, axis="y", linestyle="--", alpha=0.4)

            # Save
            filename = self.filename_for_instance(
                instance, self.filename_suffix()) + ".pdf"
            self.save_plot(fig, out_dir, filename)
