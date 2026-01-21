# plots/routing_customization_breakdown.py
"""
For fastdta and sumo-sample algorithms: plot average routing vs customization time
per repetition. Shows breakdown of routing phases.
"""
from __future__ import annotations
from typing import Dict, List, Tuple
from collections import defaultdict
import matplotlib.pyplot as plt
import numpy as np

from common import (
    DataModel,
    get_experiments_by_instance_and_algorithm,
    get_phase_times_by_name,
    is_sampled_algorithm,
)
from .base import Plot, register_plot, ensure_outdir
from .styles import style_for_algo


# Phase names for different algorithms
PHASE_MAPPING = {
    "fastdta": {
        "routing": ["fastdta routing"],
        "calibration": ["calibration"],
        "sample": ["sample"],
        "preprocessing": ["preprocessing"],
        "postprocessing": ["postprocessing"],
    },
    "sumo-sample": {
        "routing": ["sumo-based routing"],
        "sample": ["sample"],
        "preprocessing": ["preprocessing", "read edge ids", "read queries"],
        "postprocessing": ["postprocessing"],
    },
    "cch": {
        "routing": ["cch routing"],
        "customization": ["cch customization"],
        "preprocessing": ["preprocessing"],
        "postprocessing": ["postprocessing"],
    },
    "dijkstra-rust": {
        "routing": ["dijkstra routing"],
        "preprocessing": ["preprocessing"],
        "postprocessing": ["postprocessing"],
    },
}


def get_phase_breakdown(exp, phase_mapping: Dict[str, List[str]], skip_first: bool = True) -> Dict[str, List[float]]:
    """
    Get time breakdown by phase category for an experiment.
    Returns dict: phase_category -> list of times (one per iteration).
    """
    result: Dict[str, List[float]] = defaultdict(list)

    for step in exp.steps:
        if skip_first and step.iteration == 0:
            continue

        # Aggregate times by phase category for this step
        step_times: Dict[str, float] = defaultdict(float)
        for pd in step.phase_details:
            for category, phase_names in phase_mapping.items():
                if pd.phase_name.lower() in [p.lower() for p in phase_names]:
                    step_times[category] += pd.duration_seconds
                    break

        # Add to results
        for category in phase_mapping.keys():
            if category in step_times:
                result[category].append(step_times[category])

    return result


@register_plot
class RoutingCustomizationBreakdown(Plot):
    @staticmethod
    def key() -> str:
        return "routing-breakdown"

    @staticmethod
    def display_name() -> str:
        return "Routing phase breakdown (for sampled algorithms)"

    @staticmethod
    def filename_suffix() -> str:
        return "routing-breakdown"

    def run(self, dm: DataModel, out_dir: str) -> None:
        ensure_outdir(out_dir)
        exp_by_inst_algo = get_experiments_by_instance_and_algorithm(dm)

        for (instance_idx, algorithm), exps in exp_by_inst_algo.items():
            instance = dm.instances.get(instance_idx)
            if not instance:
                continue

            # Get the base algorithm to determine phase mapping
            base_algo = exps[0].algorithm_base if exps else algorithm

            # Get phase mapping for this algorithm
            phase_mapping = PHASE_MAPPING.get(base_algo, {})
            if not phase_mapping:
                continue

            # Collect phase breakdown per repetition
            # rep -> {phase -> avg_time}
            rep_breakdowns: Dict[int, Dict[str, float]] = {}

            for exp in exps:
                breakdown = get_phase_breakdown(
                    exp, phase_mapping, skip_first=True)
                if breakdown:
                    # Calculate average for each phase
                    avg_breakdown = {}
                    for phase, times in breakdown.items():
                        if times:
                            avg_breakdown[phase] = np.mean(times)
                    if avg_breakdown:
                        rep_breakdowns[exp.repetition] = avg_breakdown

            if not rep_breakdowns:
                continue

            # Get algorithm style for base color
            algo_style = style_for_algo(algorithm)

            # Prepare data for stacked bar chart
            reps = sorted(rep_breakdowns.keys())
            phases = list(phase_mapping.keys())

            # Color map for phases
            phase_colors = plt.cm.Set2(np.linspace(0, 1, len(phases)))

            # Create figure
            fig, ax = plt.subplots(figsize=(max(6, len(reps) * 1.5), 5))

            x = np.arange(len(reps))
            width = 0.6
            bottom = np.zeros(len(reps))

            for phase_idx, phase in enumerate(phases):
                heights = [rep_breakdowns[r].get(phase, 0) for r in reps]
                ax.bar(x, heights, width, label=phase,
                       bottom=bottom, color=phase_colors[phase_idx])
                bottom += heights

            # Style
            ax.set_ylabel("Average Time (s)", fontsize=14)
            ax.set_xticks(x)
            ax.set_xticklabels([f"Rep {r}" for r in reps])
            ax.tick_params(axis='both', labelsize=12)
            ax.legend(title="Phase", fontsize=8, loc='upper center',
                      bbox_to_anchor=(0.5, -0.15), ncol=3)
            ax.grid(True, axis="y", linestyle="--", alpha=0.4)

            # Save
            filename = self.filename_for_instance_algo(
                instance, algorithm, self.filename_suffix()) + ".pdf"
            self.save_plot(fig, out_dir, filename)
