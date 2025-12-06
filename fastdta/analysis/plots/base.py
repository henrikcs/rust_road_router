# plots/base.py
from __future__ import annotations
from abc import ABC, abstractmethod
from typing import Dict, Type, Optional
from common import sanitize_for_filename, Instance, DataModel
import os

PLOT_REGISTRY: Dict[str, Type["Plot"]] = {}


def register_plot(cls: Type["Plot"]) -> Type["Plot"]:
    key = cls.key()
    if key in PLOT_REGISTRY:
        raise ValueError(f"Duplicate plot key: {key}")
    PLOT_REGISTRY[key] = cls
    return cls


def ensure_outdir(out_dir: str):
    """Ensure output directory exists."""
    os.makedirs(out_dir, exist_ok=True)


class Plot(ABC):
    @staticmethod
    @abstractmethod
    def key() -> str:
        """Unique key used for selection and filename suffix."""
        raise NotImplementedError

    @staticmethod
    def display_name() -> str:
        """Human readable plot name."""
        return ""

    @staticmethod
    def filename_suffix() -> str:
        """Suffix part used in output filenames."""
        return Plot.key()

    @abstractmethod
    def run(self, dm: DataModel, out_dir: str) -> None:
        """Generate this plot for all experiments (grouped by input line)."""
        raise NotImplementedError

    @staticmethod
    def filename_for_instance(instance: Instance, suffix: str) -> str:
        """Generate filename for instance-level plots."""
        return "{}-{}-agg{}-{}".format(
            sanitize_for_filename(instance.prefix),
            sanitize_for_filename(instance.trip_file_name),
            int(instance.aggregation),
            suffix,
        )

    @staticmethod
    def filename_for_instance_algo(instance: Instance, algorithm: str, suffix: str) -> str:
        """Generate filename for instance+algorithm level plots."""
        return "{}-{}-agg{}-{}-{}".format(
            sanitize_for_filename(instance.prefix),
            sanitize_for_filename(instance.trip_file_name),
            int(instance.aggregation),
            sanitize_for_filename(algorithm),
            suffix,
        )

    @staticmethod
    def save_plot(fig, out_dir: str, filename: str, close: bool = True):
        """Save a matplotlib figure to file."""
        import matplotlib.pyplot as plt
        ensure_outdir(out_dir)
        filepath = os.path.join(out_dir, filename)
        fig.tight_layout()
        fig.savefig(filepath, bbox_inches='tight', dpi=150)
        if close:
            plt.close(fig)
