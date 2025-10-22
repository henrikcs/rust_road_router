#!/usr/bin/env python3
"""
Generate random network instances for traffic assignment experiments.

This script creates random SUMO networks with random trip demands, and generates
CSV files with experiment parameters for FastDTA and runtime experiments.
"""

import argparse
import os
import random
import subprocess
import sys
import xml.etree.ElementTree as ET
from pathlib import Path


def parse_arguments():
    """Parse command line arguments."""
    parser = argparse.ArgumentParser(
        description="Generate random network instances for experiments"
    )
    parser.add_argument(
        "-n", "--num-instances",
        type=int,
        required=True,
        help="Number of instances to create"
    )
    parser.add_argument(
        "--min-nodes",
        type=int,
        required=True,
        help="Minimum number of nodes in each instance"
    )
    parser.add_argument(
        "--max-nodes",
        type=int,
        required=True,
        help="Maximum number of nodes in each instance"
    )
    parser.add_argument(
        "--min-trips",
        type=int,
        required=True,
        help="Minimum number of trips in each instance"
    )
    parser.add_argument(
        "--max-trips",
        type=int,
        required=True,
        help="Maximum number of trips in each instance"
    )
    parser.add_argument(
        "--seed",
        type=int,
        default=42,
        help="Random seed for reproducibility"
    )
    parser.add_argument(
        "--output-folder",
        type=str,
        required=True,
        help="Folder to save the instances"
    )
    return parser.parse_args()


def create_network(instance_folder, prefix, num_nodes, seed):
    """Generate a random network using netgenerate."""
    net_file = os.path.join(instance_folder, f"{prefix}.net.xml")

    cmd = [
        "netgenerate",
        "-r",
        "--rand.grid",
        "--rand.iterations", str(num_nodes),
        "--seed", str(seed),
        "--bidi-probability=0.95",
        "-L", "2",
        "--random-lanenumber",
        "--roundabouts.guess=false",
        "-o", net_file
    ]

    print(f"  Creating network: {' '.join(cmd)}")
    subprocess.run(cmd, check=True, capture_output=True)

    # Convert to .nod.xml, .edg.xml, and .con.xml
    cmd_convert = [
        "netconvert",
        "-s", net_file,
        "-p", os.path.join(instance_folder, prefix)
    ]

    print(f"  Converting network: {' '.join(cmd_convert)}")
    subprocess.run(cmd_convert, check=True, capture_output=True)

    return net_file


def create_trips_for_interval(net_file, instance_folder, prefix, interval_begin,
                              interval_end, interval_trips, sumo_tools_path):
    """Create trips for a specific time interval."""
    interval_trip_file = os.path.join(
        instance_folder,
        f"{prefix}-{interval_begin}-{interval_end}.trips.xml"
    )

    # Calculate period between trips
    interval_duration = interval_end - interval_begin
    period = interval_duration / \
        interval_trips if interval_trips > 0 else interval_duration

    # Find randomTrips.py
    random_trips_script = os.path.join(sumo_tools_path, "randomTrips.py")

    cmd = [
        "python3",
        random_trips_script,
        "-n", net_file,
        "--validate",
        "--fringe-factor", "10",
        "-b", str(interval_begin),
        "-e", str(interval_end),
        "-p", str(period),
        "-o", interval_trip_file,
        "--trip-attributes", 'departLane="best" departSpeed="max" departPos="base" arrivalPos="max"'
    ]

    print(
        f"    Creating trips for interval [{interval_begin}, {interval_end}]: {interval_trips} trips")
    subprocess.run(cmd, check=True, capture_output=True)

    return interval_trip_file


def merge_trip_files(trip_files, output_file):
    """Merge multiple trip XML files into a single file with unique sequential IDs."""
    # Parse all trip files
    all_trips = []

    for trip_file in trip_files:
        tree = ET.parse(trip_file)
        root = tree.getroot()
        for trip in root.findall('trip'):
            all_trips.append(trip)

    # Sort trips by departure time
    all_trips.sort(key=lambda t: float(t.get('depart', '0')))

    # Reassign IDs sequentially based on sorted order
    for idx, trip in enumerate(all_trips, start=1):
        trip.set('id', str(idx))

    # Create merged XML
    root = ET.Element('routes')
    for trip in all_trips:
        root.append(trip)

    # Write to file
    tree = ET.ElementTree(root)
    ET.indent(tree, space="    ")
    tree.write(output_file, encoding='utf-8', xml_declaration=True)

    print(
        f"  Merged {len(trip_files)} trip files into {output_file} with {len(all_trips)} trips (IDs 1-{len(all_trips)})")


def create_trips(net_file, instance_folder, prefix, num_trips, sumo_tools_path):
    """Create trips distributed across time intervals."""
    # Define intervals and percentages
    intervals = [
        (0, 21600, 0.10),      # 00:00-06:00 : 10%
        (21600, 32400, 0.30),  # 06:00-09:00 : 30%
        (32400, 61200, 0.30),  # 09:00-17:00 : 30%
        (61200, 68400, 0.20),  # 17:00-19:00 : 20%
        (68400, 86400, 0.10),  # 19:00-24:00 : 10%
    ]

    interval_trip_files = []

    for begin, end, percentage in intervals:
        interval_trips = int(num_trips * percentage)
        if interval_trips > 0:
            trip_file = create_trips_for_interval(
                net_file, instance_folder, prefix, begin, end, interval_trips, sumo_tools_path
            )
            interval_trip_files.append(trip_file)

    # Merge all interval trip files
    merged_trip_file = os.path.join(instance_folder, f"{prefix}.trips.xml")
    merge_trip_files(interval_trip_files, merged_trip_file)

    # Delete individual interval trip files
    for trip_file in interval_trip_files:
        os.remove(trip_file)

    return merged_trip_file


