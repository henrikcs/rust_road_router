# common.py
"""
Data models and log parsing for DTA experiment analysis.
Parses .out files directly (eliminating need for sumo-log-parser.rs).
"""
from __future__ import annotations
from dataclasses import dataclass, field
from typing import Optional, List, Dict, Tuple
import re
import os


# -------------------------
# Data models for parsed log structure
# -------------------------

@dataclass
class PhaseTiming:
    """Timing information for a phase (router/simulation)."""
    begin_time: Optional[str] = None
    end_time: Optional[str] = None
    # raw string: H:MM:SS.micro or nanoseconds
    duration_str: Optional[str] = None
    duration_seconds: Optional[float] = None  # parsed to seconds


@dataclass
class PhaseDetail:
    """Detailed phase timing from semicolon-separated log lines.
    Format: <executable>; <path>; <iteration>; <phase-name>; <duration-nanos>
    """
    executable: str = ""
    output_dir: str = ""
    iteration: int = 0
    # e.g., "preprocessing", "cch customization", "cch routing", "postprocessing"
    phase_name: str = ""
    duration_nanos: int = 0
    duration_seconds: float = 0.0


@dataclass
class Step:
    """A single iteration step within an experiment."""
    iteration: int = 0
    router: Optional[PhaseTiming] = None
    simulation: Optional[PhaseTiming] = None
    duration: Optional[PhaseTiming] = None  # total step duration
    relative_travel_time_deviation: Optional[float] = None
    relative_gap: Optional[float] = None
    phase_details: List[PhaseDetail] = field(
        default_factory=list)  # detailed phases for this step


@dataclass
class Experiment:
    """A single experiment run (one algorithm, one repetition, one instance)."""
    algorithm: str = ""  # e.g., "cch", "fastdta_1_1_1", "sumo-sample_1_2_3"
    # base algorithm name without samples: "cch", "fastdta", "sumo-sample"
    algorithm_base: str = ""
    # sample configuration for fastdta/sumo-sample
    samples: Optional[str] = None
    instance_index: int = 0  # line index in CSV (0-based)
    repetition: int = 0  # repetition number (1-based)
    output_dir: str = ""  # full output directory path
    steps: List[Step] = field(default_factory=list)
    preprocessing_phases: List[PhaseDetail] = field(
        default_factory=list)  # iteration -1 phases
    total_duration: Optional[PhaseTiming] = None


@dataclass
class Instance:
    """An instance from the CSV input file."""
    line_index: int = 0  # 0-based line index
    input_dir: str = ""
    prefix: str = ""
    trip_file_name: str = ""
    begin: float = 0.0
    end: float = 0.0
    aggregation: float = 0.0
    convergence_deviation: float = 0.0
    convergence_relgap: float = 0.0
    last_iter: int = 0


@dataclass
class DataModel:
    """Complete data model for analysis."""
    experiments: List[Experiment] = field(default_factory=list)
    instances: Dict[int, Instance] = field(
        default_factory=dict)  # keyed by line_index
    algorithms: List[str] = field(
        default_factory=list)  # unique algorithm names


# -------------------------
# Parsing utilities
# -------------------------

HMS_RE = re.compile(r"^(\d+):([0-5]?\d):([0-5]?\d)(?:\.(\d{1,6}))?$")


def parse_duration_to_seconds(d: Optional[str]) -> Optional[float]:
    """
    Convert duration string to seconds.
    - If purely digits, interpret as nanoseconds.
    - Else parse H:MM:SS(.micro) format.
    """
    if not d:
        return None
    s = d.strip()

    # Check for nanoseconds (all digits)
    if s.isdigit():
        try:
            return int(s) / 1e9
        except ValueError:
            return None

    # Try H:MM:SS.micro format
    m = HMS_RE.match(s)
    if m:
        hours = int(m.group(1))
        minutes = int(m.group(2))
        seconds = int(m.group(3))
        micro = int((m.group(4) or "0").ljust(6, "0"))
        return hours * 3600 + minutes * 60 + seconds + micro / 1e6

    return None


def to_float_or_none(s: Optional[str]) -> Optional[float]:
    """Safely convert string to float."""
    if s is None:
        return None
    try:
        return float(s.strip())
    except Exception:
        return None


