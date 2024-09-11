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

## Usage:

To use zkRust, define the code you would like to generate a proof for in a `fn main()` in a `main.rs` directory with the following structure:

```
.
└── <PROGRAM_DIRECTORY>
    ├── Cargo.toml
    ├── lib/
    └── src
        └── main.rs
```

For using more complex workspaces write and import a separate lib/ crate into the .

```
.
└── <PROGRAM_DIRECTORY>
    ├── Cargo.toml
    ├── lib/
    └── src
        ├── main.rs
        └── lib
            └── ...
```

To use zkRust users specify a `main()` whose execution is proven within the zkVM. The user may also define a `input()`, `output()`, in addition to the `main()` function which defines code that runs outside of the zkVM before and after the VM executes. The `input()` function executes before the zkVM code is executed and allows the user to define inputs passed to the vm such as a deserialized Tx or data fetched from an external source at runtime. Within the `main()` function the user may define information that will to written to a output buffer after proof generation. The `output()` then allows the user to read the information written to that buffer and perform post-processing of that data.

The user may specify inputs into the VM (guest) code using `zk_rust_io::write()` as long on the type of rust object they are writing implements `Serializable`. Within there `main()` function (guest) the user may read in the inputs by specifying `zk_rust_io::read()` and output data computed during the execution phase of the code within the VM (guest) program by specifying `zk_rust_io::commit()`. To read the output of the output of the VM (guest) program you declare `zk_rust_io::out()`. The `zk_rust_io` crate defines function headers that are not inlined and are purely used as compile time symbols to ensure a user can compile there rust code before running it within one of the zkVM available in zkRust.

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

- `--submit_to_aligned`: Sends the proof to be verified on Aligned after proof generation. Requires an rpc url and keystore for a funded wallet specified via the `--rpc-url` and `--key_store` flags.

- `--keystore_path`: Path to the keystore of the users wallet. Defaults to `~/keystore`.

- `--rpc-url`: Specify the rpc-url used for the user eth rpc-url. Defaults to `https://ethereum-holesky-rpc.publicnode.com`.

- `--chain_id`: Chain ID number of the ethereum chain Aligned is deployed on. Defaults to `1700`.

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

# Acknowledgments:

ZK Rust was intioned and designed as a tool to make development on programs that use zkVM's easier and reduce deduplication of code for developers that want to experiment with zk on aligned layer. We want the work and contributions of the SP1 and Risc0 teams to the field of Zero Knowledge Cryptography.

[SP1](https://github.com/succinctlabs/sp1.git)

[Risc0](https://github.com/risc0/risc0.git)
