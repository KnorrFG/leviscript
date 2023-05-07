set positional-arguments

list:
	@just --list
	
test-suite:
	cd test-suite && cargo run

check project:
	cd {{project}} && cargo check --color=always 2>&1 | less -R

levis *args:
	cd interpreter && cargo build --features dev
	RUST_BACKTRACE=1 interpreter/target/debug/levis {{args}}

debug *args:
	#! /usr/bin/env bash
	set -eu
	cd interpreter && cargo build --features dev && cd ..
	if [[ -e gdbscript ]]; then dbg_script_args="-x gdbscript"; else dbg_script_args=""; fi
	rust-gdb $dbg_script_args --args interpreter/target/debug/levis "$@"

doc-lib *bonus_args:
	cd leviscript-lib && cargo doc --document-private-items {{bonus_args}}