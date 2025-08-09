

def convert_trips(input_file, output_file_name):
    """ converts a matsim csv file <input_file> with the headers

    to a SUMO trip file with the name <output_file_name>
    the csv should contain the following headers:
    tripId, legId, tripBeginTime, locationFrom, locationTo
    the SUMO trip file is an xml file with the following structure:
    <routes>
    <trip id="<tripId>-<legId>" depart="<convert_to_seconds_since_midnight(<tripBeginTime>)>" from="<parse_location(<locationFrom>)>" to="<parse_location(<locationTo>)>" departLane="best" departSpeed="max" departPos="base" arrivalPos="0.0"/>
    ...
    </routes>
    """

    import os
    import csv

    if not os.path.exists(input_file):
        raise FileNotFoundError(f"Input file not found: {input_file}")

    trips = []

    # read the input file
    with open(input_file, 'r') as csvfile:
        reader = csv.DictReader(csvfile, delimiter=';')
        print(f"Reading trips from {input_file}...")
        if not all(header in reader.fieldnames for header in ['tripId', 'legId', 'tripBeginTime', 'locationFrom', 'locationTo']):
            raise ValueError(
                "Input file does not contain the required headers: tripId, legId, tripBeginTime, locationFrom, locationTo but has " +
                ', '.join(reader.fieldnames))
        for row in reader:
            trip_id = row['tripId']
            leg_id = row['legId']
            trip_begin_time = row['tripBeginTime']
            location_from = row['locationFrom']
            location_to = row['locationTo']

            # convert the trip begin time to seconds since midnight
            depart_time = convert_to_seconds_since_midnight(trip_begin_time)

            # parse the locations
            from_edge = parse_location(location_from)
            to_edge = parse_location(location_to)

            trips.append((trip_id, leg_id, depart_time, from_edge, to_edge))

    # sort trips by depart_time
    trips.sort(key=lambda x: x[2])  # sort by depart_time

    # write the output file
    output_file_path = os.path.join(output_file_name)
    with open(output_file_path, 'w') as output_file:
        output_file.write('<?xml version="1.0" encoding="UTF-8"?>\n')
        output_file.write(
            '<routes xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:noNamespaceSchemaLocation="http://sumo.dlr.de/xsd/routes_file.xsd">\n')

        for trip_id, leg_id, depart_time, from_edge, to_edge in trips:
            output_file.write(
                f'  <trip id="{trip_id}-{leg_id}" depart="{depart_time}" from="{from_edge}" to="{to_edge}" departLane="best" departSpeed="max" departPos="base" arrivalPos="0.0"/>\n')

        output_file.write('</routes>\n')

    print(f"Trips converted and saved to: {output_file_path}")


def parse_location(location):
    """ Parses a location string into a SUMO compatible format.
    The given location is a string in the format: "(<lat>,<lon>: <edge_id>, <edge_start_pos>)"
    the parsed location is the edge_id
    """

    if not location:
        raise ValueError(f"Invalid location format: {location}")
    parts = location.split(':')
    if len(parts) < 2:
        raise ValueError(f"Invalid location format: {location}")
    edge_id = parts[1].strip()
    edge_id = edge_id.split(',')[0].strip()  # get only the edge_id part

    if not edge_id:
        raise ValueError(f"Invalid edge_id in location: {location}")

    return edge_id


def convert_to_seconds_since_midnight(time_str):
    """ Converts a time string in the format HH:MM:SS to seconds since midnight.
    """
    if not time_str:
        raise ValueError(f"Invalid time format: {time_str}")
    parts = time_str.split(':')
    if len(parts) != 3:
        raise ValueError(f"Invalid time format: {time_str}")
    hours, minutes, seconds = map(int, parts)
    return hours * 3600 + minutes * 60 + seconds


if __name__ == "__main__":
    # read args:
    import sys

    # input_file and output_file are required
    if len(sys.argv) < 3:
        print("Usage: python convert_matsim_trips.py <csv-input-file> <output-file-name>")
        sys.exit(1)
    input_file = sys.argv[1]
    output_file_name = sys.argv[2]

    # print the arguments
    print(f"Input file: {input_file}")
    print(f"Output file name: {output_file_name}")

    convert_trips(input_file, output_file_name)
