# Use an official Python runtime as a parent image
FROM docker.io/rust:1.87.0

# Set the working directory in the container
WORKDIR /app

# Copy the current directory contents into the container at /app
COPY .. /app

# Install Python, pip, remove cache, and install libsumo
RUN apt-get update && apt-get install -y libgl1 pipx python3 python3-pip && \
    rm -rf /var/lib/apt/lists/* && \
    python -m venv lib/libsumo && \
    source lib/libsumo/bin/activate && \
    python -m pip install libsumo && \
    cargo build --release && \
    cp /app/fastdta/duaIterate.py /app/lib/libsumo/lib/python3.12/site-packages/sumo/tools/assign/duaIterate.py

WORKDIR /app/data/grid/dta
ENTRYPOINT ["python3", "../../../libsumo/lib/python3.12/site-packages/sumo/tools/assign/duaIterate.py"]

# Default command to run when the container starts
CMD ["-n", "../grid.net.xml", "-t", "../grid.trips.xml", "--routing-algorithm", "CCH"]