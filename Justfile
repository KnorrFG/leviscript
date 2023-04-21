set positional-arguments

check project:
	cd "$1" && cargo check --color=always 2>&1 | less -R
