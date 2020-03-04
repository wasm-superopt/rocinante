# `Rocinante`

## Install

1. Install rust nightly.
2. Install [Z3](https://github.com/Z3Prover/z3) theorem prover.

   - Ubuntu: `sudo apt-get install libz3-dev z3`
   - macOS: `brew install z3`

3. Clone this repo.
4. Run `cargo test` to verify and setup pre-commit hooks via
   [cargo-husky](https://github.com/rhysd/cargo-husky).
5. Now you're good to go.

## Usage

```shell
$> cargo run -- help

USAGE:
    rocinante [FLAGS] [OPTIONS] <FILE> [SUBCOMMAND]

FLAGS:
    -e               Turn off optimization counting values on the stack
    -h, --help       Prints help information
    -s               Run synthesis step only and skip optimization step.
    -V, --version    Prints version information

OPTIONS:
    -a <algorithm>                    Superoptimization algorithm to use. [default: Stoke]  [possible values: Random,
                                      Stoke]
    -b <compute_budget_in_min>        The max runtime of one synthesis or optimization step in minutes [default: 3]
    -i <interpreter>                  Which interpreter to use for evaluating test cases. [default: Wasmer]  [possible
                                      values: Wasmer, Wasmtime]

ARGS:
    <FILE>    .wasm/.wat/.wast file to optimize

SUBCOMMANDS:
    help     Prints this message or the help of the given subcommand(s)
    print    Prints all functions in the given module.
```

```shell
$> cargo run -- ./examples/times-two/add.wat Stoke
```

```shell
cargo run -- <FILE> print
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
