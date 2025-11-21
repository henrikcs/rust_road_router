#!/usr/bin/env python3
"""
Multiply trips in a SUMO trips XML file by an integer factor k.

This script reads a SUMO trips XML file, multiplies each trip by k times,
and writes the result to a new file. The trip IDs are renumbered sequentially
based on the order sorted by departure time (which is assumed to already be
sorted in the input file).
"""

import sys
import os
import xml.etree.ElementTree as ET
from typing import List, Tuple


def multiply_trips(input_file: str, output_file: str, k: int) -> None:
    """
    Multiply trips in a SUMO trips XML file by factor k.

    Args:
        input_file: Path to the input trips XML file
        output_file: Name of the output file (will be saved in same directory as input)
        k: Multiplication factor (number of copies per trip)
    """
    if k <= 0:
        raise ValueError(f"Multiplication factor k must be positive, got {k}")

    # Parse the XML file
    tree = ET.parse(input_file)
    root = tree.getroot()

    # Find all trip elements
    original_trips = root.findall('trip')

    print(f"Found {len(original_trips)} original trips")
    print(f"Multiplying by {k} to create {len(original_trips) * k} trips")

    # Create new trips list
    new_trips = []

    # For each original trip, create k copies
    for trip in original_trips:
        for copy_idx in range(k):
            # Create a copy of the trip element
            new_trip = ET.Element('trip')

            # Copy all attributes except 'id' (we'll set it later)
            for attr_name, attr_value in trip.attrib.items():
                if attr_name != 'id':
                    new_trip.set(attr_name, attr_value)

            new_trips.append(new_trip)

    # Renumber all trips sequentially (1-indexed)
    for idx, trip in enumerate(new_trips, start=1):
        trip.set('id', str(idx))

    # Clear the original trips from the root
    for trip in original_trips:
        root.remove(trip)

    # Add the new trips to the root
    for trip in new_trips:
        root.append(trip)

    # Determine output path
    input_dir = os.path.dirname(input_file)
    output_path = os.path.join(input_dir, output_file)

    # Write the modified tree to the output file
    tree.write(output_path, encoding='UTF-8', xml_declaration=True)

    print(f"Successfully wrote {len(new_trips)} trips to {output_path}")


def main():
    """Main entry point for the script."""
    if len(sys.argv) != 4:
        print("Usage: python multiply_trips.py <input_file> <output_file> <k>")
        print("  input_file: Path to the input trips XML file")
        print("  output_file: Name of the output file (saved in same directory as input)")
        print("  k: Multiplication factor (integer)")
        sys.exit(1)

    input_file = sys.argv[1]
    output_file = sys.argv[2]

    try:
        k = int(sys.argv[3])
    except ValueError:
        print(f"Error: k must be an integer, got '{sys.argv[3]}'")
        sys.exit(1)

    if not os.path.exists(input_file):
        print(f"Error: Input file '{input_file}' does not exist")
        sys.exit(1)

    try:
        multiply_trips(input_file, output_file, k)
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
