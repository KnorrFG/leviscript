set positional-arguments

list:
	@just --list
	
test-suite:
	cd test-suite && cargo run

check project:
	cd "$1" && cargo check --color=always 2>&1 | less -R
