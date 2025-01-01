
# show list of available commands
default: help

[private]
help:
    @just --list

inject process:
    cargo run --package game_capture_dll --bin inject 