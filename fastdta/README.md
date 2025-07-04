# Fast DTA POC

This is a proof of concept project for the master's thesis on _Fast Dynamic Traffic Assignment using Engineered Shortest-Path Speedup Techniques_ by Henrik Csöre.

## Using Podman

Assuming you have installed Podman on your machine, run the following command in the root directory of this project to build an image:

```bash
podman build -t fast-dta:latest .
```

Then, run a benchmark with

### Windows:

using Powershell:

```bash
podman run --rm -a=stdout -a=stderr -v ${PWD}\out:/app/out fast-dta:latest
```

### Linux/Mac:

```bash
podman run --rm -a=stdout -a=stderr -v ${PWD}/out:/app/out fast-dta:latest
```

This command will print the console outputs of the benchmark, move the output into the current working directory's `/out` directory and remove the container once it's finished.

## Run Locally (Ubuntu)

### Prerequisites

- Install Python
- Create a venv
- Activate the venv
- Install libsumo
- Inject a modified duaIterate.py file into the libsumo directory

```bash
sudo apt install python
python -m venv lib/libsumo
source lib/libsumo/bin/activate
python -m pip install libsumo
cp fastdta/duaIterate.py lib/libsumo/lib/python3.12/site-packages/sumo/tools/assign/duaIterate.py
```
