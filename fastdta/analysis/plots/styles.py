# plots/styles.py
from __future__ import annotations
from typing import Dict
from common import normalize_algo

# Central style mapping: algorithm -> color + marker
ALGO_STYLES: Dict[str, Dict[str, str]] = {
    "cch": {"color": "orange",  "marker": "o"},  # circles
    "astar": {"color": "red",   "marker": "*"},  # stars
    "ch": {"color": "blue",     "marker": "s"},  # squares
    "dijkstra": {"color": "purple", "marker": "x"},  # crosses
    "dijkstra-rust": {"color": "brown",  "marker": "^"},  # triangles
}
DEFAULT_STYLE = {"color": "black", "marker": "o"}

# Shared sizing constants
MS = 4        # line marker size (points)
LW = 1.2      # line width
S_SCATTER = 30  # scatter marker area (points^2)


def style_for_algo(algo: str) -> Dict[str, str]:
    """Return the style dict for an algorithm (case-insensitive), with a default fallback."""
    return ALGO_STYLES.get(normalize_algo(algo), DEFAULT_STYLE)
