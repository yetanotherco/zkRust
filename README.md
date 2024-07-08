# zkRust

CLI tool to prove your rust code easily using either SP1 or Risc0.

zkRust supports generating proofs for executable scripts. Specifically, zkRust supports generating proofs for executable programs with inputs, code, and outputs known at compile time and defined within a `main()` function and `main.rs` file.

## Usage:

To use zkRust, define the code you would like to generate a proof for in a `main.rs` in a directory with the following structure:

```
.
└── <PROGRAM_DIRECTORY>
    ├── Cargo.toml
    └── src
        └── main.rs

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

## Limitations:
Currently zkRust does not support fully support the following:

- Programs with a library structure.
- VM user Input and Output
- VM Precompiles

These are features are planned to be added in later editions.

# Acknowledgments

[SP1](https://github.com/succinctlabs/sp1.git)

[Risc0](https://github.com/risc0/risc0.git)
