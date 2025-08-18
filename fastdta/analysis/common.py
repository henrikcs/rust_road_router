# common.py
from __future__ import annotations
from dataclasses import dataclass, field
from typing import Optional, List, Dict, Tuple
import json
import csv
import os
import re

# -------------------------
# Data models (mirroring parser_model.rs and test.json) [1][3]
# -------------------------


@dataclass
class PhaseTiming:
    begin_time: Optional[str] = None
    end_time: Optional[str] = None
    duration: Optional[str] = None  # may be ns or H:MM:SS.micro


@dataclass
class Step:
    iteration: int = 0
    router: Optional[PhaseTiming] = None
    simulation: Optional[PhaseTiming] = None
    duration: Optional[str] = None
    relative_travel_time_deviation: Optional[str] = None


@dataclass
class OutputDirMeta:
    full: str = ""
    path_root: Optional[str] = None
    timestamp: Optional[str] = None
    experiments_file_line: Optional[str] = None
    algorithm: Optional[str] = None  # last path segment


@dataclass
class PhaseLine:
    executable: str = ""
    output_dir: str = ""
    iteration: int = 0
    algorithm_phase: str = ""
    phase_duration: str = ""  # may be ns or H:MM:SS.micro


@dataclass
class Experiment:
    algorithm: Optional[str] = None
    aggregation: Optional[str] = None
    output_dir: Optional[OutputDirMeta] = None
    steps: List[Step] = field(default_factory=list)
    phases: List[PhaseLine] = field(default_factory=list)
    total_duration: Optional[str] = None  # may be ns or H:MM:SS.micro


@dataclass
class LogSummary:
    experiments: List[Experiment] = field(default_factory=list)

# CSV schema: in_dir;prefix;trip_file_name;aggregation;begin;end;convergence_deviation;first_iter;last_iter;seed


@dataclass
class InputRow:
    in_dir: str
    prefix: str
    trip_file_name: str
    aggregation: str
    begin: str
    end: str
    convergence_deviation: str
    first_iter: int
    last_iter: int
    seed: int
    # implicit key = line index in the CSV (0-based)
    line_index: int = -1


@dataclass
class DataModel:
    summary: LogSummary
    inputs_by_line: Dict[int, InputRow]  # key: experiments_file_line (int)


# -------------------------
# Utilities
# -------------------------

HMS_RE = re.compile(r"^(\d+):([0-5]\d):([0-5]\d)(?:\.(\d{1,6}))?$")


def parse_duration_to_seconds(d: Optional[str]) -> Optional[float]:
    """
    Convert duration string to seconds.
    - If purely digits (int), interpret as nanoseconds.
    - Else parse H:MM:SS(.micro) per your data format.
    Returns None if d is falsy or unparsable.
    """
    if not d:
        return None
    s = d.strip()
    if s.isdigit():
        # nanoseconds
        try:
            return int(s) / 1e9
        except ValueError:
            return None
    m = HMS_RE.match(s)
    if m:
        hours = int(m.group(1))
        minutes = int(m.group(2))
        seconds = int(m.group(3))
        micro = int((m.group(4) or "0").ljust(6, "0"))
        return hours * 3600 + minutes * 60 + seconds + micro / 1e6
    return None


def to_float_or_none(s: Optional[str]) -> Optional[float]:
    if s is None:
        return None
    try:
        return float(s.strip())
    except Exception:
        return None


def sanitize_for_filename(x: str) -> str:
    # Keep it simple: lower, replace path separators and spaces
    base = os.path.basename(x)
    base = re.sub(r"[^\w\-.]+", "_", base)
    return base


# -------------------------
# Deserialization
# -------------------------

def _obj_to_phase_timing(obj: dict) -> PhaseTiming:
    if obj is None:
        return PhaseTiming()
    return PhaseTiming(
        begin_time=obj.get("begin_time"),
        end_time=obj.get("end_time"),
        duration=obj.get("duration"),
    )


def _obj_to_step(obj: dict) -> Step:
    return Step(
        iteration=int(obj.get("iteration", 0)),
        router=_obj_to_phase_timing(obj.get("router")),
        simulation=_obj_to_phase_timing(obj.get("simulation")),
        duration=obj.get("duration"),
        relative_travel_time_deviation=obj.get(
            "relative_travel_time_deviation"),
    )


