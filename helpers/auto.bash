#!/bin/bash

cargo +nightly build

# This hacky workaround is "temporary"
while true
do
    ./target/debug/monopoly-math
    echo "simulations crashed; restarting..."
done