#!/usr/bin/env sh

git pull
declare spack_env="fast-dta"

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

