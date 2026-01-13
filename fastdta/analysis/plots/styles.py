# plots/styles.py
from __future__ import annotations
from typing import Dict, Tuple
from common import normalize_algo
import colorsys

# Central style mapping: algorithm -> color + marker
# Base algorithms
ALGO_STYLES: Dict[str, Dict[str, str]] = {
    "cch": {"color": "#E69F00", "marker": "o"},  # orange circles
    "astar": {"color": "#D55E00", "marker": "*"},  # vermillion stars
    "ch": {"color": "#0072B2", "marker": "s"},  # blue squares
    "dijkstra": {"color": "#CC79A7", "marker": "x"},  # pink crosses
    "dijkstra-rust": {"color": "#009E73", "marker": "^"},  # green triangles
}

# Base colors for sampled algorithms (will generate variants)
FASTDTA_BASE_COLOR = "#56B4E9"  # sky blue
SUMO_SAMPLE_BASE_COLOR = "#F0E442"  # yellow

DEFAULT_STYLE = {"color": "#999999", "marker": "o"}

# Markers for different sample configurations
SAMPLE_MARKERS = ["o", "s", "^", "D", "v", "<", ">", "p", "h", "8"]

# Shared sizing constants
MS = 4        # line marker size (points)
LW = 1.2      # line width
S_SCATTER = 30  # scatter marker area (points^2)

# Display labels for algorithms
ALGO_DISPLAY_LABELS: Dict[str, str] = {
    "cch": "CATCHUp",
    "dijkstra-rust": "Dijkstra",
    "fastdta2": "FAST DTA2",
    "fastdta_1_1": "FAST DTA1 ($S_a$)",
    "fastdta_1_1_1": "FAST DTA1 ($S_b$)",
    "fastdta_1_2_3_4": "FAST DTA1 ($S_c$)",
    "dijkstra": "Dijkstra (SUMO)",
    "astar": "A* (SUMO)",
    "ch": "CH (SUMO)",
}

# Cache for dynamically generated styles
_dynamic_styles: Dict[str, Dict[str, str]] = {}
_sample_counter: Dict[str, int] = {"fastdta": 0, "sumo-sample": 0}


def _hex_to_rgb(hex_color: str) -> Tuple[int, int, int]:
    """Convert hex color to RGB tuple."""
    hex_color = hex_color.lstrip('#')
    return tuple(int(hex_color[i:i+2], 16) for i in (0, 2, 4))


def _rgb_to_hex(r: int, g: int, b: int) -> str:
    """Convert RGB to hex color."""
    return f"#{r:02x}{g:02x}{b:02x}"


def _vary_color(base_hex: str, index: int, total_variants: int = 10) -> str:
    """
    Create a color variant by adjusting hue/saturation.
    """
    r, g, b = _hex_to_rgb(base_hex)
    h, l, s = colorsys.rgb_to_hls(r/255, g/255, b/255)

    # Vary hue slightly and adjust lightness
    hue_shift = (index * 0.05) % 1.0
    new_h = (h + hue_shift) % 1.0
    new_l = max(0.2, min(0.8, l + (index % 3 - 1) * 0.1))
    new_s = max(0.3, min(1.0, s))

    new_r, new_g, new_b = colorsys.hls_to_rgb(new_h, new_l, new_s)
    return _rgb_to_hex(int(new_r * 255), int(new_g * 255), int(new_b * 255))


def _get_style_for_sampled_algo(algo: str, base_algo: str, base_color: str) -> Dict[str, str]:
    """Generate a style for a sampled algorithm variant."""
    global _sample_counter

    if algo not in _dynamic_styles:
        idx = _sample_counter.get(base_algo, 0)
        _sample_counter[base_algo] = idx + 1

        color = _vary_color(base_color, idx)
        marker = SAMPLE_MARKERS[idx % len(SAMPLE_MARKERS)]

        _dynamic_styles[algo] = {"color": color, "marker": marker}

    return _dynamic_styles[algo]


def style_for_algo(algo: str) -> Dict[str, str]:
    """
    Return the style dict for an algorithm (case-insensitive), with a default fallback.
    Handles sampled algorithms (fastdta_*, sumo-sample_*) dynamically.
    """
    algo_lower = normalize_algo(algo)

    # Check exact match first
    if algo_lower in ALGO_STYLES:
        return ALGO_STYLES[algo_lower]

    # Check if it's a sampled algorithm
    if algo_lower.startswith("fastdta"):
        return _get_style_for_sampled_algo(algo_lower, "fastdta", FASTDTA_BASE_COLOR)
    elif algo_lower.startswith("sumo-sample"):
        return _get_style_for_sampled_algo(algo_lower, "sumo-sample", SUMO_SAMPLE_BASE_COLOR)

    return DEFAULT_STYLE


def reset_dynamic_styles():
    """Reset dynamically generated styles (useful for testing)."""
    global _dynamic_styles, _sample_counter
    _dynamic_styles = {}
    _sample_counter = {"fastdta": 0, "sumo-sample": 0}


def get_all_algorithm_colors(algorithms: list) -> Dict[str, str]:
    """
    Get a consistent color mapping for a list of algorithms.
    Call this at the start of plotting to ensure consistent colors.
    """
    reset_dynamic_styles()
    return {algo: style_for_algo(algo)["color"] for algo in sorted(algorithms)}


def get_display_label(algo: str) -> str:
    """
    Get the display label for an algorithm.
    Returns the mapped label if available, otherwise returns the original algorithm name.
    """
    algo_lower = normalize_algo(algo)
    return ALGO_DISPLAY_LABELS.get(algo_lower, algo)
