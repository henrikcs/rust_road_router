# plots/rel_gap_by_repetition.py
"""
For each instance + algorithm: plot relative gap progress over iterations,
one line per repetition.
"""
from __future__ import annotations
from typing import Dict, List
import matplotlib.pyplot as plt

from common import (
    DataModel,
    get_experiments_by_instance_and_algorithm,
    get_relative_gaps,
)
from .base import Plot, register_plot, ensure_outdir
from .styles import style_for_algo, MS, LW


# Different line styles for repetitions
REP_LINESTYLES = ['-', '--', '-.', ':', (0, (3, 1, 1, 1)), (0, (5, 2))]


@register_plot
class RelGapByRepetition(Plot):
    @staticmethod
    def key() -> str:
        return "rel-gap-by-rep"

    @staticmethod
    def display_name() -> str:
        return "Relative gap by iteration (one line per repetition)"

    @staticmethod
    def filename_suffix() -> str:
        return "rel-gap-by-rep"

    def run(self, dm: DataModel, out_dir: str) -> None:
        ensure_outdir(out_dir)
        exp_by_inst_algo = get_experiments_by_instance_and_algorithm(dm)

        for (instance_idx, algorithm), exps in exp_by_inst_algo.items():
            instance = dm.instances.get(instance_idx)
            if not instance:
                continue

            # Collect relative gaps per repetition
            rep_gaps: Dict[int, Dict[int, float]] = {}  # rep -> {iter -> gap}
            for exp in exps:
                gaps = get_relative_gaps(exp)
                if gaps:
                    rep_gaps[exp.repetition] = gaps

            if not rep_gaps:
                continue

            # Get algorithm style
            algo_style = style_for_algo(algorithm)
            base_color = algo_style["color"]
            marker = algo_style["marker"]

            # Create figure
            fig, ax = plt.subplots(figsize=(8, 5))

            max_iter = 0
            for rep_idx, (rep, gaps) in enumerate(sorted(rep_gaps.items())):
                iters = sorted(gaps.keys())
                values = [gaps[i] for i in iters]
                max_iter = max(max_iter, max(iters) if iters else 0)

                linestyle = REP_LINESTYLES[rep_idx % len(REP_LINESTYLES)]
                ax.plot(
                    iters, values,
                    label=f"Rep {rep}",
                    color=base_color,
                    marker=marker,
                    markersize=MS,
                    linewidth=LW,
                    linestyle=linestyle,
                    alpha=0.8
                )

            # Style
            ax.set_ylabel("Relative Gap", fontsize=14)
            ax.set_yscale('log')
            ax.set_xlim(0, max(1, max_iter))
            ax.tick_params(axis='both', labelsize=12)
            ax.grid(True, linestyle="--", alpha=0.4)
            ax.legend(title="Repetition", fontsize=8)

            # Save
            filename = self.filename_for_instance_algo(
                instance, algorithm, self.filename_suffix()) + ".pdf"
            self.save_plot(fig, out_dir, filename)
