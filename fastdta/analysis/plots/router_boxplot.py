# plots/router_boxplot.py
from __future__ import annotations
import os
from typing import Dict, List, Tuple
import matplotlib.pyplot as plt

from common import (
    DataModel, experiments_by_line, parse_duration_to_seconds, normalize_algo,
)
from .base import Plot, register_plot


def _ensure_outdir(out_dir: str):
    os.makedirs(out_dir, exist_ok=True)


@register_plot
class RouterBoxplotByAlgoAgg(Plot):
    @staticmethod
    def key() -> str:
        return "router-boxplot"

    @staticmethod
    def display_name() -> str:
        return "Router duration by (algorithm, aggregation) (boxplot)"

    @staticmethod
    def filename_suffix() -> str:
        return "router-boxplot"

    def run(self, dm: DataModel, out_dir: str) -> None:
        _ensure_outdir(out_dir)
        exp_by_line = experiments_by_line(dm)

        for line_idx, exps in exp_by_line.items():
            input_row = dm.inputs_by_line.get(line_idx)
            if not input_row:
                continue

            groups: Dict[Tuple[str, str], List[float]] = {}
            for exp in exps:
                algo = normalize_algo(exp.algorithm)
                agg = str(exp.aggregation or "")
                for s in exp.steps:
                    if s.iteration < 1 or s.iteration > input_row.last_iter:
                        continue
                    if not s.router or not s.router.duration:
                        continue
                    sec = parse_duration_to_seconds(s.router.duration)
                    if sec is None:
                        continue
                    groups.setdefault((algo, agg), []).append(sec)

            if not groups:
                continue

            labels = []
            data = []
            for key in sorted(groups.keys()):
                labels.append(f"{key[0]} | {key[1]}")
                data.append(groups[key])

            plt.figure(figsize=(max(6, len(labels) * 1.2), 4))
            bp = plt.boxplot(data, patch_artist=True)
            for patch in bp['boxes']:
                patch.set_facecolor('#99c2ff')
            plt.title(
                f"Router duration by (algorithm, aggregation) (line #{line_idx})")
            plt.xlabel("(algorithm, aggregation)")
            plt.ylabel("Router duration (s)")
            plt.grid(True, axis="y", linestyle="--", alpha=0.4)
            plt.xticks(range(1, len(labels) + 1),
                       labels, rotation=30, ha="right")

            fname = f"{self.filename_base(input_row)}-{self.filename_suffix()}.png"
            plt.tight_layout()
            plt.savefig(os.path.join(out_dir, fname), dpi=200)
            plt.close()
