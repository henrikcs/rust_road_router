# plots/__init__.py
import pkgutil
import importlib
from typing import List
from .base import Plot, PLOT_REGISTRY


def discover_plots() -> List[Plot]:
    # Import all submodules (except base) to trigger @register_plot
    pkg = __name__
    for m in pkgutil.iter_modules(__path__):  # type: ignore[name-defined]
        if m.name in {"base", "__init__"}:
            continue
        importlib.import_module(f"{pkg}.{m.name}")
    # Instantiate in a stable order (by key)
    return [cls() for key, cls in sorted(PLOT_REGISTRY.items(), key=lambda kv: kv[0])]