def sanitize_for_filename(x: str) -> str:
    """Sanitize a string for use in filenames."""
    base = os.path.basename(x)
    base = re.sub(r"[^\w\-.]+", "_", base)
    return base


def normalize_algo(a: Optional[str]) -> str:
    """Normalize algorithm name to lowercase."""
    return (a or "").strip().lower()


def parse_algorithm_from_path(path: str) -> Tuple[str, str, Optional[str]]:
    """
    Parse algorithm info from output directory path.
    Path format from run_experiments.sh:
      .../base_output_dir/instance_index/algorithm/repetition
      .../base_output_dir/instance_index/algorithm_samples/repetition

    Examples:
      .../0/cch/1 -> algorithm="cch"
      .../0/fastdta_1_1/1 -> algorithm="fastdta_1_1", base="fastdta", samples="1 1"

    Returns: (full_algo_name, base_algo_name, samples_or_none)
    """
    parts = [p for p in path.rstrip('/').split('/') if p]
    if len(parts) < 2:
        return ("unknown", "unknown", None)

    # Path structure: .../instance_index/algorithm/repetition
    # Algorithm is second to last part (before repetition)
    algo_part = parts[-2] if len(parts) >= 2 else parts[-1]

    # Check if it's a sampled algorithm (fastdta_X_X_X or sumo-sample_X_X_X)
    # Note: samples use underscores instead of spaces in directory names
    if algo_part.startswith("fastdta_"):
        samples = algo_part[len("fastdta_"):].replace("_", " ")
        return (algo_part, "fastdta", samples)
    elif algo_part.startswith("sumo-sample_"):
        samples = algo_part[len("sumo-sample_"):].replace("_", " ")
        return (algo_part, "sumo-sample", samples)
    else:
        return (algo_part, algo_part, None)


def parse_instance_and_repetition_from_path(path: str) -> Tuple[int, int]:
    """
    Parse instance index and repetition from output directory path.
    Path format from run_experiments.sh:
      .../base_output_dir/instance_index/algorithm/repetition

    Examples:
      .../0/cch/1 -> instance=0, repetition=1
      .../2/fastdta_1_1/3 -> instance=2, repetition=3

    Returns: (instance_index, repetition)
    """
    parts = [p for p in path.rstrip('/').split('/') if p]

    repetition = 1
    instance_index = 0

    # Path structure: .../instance_index/algorithm/repetition
    # Repetition is last part (must be a number)
    if parts and parts[-1].isdigit():
        repetition = int(parts[-1])

    # Instance index is third from last (before algorithm and repetition)
    # We need to find it by looking for a numeric part before the algorithm
    if len(parts) >= 3:
        # parts[-1] = repetition, parts[-2] = algorithm, parts[-3] = instance_index
        try:
            instance_index = int(parts[-3])
        except ValueError:
            # If parts[-3] is not numeric, search backwards for a numeric part
            for i in range(len(parts) - 3, -1, -1):
                if parts[i].isdigit():
                    instance_index = int(parts[i])
                    break

    return (instance_index, repetition)


# -------------------------
# Log Parser
# -------------------------

