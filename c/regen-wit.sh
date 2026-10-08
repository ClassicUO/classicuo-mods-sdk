#!/bin/sh
# Regenerates generated/cuo_wit.{c,h} + cuo_wit_component_type.o from ../wit (world
# cuo:modding/mod) with wit-bindgen-cli 0.57 (`cargo install wit-bindgen-cli --version ^0.57`,
# the family the Rust SDK uses). Run after any change under ../wit.
set -e
cd "$(dirname "$0")"
rm -rf generated
wit-bindgen c ../wit --world cuo:modding/mod --rename-world cuo_wit --out-dir generated
