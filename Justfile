set positional-arguments

list:
	@just --list
	
test-suite:
	cd test-suite && cargo run

check project:
	cd {{project}} && cargo check --color=always 2>&1 | less -R

levis *args:
    cd interpreter && RUST_BACKTRACE=1 cargo run --features dev -- {{args}}