def find_sumo_tools_path():
    """Find the SUMO tools directory."""
    # Try common locations
    possible_paths = [
        os.path.join(os.environ.get('SUMO_HOME', ''), 'tools'),
        '/usr/share/sumo/tools',
        os.path.expanduser('~/sumo/tools'),
    ]

    for path in possible_paths:
        if os.path.isdir(path) and os.path.isfile(os.path.join(path, 'randomTrips.py')):
            return path

    # If SUMO_HOME is set, use it
    if 'SUMO_HOME' in os.environ:
        return os.path.join(os.environ['SUMO_HOME'], 'tools')

    raise RuntimeError(
        "Could not find SUMO tools directory. Please set SUMO_HOME environment variable.")


def generate_fastdta_parameters_csv(output_folder, instances):
    """Generate the fastdta_parameters_experiment.csv file."""
    csv_file = os.path.join(output_folder, "fastdta_parameters_experiment.csv")

    aggregations = [60, 300, 900]
    samples_list = [
        "(0.1, 0.9)",
        "(0.9, 0.1)",
        "(0.5, 0.5)",
        "(0.1, 0.2, 0.3, 0.4)",
        "(0.4, 0.3, 0.2, 0.1)"
    ]
    vdfs = ["bpr", "ptv"]

    convergence_deviation = 1e-6
    convergence_relgap = 1e-4
    last_iter = 150

    with open(csv_file, 'w') as f:
        # Write header
        f.write("input_dir;prefix;trip_file_name;aggregation;convergence_deviation;convergence_relgap;last_iter;seed;samples;vdf\n")

        # Write data for each instance and parameter combination
        for instance in instances:
            input_dir = instance['folder']
            prefix = instance['prefix']
            trip_file_name = f"{prefix}.trips.xml"

            for aggregation in aggregations:
                for samples in samples_list:
                    for vdf in vdfs:
                        exp_seed = random.randint(0, 2**31 - 1)
                        f.write(
                            f"{input_dir};{prefix};{trip_file_name};{aggregation};{convergence_deviation};{convergence_relgap};{last_iter};{exp_seed};{samples};{vdf}\n")

    print(f"\nGenerated {csv_file}")


def generate_runtime_experiment_csv(output_folder, instances):
    """Generate the runtime_experiment.csv file."""
    csv_file = os.path.join(output_folder, "runtime_experiment.csv")

    aggregations = [60, 300, 900]
    convergence_deviation = 1e-6
    convergence_relgap = 1e-4
    last_iter = 150

    with open(csv_file, 'w') as f:
        # Write header
        f.write("input_dir;prefix;trip_file_name;aggregation;convergence_deviation;convergence_relgap;last_iter;seed\n")

        # Write data for each instance and parameter combination
        for instance in instances:
            input_dir = instance['folder']
            prefix = instance['prefix']
            trip_file_name = f"{prefix}.trips.xml"

            for aggregation in aggregations:
                exp_seed = random.randint(0, 2**31 - 1)
                f.write(
                    f"{input_dir};{prefix};{trip_file_name};{aggregation};{convergence_deviation};{convergence_relgap};{last_iter};{exp_seed}\n")

    print(f"Generated {csv_file}")


def main():
    """Main function to generate random instances."""
    args = parse_arguments()

    # Set random seed for reproducibility
    random.seed(args.seed)

    # Create output folder if it doesn't exist
    os.makedirs(args.output_folder, exist_ok=True)

    # Find SUMO tools path
    try:
        sumo_tools_path = find_sumo_tools_path()
        print(f"Using SUMO tools from: {sumo_tools_path}")
    except RuntimeError as e:
        print(f"Error: {e}")
        sys.exit(1)

    instances = []

    # Generate each instance
    for i in range(1, args.num_instances + 1):
        print(f"\nGenerating instance {i}/{args.num_instances}")

        # Draw random parameters for this instance
        num_nodes = random.randint(args.min_nodes, args.max_nodes)
        num_trips = random.randint(args.min_trips, args.max_trips)
        instance_seed = random.randint(0, 2**31 - 1)

        # Create instance folder
        instance_folder = os.path.join(args.output_folder, f"instance_{i}")
        os.makedirs(instance_folder, exist_ok=True)

        # Set prefix
        prefix = f"{i}_n{num_nodes}_t{num_trips}"

        print(
            f"  Nodes: {num_nodes}, Trips: {num_trips}, Seed: {instance_seed}")

        # Generate network
        net_file = create_network(
            instance_folder, prefix, num_nodes, instance_seed)

        # Generate trips
        trip_file = create_trips(
            net_file, instance_folder, prefix, num_trips, sumo_tools_path)

        instances.append({
            'folder': instance_folder,
            'prefix': prefix,
            'num_nodes': num_nodes,
            'num_trips': num_trips
        })

    # Generate CSV files
    print("\nGenerating experiment CSV files...")
    generate_fastdta_parameters_csv(args.output_folder, instances)
    generate_runtime_experiment_csv(args.output_folder, instances)

    print(
        f"\n✓ Successfully generated {args.num_instances} instances in {args.output_folder}")


if __name__ == "__main__":
    main()
