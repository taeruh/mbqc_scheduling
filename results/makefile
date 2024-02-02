MAKEFLAGS += --no-builtin-rules
MAKEFLAGS += --no-builtin-variables
SHELL := /usr/bin/dash

srs := rsync -avh --info=progress2 -e "ssh -t -c aes128-ctr -o compression=no -x"
remote := uts_hpc:s/a/phd/mbqc_scheduling

send:
	$(srs) -R scripts/exe_hpc.bash parameters Cargo.toml .cargo/config.toml src $(remote)

receive:
	$(srs) $(remote)/output/* output