def _obj_to_output_dir(obj: dict) -> OutputDirMeta:
    if obj is None:
        return OutputDirMeta()
    return OutputDirMeta(
        full=obj.get("full", ""),
        path_root=obj.get("path_root"),
        timestamp=obj.get("timestamp"),
        experiments_file_line=obj.get("experiments_file_line"),
        algorithm=obj.get("algorithm"),
    )


def _obj_to_phase_line(obj: dict) -> PhaseLine:
    return PhaseLine(
        executable=obj.get("executable", ""),
        output_dir=obj.get("output_dir", ""),
        iteration=int(obj.get("iteration", 0)),
        algorithm_phase=obj.get("algorithm_phase", ""),
        phase_duration=obj.get("phase_duration", ""),
    )


def _obj_to_experiment(obj: dict) -> Experiment:
    steps = [_obj_to_step(s) for s in obj.get("steps", [])]
    phases = [_obj_to_phase_line(p) for p in obj.get("phases", [])]
    return Experiment(
        algorithm=obj.get("algorithm"),
        aggregation=obj.get("aggregation"),
        output_dir=_obj_to_output_dir(obj.get("output_dir")),
        steps=steps,
        phases=phases,
        total_duration=obj.get("total_duration"),
    )


def load_json_summary(path: str) -> LogSummary:
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)

    # data may have a root like {"experiments": [...]}
    experiments_raw = data.get("experiments", [])
    experiments = [_obj_to_experiment(e) for e in experiments_raw]
    return LogSummary(experiments=experiments)


def _split_potentially_double_line(raw_line: str) -> List[str]:
    """
    Your sample CSV shows two records on one physical line separated by ': ' [2].
    This splits such lines into individual logical records.
    """
    if ":" in raw_line:
        parts = [p.strip() for p in raw_line.split(":") if p.strip()]
        return parts
    return [raw_line.strip()]


def _parse_csv_record(row_text: str, expected_cols: int = 10) -> Optional[List[str]]:
    # Use csv module to handle quoting; but we need semicolon delimiter.
    # We feed a single line to csv.reader.
    reader = csv.reader([row_text], delimiter=";")
    for rec in reader:
        if len(rec) >= expected_cols:
            return rec[:expected_cols]
    return None


def load_csv_parameters(path: str) -> Dict[int, InputRow]:
    """
    Load CSV into a mapping from line index (0-based) to InputRow.
    Handles the odd case where one physical line contains multiple records separated by ':' [2].
    """
    inputs: Dict[int, InputRow] = {}
    current_index = 0
    with open(path, "r", encoding="utf-8") as f:
        for physical_line in f:
            physical_line = physical_line.strip()
            if not physical_line:
                continue
            logical_lines = _split_potentially_double_line(physical_line)
            for ll in logical_lines:
                rec = _parse_csv_record(ll)
                if not rec:
                    continue
                in_dir, prefix, trip, aggregation, begin, end, conv_dev, first_iter, last_iter, seed = rec
                row = InputRow(
                    in_dir=in_dir,
                    prefix=prefix,
                    trip_file_name=trip,
                    aggregation=str(aggregation),
                    begin=str(begin),
                    end=str(end),
                    convergence_deviation=str(conv_dev),
                    first_iter=int(first_iter),
                    last_iter=int(last_iter),
                    seed=int(seed),
                    line_index=current_index,
                )
                inputs[current_index] = row
                current_index += 1
    return inputs


def build_model(json_path: str, csv_path: str) -> DataModel:
    summary = load_json_summary(json_path)
    inputs_by_line = load_csv_parameters(csv_path)
    return DataModel(summary=summary, inputs_by_line=inputs_by_line)

# -------------------------
# Helpers to query data
# -------------------------


def normalize_algo(a: Optional[str]) -> str:
    return (a or "").strip().lower()


def experiments_by_line(dm: DataModel) -> Dict[int, List[Experiment]]:
    """
    Group experiments by experiments_file_line.
    """
    groups: Dict[int, List[Experiment]] = {}
    for exp in dm.summary.experiments:
        line_str = exp.output_dir.experiments_file_line if exp.output_dir else None
        if line_str is None:
            continue
        try:
            line_idx = int(line_str)
        except Exception:
            continue
        groups.setdefault(line_idx, []).append(exp)
    return groups


def steps_in_range(exp: Experiment, min_iter: int, max_iter: int) -> List[Step]:
    return [s for s in exp.steps if s.iteration >= min_iter and s.iteration <= max_iter]


def max_iteration(exp: Experiment) -> int:
    if not exp.steps:
        return 0
    return max((s.iteration for s in exp.steps if s.iteration >= 0), default=0)
