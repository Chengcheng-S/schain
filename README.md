# Schain
Do everything that you think is right

This is a chain based on substrate, and then some on-chain modules are added according to my own ideas

## Getting Started

```shell
sudo apt install --assume-yes git clang curl libssl-dev protobuf-compiler
```

rust toolchain
```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

rustup default stable
rustup update

rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
```

### Build && Help

```shell
make help
```
Use the following command to build the node without launching it:

```sh
make build
```
### Embedded Docs

After you build the project, you can use the following command to explore its parameters and subcommands:

```sh
./build/node -h
```

You can generate and view the [Rust Docs](https://doc.rust-lang.org/cargo/commands/cargo-doc.html) for this template with this command:

```sh
cargo +nightly doc --open
```
