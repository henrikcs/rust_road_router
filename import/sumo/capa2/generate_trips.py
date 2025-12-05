#!/usr/bin/env python3
"""
Generate trips between random corner edges with configurable frequency.

Usage: python3 generate_trips.py <period> [output_file]
    period: Floating point number representing seconds between trips
    output_file: Optional output filename (default: capa2.trips.xml)

Example: python3 generate_trips.py 0.59
"""

import sys
import random
import xml.etree.ElementTree as ET
from xml.dom import minidom

# Define corner edges for the 3x3 grid
# Edges going away from corners (for trip origins)
from_corner_edges = [
    # Bottom-left corner (A0)
    "A0A1", "A0B0",
    # Bottom-right corner (C0)
    "C0C1", "C0B0",
    # Top-left corner (A2)
    "A2A1", "A2B2",
    # Top-right corner (C2)
    "C2B2", "C2C1"
]

# Edges going towards corners (for trip destinations)
to_corner_edges = [
    # Bottom-left corner (A0)
    "A1A0", "B0A0",
    # Bottom-right corner (C0)
    "C1C0", "B0C0",
    # Top-left corner (A2)
    "A1A2", "B2A2",
    # Top-right corner (C2)
    "B2C2", "C1C2"
]


def generate_trips(period, start_time=0, end_time=21600, output_file="capa2.trips.xml"):
    """
    Generate trips from random corner edges at regular intervals.

    Args:
        period: Time in seconds between consecutive trips
        start_time: Start time for trip generation
        end_time: End time for trip generation
        output_file: Output XML filename
    """
    # Create root element with proper namespace
    root = ET.Element('routes')
    root.set('xmlns:xsi', 'http://www.w3.org/2001/XMLSchema-instance')
    root.set('xsi:noNamespaceSchemaLocation',
             'http://sumo.dlr.de/xsd/routes_file.xsd')

    # Generate trips
    trip_id = 0
    current_time = start_time

    while current_time <= end_time:
        trip = ET.SubElement(root, 'trip')
        trip.set('id', str(trip_id))
        trip.set('depart', f"{current_time:.2f}")
        trip.set('from', random.choice(from_corner_edges))
        trip.set('to', random.choice(to_corner_edges))
        trip.set('departSpeed', 'max')
        trip.set('departPos', 'base')
        trip.set('arrivalPos', 'max')

        trip_id += 1
        current_time += period

    # Create XML tree
    tree = ET.ElementTree(root)

    # Pretty print XML
    xml_string = ET.tostring(root, encoding='unicode')
    dom = minidom.parseString(xml_string)
    pretty_xml = dom.toprettyxml(indent="    ")

    # Remove extra blank lines and write to file
    lines = [line for line in pretty_xml.split('\n') if line.strip()]
    with open(output_file, 'w', encoding='UTF-8') as f:
        f.write('\n'.join(lines) + '\n')

    print(f"Generated {trip_id} trips with period {period}s")
    print(f"Time range: {start_time}s to {current_time - period:.2f}s")
    print(f"Output file: {output_file}")
    print(f"From edges (away from corners): {len(from_corner_edges)}")
    print(f"To edges (towards corners): {len(to_corner_edges)}")


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Error: Period argument required")
        print(__doc__)
        sys.exit(1)

    try:
        period = float(sys.argv[1])
        if period <= 0:
            print("Error: Period must be positive")
            sys.exit(1)
    except ValueError:
        print(f"Error: Invalid period '{sys.argv[1]}' - must be a number")
        sys.exit(1)

    output_file = sys.argv[2] if len(sys.argv) > 2 else "capa2.trips.xml"

    generate_trips(period, output_file=output_file)
