SHELL := /usr/bin/env bash

.PHONY: build all-in-one run loop help clean

build:
	bash ./util_compile.sh

all-in-one:
	bash ./util_all-in-one-compile.sh

run:
	bash ./util_corn-dbio-kit.sh

loop:
	bash ./util_corn-dbio-kit-loop-exec.sh

help:
	./corn-dbio-kit --help || cargo run --manifest-path ./Cargo.toml -- --help

clean:
	rm -rf ./target ./corn-dbio-kit ./sql

