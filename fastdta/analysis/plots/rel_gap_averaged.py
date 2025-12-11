# plots/rel_gap_averaged.py
"""
For each instance: plot relative gap progress over iterations,
averaged over repetitions, one line per algorithm.
"""
from __future__ import annotations
from typing import Dict, List
from collections import defaultdict
import matplotlib.pyplot as plt
import numpy as np

from common import (
    DataModel,
    get_experiments_by_instance,
    get_relative_gaps,
)
from .base import Plot, register_plot, ensure_outdir
from .styles import style_for_algo, get_all_algorithm_colors, MS, LW


@register_plot
class RelGapAveraged(Plot):
    @staticmethod
    def key() -> str:
        return "rel-gap-averaged"

    @staticmethod
    def display_name() -> str:
        return "Relative gap by iteration (averaged over repetitions, per algorithm)"

    @staticmethod
    def filename_suffix() -> str:
        return "rel-gap-averaged"

    def run(self, dm: DataModel, out_dir: str) -> None:
        ensure_outdir(out_dir)
        exp_by_instance = get_experiments_by_instance(dm)

        # Get consistent colors for all algorithms
        algo_colors = get_all_algorithm_colors(dm.algorithms)

        for instance_idx, exps in exp_by_instance.items():
            instance = dm.instances.get(instance_idx)
            if not instance:
                continue

            # Collect gaps per algorithm -> iteration -> list of values (for averaging)
            algo_iter_gaps: Dict[str, Dict[int, List[float]]
                                 ] = defaultdict(lambda: defaultdict(list))

            for exp in exps:
                gaps = get_relative_gaps(exp)
                for iteration, gap in gaps.items():
                    algo_iter_gaps[exp.algorithm][iteration].append(gap)

            if not algo_iter_gaps:
                continue

            # Create figure
            fig, ax = plt.subplots(figsize=(8, 5))

            max_iter = 0
            for algorithm in sorted(algo_iter_gaps.keys()):
                iter_gaps = algo_iter_gaps[algorithm]

                # Calculate average and std for each iteration
                iters = sorted(iter_gaps.keys())
                avg_values = [np.mean(iter_gaps[i]) for i in iters]
                std_values = [np.std(iter_gaps[i]) for i in iters]

                max_iter = max(max_iter, max(iters) if iters else 0)

                algo_style = style_for_algo(algorithm)
                color = algo_style["color"]
                marker = algo_style["marker"]

                # Find minimum value and its position
                if avg_values:
                    min_value = min(avg_values)
                    min_idx = avg_values.index(min_value)
                    min_iter = iters[min_idx]
                else:
                    min_value = None
                    min_iter = None

                # Plot mean line
                label_text = f"{algorithm} (min: {min_value:.6f})" if min_value is not None else algorithm
                ax.plot(
                    iters, avg_values,
                    label=label_text,
                    color=color,
                    marker=marker,
                    markersize=MS,
                    linewidth=LW,
                )

                # Plot std deviation as shaded area
                if len(avg_values) > 0:
                    ax.fill_between(
                        iters,
                        [a - s for a, s in zip(avg_values, std_values)],
                        [a + s for a, s in zip(avg_values, std_values)],
                        alpha=0.2,
                        color=color
                    )

                # Annotate minimum value on the plot
                if min_value is not None:
                    ax.annotate(
                        f'{min_value:.6f}',
                        xy=(min_iter, min_value),
                        xytext=(5, 5),
                        textcoords='offset points',
                        fontsize=7,
                        color=color,
                        bbox=dict(boxstyle='round,pad=0.3',
                                  facecolor='white', edgecolor=color, alpha=0.7)
                    )

            # Style
            ax.set_title(
                f"Relative Gap Progress (averaged)\n{instance.prefix} - {instance.trip_file_name}")
            ax.set_xlabel("Iteration")
            ax.set_ylabel("Relative Gap")
            ax.set_yscale('log')
            ax.set_xlim(0, max(1, max_iter))
            ax.grid(True, linestyle="--", alpha=0.4)
            ax.legend(title="Algorithm", fontsize=8, loc='best')

            # Save
            filename = self.filename_for_instance(
                instance, self.filename_suffix()) + ".pdf"
            self.save_plot(fig, out_dir, filename)