class LogParser:
    """Parser for DTA experiment .out log files."""

    def __init__(self):
        # Regex patterns
        self.re_step_start = re.compile(r"^\s*>\s*Executing step\s+(\d+)\s*$")
        self.re_running_router = re.compile(r"^\s*>>\s*Running router\b")
        self.re_running_sim = re.compile(r"^\s*>>\s*Running simulation\b")
        self.re_begin = re.compile(r"^\s*>>>\s*Begin time:\s*(.+?)\s*$")
        self.re_end = re.compile(r"^\s*>>>\s*End time:\s*(.+?)\s*$")
        self.re_duration = re.compile(r"^\s*>>>\s*Duration:\s*(.+?)\s*$")
        self.re_block_end = re.compile(r"^\s*<<\s*$")

        # Relative deviation/gap
        self.re_rel_dev = re.compile(
            r"^\s*<\s*relative travel time deviation in the last.*?steps:\s*(.+?)\s*$"
        )
        self.re_rel_gap = re.compile(
            r"^\s*<\s*relative gap in iteration.*?:\s*(.+?)\s*$"
        )

        # Step end
        self.re_step_end = re.compile(
            r"^\s*<\s*Step\s+(\d+)\s*ended\s*\(duration:\s*(.+?)\)\s*$"
        )

        # Experiment end
        self.re_experiment_end = re.compile(
            r"^\s*dua-iterate ended\s*\(duration:\s*(.+?)\)\s*$"
        )

        # Semicolon phase lines (from rust routers):
        # <executable>; <path>; <iteration>; <phase-name>; <duration-nanos>
        self.re_phase_line = re.compile(
            r"^\s*([^;]+);\s*([^;]+);\s*(-?\d+);\s*([^;]+);\s*(\d+)\s*$"
        )

        # Experiment start line: "Calling duaIterate.py with output directory: <path> and arguments: ..."
        # This line contains the output directory AND the --routing-algorithm argument
        self.re_experiment_start = re.compile(
            r"Calling duaIterate\.py with output directory:\s*(\S+)\s+and arguments:\s*(.+)$"
        )

    def _parse_experiment_start(self, output_dir: str, arguments: str) -> Experiment:
        """Parse experiment info from the 'Calling duaIterate.py' line."""
        exp = Experiment()
        exp.output_dir = output_dir

        # Parse algorithm and instance/repetition from output directory
        full_algo, base_algo, samples = parse_algorithm_from_path(output_dir)
        instance_idx, rep = parse_instance_and_repetition_from_path(output_dir)

        exp.algorithm = full_algo
        exp.algorithm_base = base_algo
        exp.samples = samples
        exp.instance_index = instance_idx
        exp.repetition = rep

        return exp

    def parse_file(self, filepath: str) -> List[Experiment]:
        """Parse a log file and return list of experiments."""
        experiments = []

        with open(filepath, 'r', encoding='utf-8', errors='replace') as f:
            lines = f.readlines()

        current_exp: Optional[Experiment] = None
        current_step: Optional[Step] = None
        current_phase: Optional[str] = None  # 'router' or 'simulation'
        router_timing = PhaseTiming()
        sim_timing = PhaseTiming()

        for line in lines:
            line = line.rstrip('\n')

            # Check for experiment start line (Calling duaIterate.py)
            m = self.re_experiment_start.search(line)
            if m:
                # Save any previous experiment
                if current_exp is not None:
                    if current_step is not None:
                        current_exp.steps.append(current_step)
                        current_step = None
                    experiments.append(current_exp)

                # Start new experiment
                current_exp = self._parse_experiment_start(
                    m.group(1), m.group(2))
                current_step = None
                current_phase = None
                router_timing = PhaseTiming()
                sim_timing = PhaseTiming()
                continue

            # Check for phase detail lines (semicolon format)
            m = self.re_phase_line.match(line)
            if m:
                phase_detail = PhaseDetail(
                    executable=m.group(1).strip(),
                    output_dir=m.group(2).strip(),
                    iteration=int(m.group(3)),
                    phase_name=m.group(4).strip(),
                    duration_nanos=int(m.group(5)),
                    duration_seconds=int(m.group(5)) / 1e9
                )

                # If no current experiment, create one from the phase line path
                # (fallback for logs that don't have the "Calling duaIterate.py" line)
                if current_exp is None:
                    current_exp = Experiment()
                    full_algo, base_algo, samples = parse_algorithm_from_path(
                        phase_detail.output_dir)
                    instance_idx, rep = parse_instance_and_repetition_from_path(
                        phase_detail.output_dir)
                    current_exp.algorithm = full_algo
                    current_exp.algorithm_base = base_algo
                    current_exp.samples = samples
                    current_exp.instance_index = instance_idx
                    current_exp.repetition = rep
                    current_exp.output_dir = phase_detail.output_dir

                # Add to preprocessing or current step
                if phase_detail.iteration == -1:
                    current_exp.preprocessing_phases.append(phase_detail)
                elif current_step is not None:
                    current_step.phase_details.append(phase_detail)
                continue

            # Step start
            m = self.re_step_start.match(line)
            if m:
                # Save previous step
                if current_step is not None and current_exp is not None:
                    current_exp.steps.append(current_step)

                current_step = Step(iteration=int(m.group(1)))
                current_phase = None
                router_timing = PhaseTiming()
                sim_timing = PhaseTiming()
                continue

            # Running router/simulation
            if self.re_running_router.match(line):
                current_phase = 'router'
                continue
            if self.re_running_sim.match(line):
                current_phase = 'simulation'
                continue

            # Begin time
            m = self.re_begin.match(line)
            if m and current_phase:
                if current_phase == 'router':
                    router_timing.begin_time = m.group(1)
                elif current_phase == 'simulation':
                    sim_timing.begin_time = m.group(1)
                continue

            # End time
            m = self.re_end.match(line)
            if m and current_phase:
                if current_phase == 'router':
                    router_timing.end_time = m.group(1)
                elif current_phase == 'simulation':
                    sim_timing.end_time = m.group(1)
                continue

            # Duration
            m = self.re_duration.match(line)
            if m and current_phase:
                dur_str = m.group(1)
                dur_sec = parse_duration_to_seconds(dur_str)
                if current_phase == 'router':
                    router_timing.duration_str = dur_str
                    router_timing.duration_seconds = dur_sec
                elif current_phase == 'simulation':
                    sim_timing.duration_str = dur_str
                    sim_timing.duration_seconds = dur_sec
                continue

            # Block end
            if self.re_block_end.match(line):
                if current_step is not None and current_phase:
                    if current_phase == 'router' and current_step.router is None:
                        current_step.router = PhaseTiming(
                            begin_time=router_timing.begin_time,
                            end_time=router_timing.end_time,
                            duration_str=router_timing.duration_str,
                            duration_seconds=router_timing.duration_seconds
                        )
                    elif current_phase == 'simulation' and current_step.simulation is None:
                        current_step.simulation = PhaseTiming(
                            begin_time=sim_timing.begin_time,
                            end_time=sim_timing.end_time,
                            duration_str=sim_timing.duration_str,
                            duration_seconds=sim_timing.duration_seconds
                        )
                current_phase = None
                continue

            # Relative travel time deviation
            m = self.re_rel_dev.match(line)
            if m and current_step is not None:
                current_step.relative_travel_time_deviation = to_float_or_none(
                    m.group(1))
                continue

            # Relative gap
            m = self.re_rel_gap.match(line)
            if m and current_step is not None:
                current_step.relative_gap = to_float_or_none(m.group(1))
                continue

            # Step end
            m = self.re_step_end.match(line)
            if m:
                if current_step is not None:
                    dur_str = m.group(2)
                    current_step.duration = PhaseTiming(
                        duration_str=dur_str,
                        duration_seconds=parse_duration_to_seconds(dur_str)
                    )
                    if current_exp is not None:
                        current_exp.steps.append(current_step)
                current_step = None
                current_phase = None
                router_timing = PhaseTiming()
                sim_timing = PhaseTiming()
                continue

            # Experiment end
            m = self.re_experiment_end.match(line)
            if m:
                if current_exp is not None:
                    dur_str = m.group(1)
                    current_exp.total_duration = PhaseTiming(
                        duration_str=dur_str,
                        duration_seconds=parse_duration_to_seconds(dur_str)
                    )
                    # Add any remaining step
                    if current_step is not None:
                        current_exp.steps.append(current_step)
                    experiments.append(current_exp)

                current_exp = None
                current_step = None
                current_phase = None
                router_timing = PhaseTiming()
                sim_timing = PhaseTiming()
                continue

        # Flush any remaining experiment
        if current_exp is not None:
            if current_step is not None:
                current_exp.steps.append(current_step)
            experiments.append(current_exp)

        return experiments


