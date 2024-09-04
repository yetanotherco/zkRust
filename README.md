# zkRust

CLI tool to prove your rust code easily using either SP1 or Risc0.

zkRust supports generating proofs for executable scripts. Specifically, zkRust supports generating proofs for executable programs with inputs, code, and outputs known at compile time and defined within a `main()` function and `main.rs` file.

## Installation:

First make sure [Rust](https://www.rust-lang.org/tools/install) is installed on your machine. Then install the zkVM toolchains from [risc0](https://github.com/risc0/risc0) and [sp1](https://github.com/succinctlabs/sp1) by running:

```sh
make install
```

zkRust can then be installed directly by downloading the latest release binaries.

```sh
curl -L https://raw.githubusercontent.com/yetanotherco/zkRust/main/install_zkrust.sh | bash
```

## Quickstart

You can test zkRust for any of the examples in the `examples` folder. This include:

- a Fibonacci program
- a RSA program
- an ECDSA program
- a simple blockchain state diff program

Run one of the following commands to test zkRust. You can choose either risc0 or sp1:

**Fibonacci**:

```bash
make prove_risc0_fibonacci
```

```bash
make prove_sp1_fibonacci
```

**RSA**:

```bash
make prove_risc0_rsa
```

```bash
make prove_sp1_rsa
```

**ECDSA**:

```bash
make prove_risc0_ecdsa
```

```bash
make prove_sp1_ecdsa
```

**Blockchain state diff**:

```bash
make prove_risc0_blockchain_state
```

```bash
make prove_sp1_blockchain_state
```

## Usage:

To use zkRust, define the code you would like to generate a proof for in a `main.rs` in a directory with the following structure:

```
.
└── <PROGRAM_DIRECTORY>
    ├── Cargo.toml
    └── src
        └── main.rs
```

For using more complex workspaces it is recommended to define it within a separate module/directory.

```
.
└── <PROGRAM_DIRECTORY>
    ├── Cargo.toml
    └── src
        ├── main.rs
        └── lib
            └── ...
```

To generate a proof of the execution of your code run the following:

- **Sp1**:
  ```sh
  cargo run --release -- prove-sp1 <PROGRAM_DIRECTORY_PATH> .
  ```
- **Risc0**:
  ```sh
  cargo run --release -- prove-risc0  <PROGRAM_DIRECTORY_PATH> .
  ```
  Make sure to have [Risc0](https://dev.risczero.com/api/zkvm/quickstart#1-install-the-risc-zero-toolchain) installed with version `v1.0.1`

To generate your proof and send it to [Aligned Layer](https://github.com/yetanotherco/aligned_layer). First generate a local wallet keystore using `[cast](https://book.getfoundry.sh/cast/).

```sh
cast wallet new-mnemonic
```

Then you can import your created keystore using:

```sh
cast wallet import --interactive <PATH_TO_KEYSTORE.json>
```

Finally, to generate and send your proof of your programs execution to aligned use the zkRust CLI with the `--submit-to-aligned-with-keystore` flag.

```sh
cargo run --release -- prove-sp1 --submit-to-aligned-with-keystore <PATH_TO_KEYSTORE> <PROGRAM_DIRECTORY_PATH .
```

### Flags

- `--precompiles`: Enables in acceleration via precompiles for supported zkVM's. Specifying this flag allows for VM specific speedups for specific expensive operations such as SHA256, SHA3, bigint multiplication, and ed25519 signature verification. By specifying this flag proving operations for specific operations within the following rust crates are accelerated:

  - SP1:
    - `sha2`
    - `sha3`
    - `crypto-bigint`
    - `tiny-keccak`
    - `ed25519-consensus`
    - `ecdsa-core`
    - `secp256k1`
  - Risc0:
    - `sha2`
    - `k256`
    - `crypto-bigint`

- `--io`: The I/O flag enables the user to specify input and output code for the executed in the zkVM. To use the feature the user defines a input, output, and main function within respective files. The input function executes before the zkVM code is executed and allows the user to define informatoin passed into the vm such as deserializing Tx or fetching information from an external source. The main function executes within the vm itself and defines the commputation performed within the vm and reads in the inputs defined in the input function and specifies what is outputted. The output function reads data from the main function in the vm and represents post processing of that information.

To define the structure do the following:

```
.
└── <PROGRAM_DIRECTORY>
    ├── Cargo.toml
    └── src
        ├── output.rs
        ├── input.rs
        └── main.rs
```

The user may specify inputs into the VM (guest) code using `zk_rust_io::write()` as long on the type of rust object they are writing implements `Serializable`. Within there VM code (guest) the user may read in the inputs by specifying `zk_rust_io::read()` and output data computed during the execution phase of the code within the VM (guest) program by specifying `zk_rust_io::commit()`. To read the output of the output of the VM (guest) program you declare `zk_rust_io::out()`. The `zk_rust_io` crate defines function headers that are not inlined and are purely used as compile time symbols to ensure a user can compile there rust code before running it on the zkVM.

### input.rs

```rust
use zk_rust_io;

pub fn input() {
    let pattern = "a+".to_string();
    let target_string = "an era of truth, not trust".to_string();

    // Write in a simple regex pattern.
    zk_rust_io::write(&pattern);
    zk_rust_io::write(&target_string);
}
```

### main.rs

```rust
use regex::Regex;
use zk_rust_io;

pub fn main() {
    // Read two inputs from the prover: a regex pattern and a target string.
    let pattern: String = zk_rust_io::read();
    let target_string: String = zk_rust_io::read();

    // Try to compile the regex pattern. If it fails, write `false` as output and return.
    let regex = match Regex::new(&pattern) {
        Ok(regex) => regex,
        Err(_) => {
            panic!("Invalid regex pattern");
        }
    };

    // Perform the regex search on the target string.
    let result = regex.is_match(&target_string);

    // Write the result (true or false) to the output.
    zk_rust_io::commit(&result);
}
```

### output.rs

```rust
use zk_rust_io;

pub fn output() {
    // Read the output.
    let res: bool = zk_rust_io::out();
    println!("res: {}", res);
}
```

# Acknowledgments

[SP1](https://github.com/succinctlabs/sp1.git)

[Risc0](https://github.com/risc0/risc0.git)
