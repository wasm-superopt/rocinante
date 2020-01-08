# Rocinante

[Notes](https://www.notion.so/taegyunk/Superoptimizer-for-WebAssembly-5650ec352a9840a3b8f38af6fa75595d)

## Usage
```shell
cargo run -- <FILE>
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