# -------------------------
# CSV Parser
# -------------------------

def parse_csv(filepath: str) -> Dict[int, Instance]:
    """
    Parse the experiment CSV file.
    Format: in_dir;prefix;trip_file_name;begin;end;aggregation;convergence_deviation;convergence_relgap;last_iter
    """
    instances = {}
    line_counter = 0

    with open(filepath, 'r', encoding='utf-8') as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith('#'):
                continue

            parts = line.split(';')
            if len(parts) < 9:
                continue

            try:
                instance = Instance(
                    line_index=line_counter,
                    input_dir=parts[0],
                    prefix=parts[1],
                    trip_file_name=parts[2],
                    begin=float(parts[3]),
                    end=float(parts[4]),
                    aggregation=float(parts[5]),
                    convergence_deviation=float(parts[6]),
                    convergence_relgap=float(parts[7]),
                    last_iter=int(parts[8])
                )
                instances[line_counter] = instance
                line_counter += 1
            except (ValueError, IndexError):
                line_counter += 1
                continue

    return instances


# -------------------------
# Build complete data model
# -------------------------

def build_model(out_file: str, csv_file: str) -> DataModel:
    """
    Build the complete data model from .out and .csv files.
    """
    parser = LogParser()
    experiments = parser.parse_file(out_file)
    instances = parse_csv(csv_file)

    # Collect unique algorithms
    algo_set = set()
    for exp in experiments:
        algo_set.add(exp.algorithm)
    algorithms = sorted(list(algo_set))

    return DataModel(
        experiments=experiments,
        instances=instances,
        algorithms=algorithms
    )


