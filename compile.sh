declare spack_env=$1
# if a flag "--debug" is provided, then set release_type to "debug"
# otherwise if no flag is provided then set to "release"

declare release_type="release"
if [ "$2" = "--debug" ]; then
    declare release_type="debug"
fi

declare pwd=$(pwd)

P=$(basename $(find ~/.user_spack/environments/"$spack_env"/.spack-env/._view -mindepth 1 -maxdepth 1 -type d))
export PATH="$pwd"/lib/InertialFlowCutter/build:"$pwd"/target/"$release_type":~/rust-nightly/bin:~/.local/bin:$PATH
export RL=$(spack location -i readline%gcc@14)
export NC=$(spack location -i ncurses%gcc@14)
export LX=$(spack location -i libx11%gcc@14)
export LIBRARY_PATH=$LX/lib:$RL/lib:$NC/lib:$LIBRARY_PATH
export CPATH=$RL/include:$NC/include:$LX/include
export LD_LIBRARY_PATH=~/.local/libnsl1/lib64:~/.user_spack/environments/"$spack_env"/.spack-env/._view/"$P"/lib:$NC/lib:$LD_LIBRARY_PATH

if [ "$release_type" = "debug" ]; then
    cargo build
else
    cargo build --release
fi