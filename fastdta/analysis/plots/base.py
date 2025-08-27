# plots/base.py
from __future__ import annotations
from abc import ABC, abstractmethod
from typing import Dict, Type
from common import sanitize_for_filename

PLOT_REGISTRY: Dict[str, Type["Plot"]] = {}


def register_plot(cls: Type["Plot"]) -> Type["Plot"]:
    key = cls.key()
    if key in PLOT_REGISTRY:
        raise ValueError(f"Duplicate plot key: {key}")
    PLOT_REGISTRY[key] = cls
    return cls


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
    def run(self, dm, out_dir: str) -> None:
        """Generate this plot for all experiments (grouped by input line)."""
        raise NotImplementedError

    @staticmethod
    def filename_base(input_row) -> str:
        # <prefix>-<trip>-<aggregation>-<relative-gap>
        return "{}-{}-{}-{}".format(
            sanitize_for_filename(input_row.prefix),
            sanitize_for_filename(input_row.trip_file_name),
            sanitize_for_filename(str(input_row.aggregation)),
            sanitize_for_filename(str(input_row.relative_gap)),
        )
