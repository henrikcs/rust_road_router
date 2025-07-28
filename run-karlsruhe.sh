#!/usr/bin/env sh

declare spack_env="fast-dta"
spack env activate "$spack_env"
git pull

declare release_type="release"
if [ "$1" = "--debug" ]; then
    declare release_type="debug"
fi

if [ release_type = "debug" ]; then
    ./compile.sh "$spack_env" --debug
    ./experiments-karlsruhe.sh "$spack_env" --debug
else
    ./compile.sh "$spack_env"
    ./experiments-karlsruhe.sh "$spack_env"
fi

