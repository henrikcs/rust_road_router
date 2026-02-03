# Fast DTA POC

This is a proof of concept project for the master's thesis on _Fast Dynamic Traffic Assignment using Engineered Shortest-Path Speedup Techniques_ by Henrik Csöre.

## Installation Instructions for Running Fast DTA Experiments

This section describes the dependencies and environment setup required to run fastdta experiments on a Linux server.

### 1. Spack Environment

Install [Spack](https://spack.io/) and create an environment named `fast-dta` with the following packages:

```bash
spack env create fast-dta
spack env activate fast-dta
```

**Required packages (gcc@14.2.0 compiler):**

```bash
spack install gcc@14
spack install cmake
spack install intel-tbb
spack install libx11
spack install libxext
spack install libxrender
spack install mesa
spack install ncurses
spack install readline
spack install pipx
spack install python@3.11.9
```

The full list of packages in the environment includes:

- **Core:** `gcc@14.2.0`, `cmake@3.30.5`, `intel-tbb@2021.12.0`
- **X11/Graphics:** `libx11@1.8.10`, `libxext@1.3.6`, `libxrender@0.9.11`, `mesa@23.3.6`
- **Terminal/IO:** `ncurses@6.5`, `readline@8.2`
- **Python:** `python@3.11.9`, `pipx@1.2.0`, and related pip packages

### 2. Rust Installation

Install Rust nightly version **1.89.0-nightly**:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup install nightly-2025-01-01  # or the specific date for 1.89.0-nightly
rustup default nightly-2025-01-01
```

Alternatively, if using a custom Rust installation directory:

```bash
# Example: ~/rust-nightly/bin should be in PATH
export PATH=~/rust-nightly/bin:$PATH
```

### 3. Python and SUMO Installation

Create a Python virtual environment and install SUMO (libsumo) version **1.23.1**:

```bash
python -m venv venvs/libsumo
source venvs/libsumo/bin/activate
pip install libsumo==1.23.1
```

Copy the modified `duaIterate.py` script into the SUMO tools directory:

```bash
cp fastdta/duaIterate.py venvs/libsumo/lib/python3.11/site-packages/sumo/tools/assign/duaIterate.py
```

### 4. Build InertialFlowCutter

The project depends on InertialFlowCutter. Build it from source:

```bash
cd lib/InertialFlowCutter
mkdir -p build && cd build
cmake ..
make -j$(nproc)
```

### 5. Environment Variables

The following environment variables must be configured. Adjust paths according to your installation:

```bash
# Activate spack environment
spack env activate fast-dta

# Set project root directory
export PROJECT_ROOT=$(pwd)

# Get spack view path
P=$(basename $(find ~/.user_spack/environments/fast-dta/.spack-env/._view -mindepth 1 -maxdepth 1 -type d))

# Library locations from spack
export RL=$(spack location -i readline%gcc@14)
export NC=$(spack location -i ncurses%gcc@14)
export LX=$(spack location -i libx11%gcc@14)

# PATH configuration
export PATH="$PROJECT_ROOT/lib/InertialFlowCutter/build:$PROJECT_ROOT/target/release:~/rust-nightly/bin:~/.local/bin:$PATH"

# Library paths
export LIBRARY_PATH=$LX/lib:$RL/lib:$NC/lib:$LIBRARY_PATH
export CPATH=$RL/include:$NC/include:$LX/include:$CPATH
export LD_LIBRARY_PATH=~/.local/libnsl1/lib64:~/.user_spack/environments/fast-dta/.spack-env/._view/"$P"/lib:$NC/lib:$LD_LIBRARY_PATH

# SUMO configuration
export SUMO_HOME=$PROJECT_ROOT/venvs/libsumo/lib/python3.11/site-packages/sumo
export PATH=$SUMO_HOME/bin:$PATH
```

### 6. Build the Project

```bash
# For release build
cargo build --release

# For debug build (enables RUST_BACKTRACE=1)
cargo build
```

Optional build features:

```bash
# Disable parallel queries
cargo build --release --features "queries-disable-par"

# Enable SUMO node expansion
cargo build --release --features "expand-sumo-nodes"
```

### 7. Running Experiments

Use the `run_experiments.sh` script to run experiments:

```bash
./run_experiments.sh \
    --spack-env fast-dta \
    --experiment <experiment_file> \
    --fast-dta2 \
    --output ./results/[TIME]
```

See `./run_experiments.sh --help` for all available options and routing algorithms.

### 8. Test Instances

The thesis experiments use the **Leopoldshafen** and **Rastatt** SUMO network instances. These are provided as compressed archives in `import/sumo-zipped/`:

```bash
# Extract the test instances to the import/sumo directory
cd import/sumo-zipped
unzip leopoldshafen.zip -d ../sumo/
unzip rastatt.zip -d ../sumo/
```

After extraction, the instances will be available at:

- `import/sumo/leopoldshafen/`
- `import/sumo/rastatt/`

---

## Run Locally (Ubuntu) - Quick Start

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
python -m pip install libsumo==1.23.1
cp fastdta/duaIterate.py lib/libsumo/lib/python3.12/site-packages/sumo/tools/assign/duaIterate.py
```
