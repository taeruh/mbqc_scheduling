MAKEFLAGS += --no-builtin-rules
MAKEFLAGS += --no-builtin-variables
SHELL := /usr/bin/dash

srs := rsync -avh --info=progress2 -e "ssh -t -c aes128-ctr -o compression=no -x"
remote := uts_hpc:s/a/phd/mbqc_scheduling

# note that we include the lock file and on the cluster we will not run cargo update
send:
	$(srs) -R scripts/*_hpc.bash parameters Cargo.toml Cargo.lock .cargo/config.toml src $(remote)

receive:
	$(srs) $(remote)/output/* output
