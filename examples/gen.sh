#!/bin/bash

for toml in ./*.toml; do
    json="${toml%.toml}.json"
    echo convert ${toml} into ${json}
    cat ${toml}| tomlq > ${json}
done
