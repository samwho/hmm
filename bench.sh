#!/usr/bin/env bash

# Make sure we're operating from the same directory that the script lives.
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
cd $DIR

function setup() {
    # Do a release build to make sure the binaries are all there.
    cargo build --release

    # Generate 10 years worth of data for an extremely active user who does one
    # entry every minute.
    rm /tmp/out
    target/release/hmmdg --path /tmp/out --num-days 3650 --entries-per-day 1440
}

setup

# Gain root permissions so we can drop_caches.
sudo -v 

hyperfine \
  --export-markdown bench.md \
  --min-runs 100 \
  --prepare 'sync; echo 3 | sudo tee /proc/sys/vm/drop_caches' \
  'target/release/hmmq --path /tmp/out --random' \
  'target/release/hmmq --path /tmp/out --last 10' \
  'target/release/hmmq --path /tmp/out --first 10' \
  'target/release/hmmq --path /tmp/out --start 2019 --first 10' \
  'target/release/hmmq --path /tmp/out --end 2019 --last 10' \
  'target/release/hmmq --path /tmp/out --start 2019-01 --end 2019-02' \
  'target/release/hmmq --path /tmp/out --start 2019 --end 2020 --count' \
  'target/release/hmmq --path /tmp/out --start 2019-01 --end 2019-06 --contains lorum' \
  'target/release/hmmq --path /tmp/out --start 2019 --end 2020 --regex "(lorum|ipsum)"' \
