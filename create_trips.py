# python script which calls the program "randomTrips.py" to create trips

seed = 42
trips = [
    (0, 18_000, 250),
    (18_000, 21_600, 750),
    (21_600, 25_200, 1500),
    (25_200, 28_800, 2000),
    (28_800, 32_400, 2500),
    (32_400, 36_000, 2000),
]

# cli arguments: input file, output file


def createTrips(input_file, output_dir, output_prefix, trips):
    import subprocess
    import os

    # read env variable "SUMO_TOOLS"
    sumo_tools = os.environ.get("SUMO_TOOLS")
    if not sumo_tools:
        print("Error: SUMO_TOOLS environment variable is not set.")
        return

    # Ensure the output directory exists
    if not os.path.exists(output_dir):
        print(f"Creating output directory: {output_dir}")
        os.makedirs(output_dir)

    # Prepare the command to run randomTrips.py
    for start, end, num in trips:
        output_file = os.path.join(
            output_dir, f"{output_prefix}.{start}-{end}.trips.xml")
        command = ["python", os.path.join(sumo_tools, "randomTrips.py"),
                   "-n", os.path.abspath(input_file), "-o", os.path.abspath(
                       output_file), "-s", str(seed),
                   "--prefix", str(start) + "-" + str(end) + "_",
                   "--trip-attributes",
                   'departLane="best" departSpeed="max" departPos="base" arrivalPos="0.0"']

        command.extend(["-b", str(start), "-e", str(end),
                        "-p", str((end - start) // num)])

        # Execute the command
        subprocess.run(command, check=True)

    # from all the generated trip files, create a single file by merging them
    # each indivial begins with:
    # <?xml version="1.0" encoding="UTF-8"?>
    #
    # <!-- generated on 2025-07-22 17:28:45.084724 by randomTrips.py UNKNOWN
    # <configuration>
    #     <net-file value="/home/henrik/rust_road_router/import/sumo/grid/grid.net.xml"/>
    #     <output-trip-file value="/home/henrik/rust_road_router/import/sumo/grid/grid.0-18000.trips.xml"/>
    #     <prefix value="0-18000_"/>
    #     <trip-attributes value="departLane=&quot;best&quot; departSpeed=&quot;max&quot; departPos=&quot;base&quot; arrivalPos=&quot;0.0&quot;"/>
    #     <end value="18000.0"/>
    #     <period value="72.0"/>
    # </configuration>
    # -->
    # <routes xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:noNamespaceSchemaLocation="http://sumo.dlr.de/xsd/routes_file.xsd">
    # <trip .../>
    # <routes>
    # i want to have a single file which contains all the trips enclodes with <routes>

    merged_output_file = os.path.join(output_dir, f"{output_prefix}.trips.xml")
    with open(merged_output_file, 'w') as merged_file:
        merged_file.write('<?xml version="1.0" encoding="UTF-8"?>\n')
        merged_file.write(
            '<routes xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:noNamespaceSchemaLocation="http://sumo.dlr.de/xsd/routes_file.xsd">\n')

        for start, end, num in trips:
            output_file = os.path.join(
                output_dir, f"{output_prefix}.{start}-{end}.trips.xml")
            with open(output_file, 'r') as trip_file:
                # until the first trip, skip the header

                for line in trip_file:
                    if not line.strip().startswith('<trip id="'):
                        continue
                    # write the header to the merged file
                    merged_file.write(line)

        merged_file.write('</routes>\n')

    # remove the intermediate files
    for start, end, num in trips:
        output_file = os.path.join(
            output_dir, f"{output_prefix}.{start}-{end}.trips.xml")
        if os.path.exists(output_file):
            os.remove(output_file)
            print(f"Removed intermediate file: {output_file}")


if __name__ == "__main__":
    # read args:
    import sys

    # input_file and output_file are required
    if len(sys.argv) < 4:
        print("Usage: python create_trips.py <input_file> <output_dir> <output_prefix>")
        sys.exit(1)
    input_file = sys.argv[1]
    output_dir = sys.argv[2]
    output_prefix = sys.argv[3]

    # print the arguments
    print(f"Input file: {input_file}")
    print(f"Output directory: {output_dir}")
    print(f"Output prefix: {output_prefix}")

    createTrips(input_file, output_dir, output_prefix, trips)
