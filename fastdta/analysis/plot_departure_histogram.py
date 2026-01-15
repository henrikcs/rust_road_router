#!/usr/bin/env python3
"""
Plot departure time histogram from SUMO trips XML file.

This script reads a SUMO trips XML file, aggregates trip departures into time bins,
and creates a histogram showing the number of vehicles departing in each time interval.
"""

import sys
import os
import argparse
import xml.etree.ElementTree as ET
from pathlib import Path
import matplotlib.pyplot as plt
import numpy as np


def read_departure_times(trips_file):
    """
    Read departure times from a SUMO trips XML file.
    
    Args:
        trips_file: Path to the trips XML file
        
    Returns:
        List of departure times (in seconds)
    """
    tree = ET.parse(trips_file)
    root = tree.getroot()
    
    departure_times = []
    for trip in root.findall('trip'):
        depart = float(trip.get('depart'))
        departure_times.append(depart)
    
    return departure_times


def aggregate_departures(departure_times, bin_size):
    """
    Aggregate departure times into bins of specified size.
    
    Args:
        departure_times: List of departure times in seconds
        bin_size: Size of time bins in seconds
        
    Returns:
        Tuple of (bin_starts, counts) where bin_starts are the start times
        of each bin and counts are the number of trips in each bin
    """
    if not departure_times:
        return [], []
    
    min_time = 0
    max_time = max(departure_times)
    
    # Create bins from 0 to max_time (rounded up to next bin boundary)
    num_bins = int(np.ceil(max_time / bin_size)) + 1
    bins = [i * bin_size for i in range(num_bins)]
    
    # Count trips in each bin
    counts = [0] * (num_bins - 1)
    
    for depart in departure_times:
        bin_idx = int(depart / bin_size)
        if bin_idx < len(counts):
            counts[bin_idx] += 1
    
    # Return bin start times and counts
    bin_starts = bins[:-1]
    
    return bin_starts, counts


def plot_departure_histogram(bin_starts, counts, output_path):
    """
    Create and save a departure histogram plot.
    
    Args:
        bin_starts: List of bin start times
        counts: List of trip counts for each bin
        output_path: Path where to save the plot (PDF)
    """
    # Set figure size appropriate for 3 plots horizontally on A4
    # A4 landscape: 297mm x 210mm
    # Each plot: ~99mm wide (~3.9 inches), height ~70mm (~2.76 inches)
    fig, ax = plt.subplots(figsize=(3.9, 2.76))
    
    # Create bar plot
    if bin_starts and counts:
        bin_width = bin_starts[1] - bin_starts[0] if len(bin_starts) > 1 else 1
        ax.bar(bin_starts, counts, width=bin_width, align='edge', 
               edgecolor='black', linewidth=0.5)
    
    # Set labels with readable font size
    ax.set_xlabel('Time (s)', fontsize=10)
    ax.set_ylabel('Number of Vehicles', fontsize=10)
    
    # Adjust tick label size
    ax.tick_params(axis='both', which='major', labelsize=9)
    
    # Add grid for better readability
    ax.grid(True, alpha=0.3, linewidth=0.5)
    
    # Tight layout to maximize space usage
    plt.tight_layout()
    
    # Save as PDF
    plt.savefig(output_path, format='pdf', bbox_inches='tight')
    plt.close()
    
    print(f"Plot saved to: {output_path}")


def main():
    parser = argparse.ArgumentParser(
        description='Plot departure time histogram from SUMO trips XML file'
    )
    parser.add_argument('trips_file', type=str,
                        help='Path to SUMO trips XML file')
    parser.add_argument('n', type=int,
                        help='Time bin size in seconds (e.g., 60 for 1-minute bins)')
    parser.add_argument('out_dir', type=str,
                        help='Output directory for the plot')
    parser.add_argument('out_file', type=str,
                        help='Output filename (should end with .pdf)')
    
    args = parser.parse_args()
    
    # Validate inputs
    if not os.path.exists(args.trips_file):
        print(f"Error: Trips file '{args.trips_file}' not found", file=sys.stderr)
        sys.exit(1)
    
    if args.n <= 0:
        print(f"Error: Time bin size must be positive, got {args.n}", file=sys.stderr)
        sys.exit(1)
    
    # Create output directory if needed
    output_dir = Path(args.out_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Construct output path
    output_path = output_dir / args.out_file
    
    # Read departure times
    print(f"Reading trips from: {args.trips_file}")
    departure_times = read_departure_times(args.trips_file)
    print(f"Found {len(departure_times)} trips")
    
    if departure_times:
        print(f"Time range: {min(departure_times):.2f}s to {max(departure_times):.2f}s")
    
    # Aggregate into bins
    print(f"Aggregating with bin size: {args.n}s")
    bin_starts, counts = aggregate_departures(departure_times, args.n)
    print(f"Created {len(counts)} bins")
    
    if counts:
        print(f"Max vehicles in a bin: {max(counts)}")
        print(f"Total vehicles: {sum(counts)}")
    
    # Create and save plot
    plot_departure_histogram(bin_starts, counts, output_path)
    print("Done!")


if __name__ == "__main__":
    main()
