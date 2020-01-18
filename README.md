# `Rocinante`

## Install

1. Install [Z3](https://github.com/Z3Prover/z3) theorem prover.

   - Ubuntu: `sudo apt-get install libz3-dev z3`
   - macOS: `brew install z3`

2. Clone this repo.
3. Run `cargo test` to verify and setup pre-commit hooks via
   [cargo-husky](https://github.com/rhysd/cargo-husky).
4. Now you're good to go.

## Usage

```shell
cargo run --bin print <FILE>
```

1. Reads `.wat` or `.wasm` file into binary format.
2. Deserializes binary into an IR.
3. Prints each function.

## Running`.wasm`/`.wat` files

### wasmtime

- Install [wasmtime](https://github.com/bytecodealliance/wasmtime)
- Open a new terminal.

- Running a WebAssembly module with a start function:

```shell
wasmtime example.wasm
```

- Passing command line arguments to a WebAssembly module:

```shell
wasmtime example.wasm arg1 arg2 arg3
```

- Invoking a specific function (e.g. `add`) in a WebAssembly module:

```shell
wasmtime example.wasm --invoke add 1 2
```

### Reference interpreter

- Install [Ocaml](https://ocaml.org/)
- Install reference
  [interpreter](https://github.com/WebAssembly/spec/tree/master/interpreter)

```shell
wasm example.wasm
```

## Additional Resources

@taegyunkim
[notes](https://www.notion.so/taegyunk/Superoptimizer-for-WebAssembly-5650ec352a9840a3b8f38af6fa75595d)
