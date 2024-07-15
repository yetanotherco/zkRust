install: install_risc0 install_sp1

install_risc0:
	@curl -L https://risczero.com/install | bash
	@rzup
	@cargo risczero --version
	@echo "Risc0 Toolchain Installed"

install_sp1:
	@curl -L https://sp1.succinct.xyz | bash
	@sp1up
	@cargo prove --version
	@echo "Sp1 Toolchain Installed"

# Default target
all: instal

__EXAMPLES__:

# RISC0
prove_risc0_fibonacci:
	@cargo run --release -- prove-risc0 examples/fibonacci .

prove_risc0_rsa:
	@cargo run --release -- prove-risc0 examples/rsa .

prove_risc0_ecdsa:
	@cargo run --release -- prove-risc0 examples/ecdsa .

prove_risc0_blockchain_state:
	@cargo run --release -- prove-risc0 examples/json .

# SP1
prove_sp1_fibonacci:
	@cargo run --release -- prove-sp1 examples/fibonacci .

prove_sp1_rsa:
	@cargo run --release -- prove-sp1 examples/rsa .

prove_sp1_ecdsa:
	@cargo run --release -- prove-sp1 examples/ecdsa .

prove_sp1_blockchain_state:
	@cargo run --release -- prove-sp1 examples/json .
