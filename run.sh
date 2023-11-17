#!/bin/sh
# nix develop -c nvidia-offload cargo run --release 2>/tmp/colonqlog.txt
nix develop -c cargo run --release
