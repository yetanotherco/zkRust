# zkRust

`zkRust` is a CLI tool to simplify the developing applications in Rust using zkVM's such as SP1 or Risc0.

zkRust seeks to simplify the development experience of developing using zkVM's and enable developers by providing the choice of which zkVM they would like to develop with and eliminating redundancy in the Zk Application development process.

## Installation:

First make sure [Rust](https://www.rust-lang.org/tools/install) is installed on your machine. Then install the zkVM toolchains from [sp1](https://github.com/succinctlabs/sp1) and [risc0](https://github.com/risc0/risc0) by running:

```sh
curl -L https://sp1.succinct.xyz | bash
sp1up
cargo prove --version
```

and

```sh
curl -L https://risczero.com/install | bash
rzup install
cargo risczero --version
```

zkRust can also be installed directly by downloading the latest release binaries.

```sh
curl -L https://raw.githubusercontent.com/yetanotherco/zkRust/main/install_zkrust.sh | bash
```

## Quickstart

To get started you can create a workspace for your project in zkRust by running:

```sh
cargo new <PROGRAM_DIRECTORY>
```

It's that simple.

You can test zkRust for any of the examples in the `examples` folder. This include programs for:

- Computing and reading the results of computing Fibonacci numbers.
- Performing RSA key verification.
- Performing ECDSA program.
- Verification of a blockchain state diff.
- Computing the Sha256 hash of a value.
- Verifying a tendermint block.
- Interacting with a user to answer a quiz.

## Usage:

To use zkRust, users must specify a `main()` function whose execution is proven within the zkVM. This function must be defined within a `main.rs` file in a directory with the following structure:

```
.
└── <PROGRAM_DIRECTORY>
    ├── Cargo.toml
    └── src
        └── main.rs
```

Projects can also store libraries in a separate `lib/` folder.

```
.
└── <PROGRAM_DIRECTORY>
    ├── Cargo.toml
    ├── lib/
    └── src
        └── main.rs
```

The user may also define `input()`, `output()` functions, in addition to `main()`. The `fn input()` and `fn output()` functions define code that runs outside of the zkVM before and after the VM executes. The `input()` function executes before the zkVM code is executed and allows the user to define inputs passed to the vm such as a deserialized Tx or data fetched from an external source at runtime. Within the `main()` (guest) function the user may write information from the computation performed in the zkVM to an output buffer to be used after proof generation. The `output()` function defines code that allows the user to read the information written to that buffer and perform post-processing of that data.

![](./assets/zkRust_execution_flow.png)

The user may specify inputs into the VM (guest) code using `zk_rust_io::write()` as long on the type of rust object they are writing implements `Serializable`. Within there `main()` function (guest) the user may read in the inputs by specifying `zk_rust_io::read()` and output data computed during the execution phase of the code within the VM (guest) program by specifying `zk_rust_io::commit()`. To read the output of the output of the VM (guest) program you declare `zk_rust_io::out()`. The `zk_rust_io` crate defines function headers that are not inlined and are purely used as compile time symbols to ensure a user can compile there rust code before running it within one of the zkVM available in zkRust.

To use the I/O imports import the `zk_rust_io` crate by adding the following to the `Cargo.toml` in your project directory.

```sh
zk_rust_io = { git = "https://github.com/yetanotherco/zkRust.git", version = "v0.1.0" }
```

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

//NOTE ADD ANNOTATION ABOUT WRITTEN AND READING FROM BUFFER IN ONE call -> Otherwise issues.

To generate a proof of the execution of your code run the following:

- **SP1**:
  ```sh
  cargo run --release -- prove-sp1 <PROGRAM_DIRECTORY_PATH> .
  ```
- **Risc0**:
  ```sh
  cargo run --release -- prove-risc0  <PROGRAM_DIRECTORY_PATH> .
  ```
  Make sure to have [Risc0](https://dev.risczero.com/api/zkvm/quickstart#1-install-the-risc-zero-toolchain) installed with version `v1.0.1`

To generate your proof and send it to [Aligned](https://github.com/yetanotherco/aligned_layer). First generate a local wallet keystore using `[cast](https://book.getfoundry.sh/cast/).

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

- `--submit_to_aligned`: Sends the proof to be verified on Aligned after proof generation. Requires an rpc url and keystore for a funded wallet specified via the `--rpc-url` and `--key_store` flags.

- `--keystore_path`: Path to the keystore of the users wallet. Defaults to `~/keystore`.

- `--rpc-url`: Specifies the rpc-url used for the user eth rpc-url. Defaults to `https://ethereum-holesky-rpc.publicnode.com`.

- `--max-fee`: Specifies the max fee the user is willing to pay for there proof to be included in a batch. Defaults to `0.01 Eth`.

- `--chain_id`: Chain ID number of the ethereum chain Aligned is deployed on. Defaults to `1700`.

- `--precompiles`: Enables acceleration via precompiles for supported zkVM's. Specifying this flag allows for VM specific speedups for specific expensive operations such as SHA256, SHA3, bigint multiplication, and ed25519 signature verification. By specifying this flag proving operations for specific operations within the following rust crates are accelerated:

  - SP1:

    - sha2 v0.10.6
    - sha3 v0.10.8
    - crypto-bigint v0.5.5
    - tiny-keccak v2.0.2
    - ed25519-consensus v2.1.0
    - ecdsa-core v0.16.9

  - Risc0:
    - sha2 v0.10.6
    - k256 v0.13.1
    - crypto-bigint v0.5.5

Run one of the following commands to test zkRust. You can choose either Risc0 or SP1:

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
make prove_risc0_json
```

```bash
make prove_sp1_json
```

**Blockchain state diff**:

```bash
make prove_risc0_json
```

```bash
make prove_sp1_json
```

**Regex**:

```bash
make prove_risc0_regex
```

```bash
make prove_sp1_regex
```

**Sha**:

```bash
make prove_risc0_sha
```

```bash
make prove_sp1_sha
```

**Tendermint**:

```bash
make prove_risc0_tendermint
```

```bash
make prove_sp1_tendermint
```

**Zk Quiz**:

```bash
make prove_risc0_zkquiz
```

```bash
make prove_sp1_zkquiz
```

**NOTE**: for the precompiles to be included within the compilation step the crate version you are using must match the crate version above.

**NOTE**: Aligned currently supports Risc0 proofs from `risc0-zkvm` version `v1.0.1`. For generating proofs using `cargo risc-zero` please ensure you are using `v1.0.1` or your proof will not be verified. If you encounter issues installing an older version of `cargo-risc0` please reference this [thread](https://discord.com/channels/953703904086994974/1290498126049841232).

# Acknowledgments:

ZK Rust was intended and designed as a tool to make development on programs that use zkVM's easier and reduce deduplication of code for developers that want to experiment with zk on aligned layer. We want the work and contributions of the SP1 and Risc0 teams to the field of Zero Knowledge Cryptography.

[SP1](https://github.com/succinctlabs/sp1.git)

[Risc0](https://github.com/risc0/risc0.git)