# -------------------------
# Query helpers
# -------------------------

def get_experiments_by_instance(dm: DataModel) -> Dict[int, List[Experiment]]:
    """Group experiments by instance index."""
    groups: Dict[int, List[Experiment]] = {}
    for exp in dm.experiments:
        groups.setdefault(exp.instance_index, []).append(exp)
    return groups


def get_experiments_by_instance_and_algorithm(
    dm: DataModel
) -> Dict[Tuple[int, str], List[Experiment]]:
    """Group experiments by (instance_index, algorithm)."""
    groups: Dict[Tuple[int, str], List[Experiment]] = {}
    for exp in dm.experiments:
        key = (exp.instance_index, exp.algorithm)
        groups.setdefault(key, []).append(exp)
    return groups


def get_routing_times(exp: Experiment, skip_first: bool = True) -> List[float]:
    """
    Get all routing times from an experiment.
    Returns times in seconds.
    """
    times = []
    for step in exp.steps:
        if skip_first and step.iteration == 0:
            continue
        if step.router and step.router.duration_seconds is not None:
            times.append(step.router.duration_seconds)
    return times


def get_simulation_times(exp: Experiment, skip_first: bool = True) -> List[float]:
    """
    Get all simulation times from an experiment.
    Returns times in seconds.
    """
    times = []
    for step in exp.steps:
        if skip_first and step.iteration == 0:
            continue
        if step.simulation and step.simulation.duration_seconds is not None:
            times.append(step.simulation.duration_seconds)
    return times


def get_relative_gaps(exp: Experiment) -> Dict[int, float]:
    """
    Get relative gaps by iteration.
    Returns dict mapping iteration -> relative_gap.
    """
    gaps = {}
    for step in exp.steps:
        if step.relative_gap is not None:
            gaps[step.iteration] = step.relative_gap
    return gaps


def get_phase_times_by_name(
    exp: Experiment, phase_names: List[str], skip_first: bool = True
) -> List[float]:
    """
    Get times for specific phase names from phase_details.
    Useful for getting 'cch customization', 'cch routing', etc.
    """
    times = []
    for step in exp.steps:
        if skip_first and step.iteration == 0:
            continue
        for pd in step.phase_details:
            if pd.phase_name.lower() in [p.lower() for p in phase_names]:
                times.append(pd.duration_seconds)
    return times


def get_avg_phase_times_per_step(
    exp: Experiment, phase_name: str, skip_first: bool = True
) -> Dict[int, float]:
    """
    Get average time for a specific phase per iteration.
    Returns dict mapping iteration -> avg_time_seconds.
    """
    times_by_iter: Dict[int, List[float]] = {}
    for step in exp.steps:
        if skip_first and step.iteration == 0:
            continue
        for pd in step.phase_details:
            if pd.phase_name.lower() == phase_name.lower():
                times_by_iter.setdefault(
                    step.iteration, []).append(pd.duration_seconds)

    return {it: sum(ts)/len(ts) for it, ts in times_by_iter.items() if ts}


def is_sampled_algorithm(algo: str) -> bool:
    """Check if algorithm is a sampled one (fastdta or sumo-sample)."""
    return algo.startswith("fastdta") or algo.startswith("sumo-sample")


def get_customization_phase_name(algo_base: str) -> str:
    """Get the customization phase name for an algorithm."""
    if algo_base == "cch":
        return "cch customization"
    elif algo_base == "fastdta":
        return "calibration"  # or could be customization depending on implementation
    return "customization"


def get_routing_phase_name(algo_base: str) -> str:
    """Get the routing phase name for an algorithm."""
    phase_map = {
        "cch": "cch routing",
        "fastdta": "fastdta routing",
        "sumo-sample": "sumo-based routing",
        "dijkstra-rust": "dijkstra routing",
    }
    return phase_map.get(algo_base, "routing")
