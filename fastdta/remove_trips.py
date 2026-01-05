#!/usr/bin/env python3
"""
Remove trips from a SUMO trips XML file by their indexes.

This script reads a SUMO trips XML file, removes trips at specified indexes
(0-indexed), and writes the result to a new file. The remaining trips keep
their original IDs.
"""

import sys
import os
import xml.etree.ElementTree as ET
from typing import List, Set


def remove_trips(input_file: str, output_file: str, indexes_to_remove: List[int]) -> None:
    """
    Remove trips from a SUMO trips XML file by their indexes.

    Args:
        input_file: Path to the input trips XML file
        output_file: Name of the output file (will be saved in same directory as input)
        indexes_to_remove: List of 0-indexed positions of trips to remove
    """
    if not indexes_to_remove:
        print("Warning: No indexes provided, output will be identical to input")

    # Convert to set for O(1) lookup and remove duplicates
    indexes_set: Set[int] = set(indexes_to_remove)

    # Check for negative indexes
    if any(idx < 0 for idx in indexes_set):
        raise ValueError("Indexes must be non-negative")

    # Parse the XML file
    tree = ET.parse(input_file)
    root = tree.getroot()

    # Find all trip elements
    original_trips = root.findall('trip')
    total_trips = len(original_trips)

    print(f"Found {total_trips} trips in input file")
    print(
        f"Removing trips at {len(indexes_set)} index(es): {sorted(indexes_set)}")

    # Check if any index is out of bounds
    max_index = max(indexes_set) if indexes_set else -1
    if max_index >= total_trips:
        raise ValueError(
            f"Index {max_index} is out of bounds (file has {total_trips} trips, valid indexes: 0-{total_trips-1})")

    # Create list of trips to keep
    trips_to_keep = []
    removed_count = 0

    for idx, trip in enumerate(original_trips):
        if idx in indexes_set:
            removed_count += 1
        else:
            trips_to_keep.append(trip)

    print(f"Removed {removed_count} trip(s)")
    print(f"Keeping {len(trips_to_keep)} trip(s)")

    # Clear all trips from root
    for trip in original_trips:
        root.remove(trip)

    # Add back only the trips we want to keep
    for trip in trips_to_keep:
        root.append(trip)

    # Determine output path
    input_dir = os.path.dirname(input_file)
    output_path = os.path.join(input_dir, output_file)

    # Write the modified tree to the output file
    tree.write(output_path, encoding='UTF-8', xml_declaration=True)

    print(f"Successfully wrote {len(trips_to_keep)} trips to {output_path}")


def parse_indexes(indexes_str: str) -> List[int]:
    """
    Parse a comma-separated string of indexes into a list of integers.

    Args:
        indexes_str: Comma-separated string of indexes (e.g., "0,5,10")

    Returns:
        List of integer indexes
    """
    try:
        indexes = [int(idx.strip()) for idx in indexes_str.split(',')]
        return indexes
    except ValueError as e:
        raise ValueError(f"Invalid index format: {e}")


def main():
    """Main entry point for the script."""
    if len(sys.argv) != 4:
        print("Usage: python remove_trips.py <input_file> <output_file> <indexes>")
        print("  input_file: Path to the input trips XML file")
        print("  output_file: Name of the output file (saved in same directory as input)")
        print("  indexes: Comma-separated list of 0-indexed trip positions to remove")
        print("\nExample: python remove_trips.py trips.xml output.xml 0,5,10")
        sys.exit(1)

    input_file = sys.argv[1]
    output_file = sys.argv[2]
    indexes_str = sys.argv[3]

    if not os.path.exists(input_file):
        print(f"Error: Input file '{input_file}' does not exist")
        sys.exit(1)

    try:
        indexes = parse_indexes(indexes_str)
        remove_trips(input_file, output_file, indexes)
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
