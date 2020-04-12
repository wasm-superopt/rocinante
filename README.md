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
rocinante 0.1.0
WebAssembly Superoptimizer

USAGE:
    rocinante [FLAGS] [OPTIONS] <FILE> <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -o, --no-opti    If set, run synthesis step only and skip optimization step, true by default.
    -V, --version    Prints version information

OPTIONS:
    -c, --constants <constants>...
            A comma separated list of integers for initial set of constants. [default: -2,-1,0,1,2]

    -i, --interpreter-kind <interpreter-kind>
            Which interpreter to use for evaluating test cases. [default: Wasmer]  [possible values: Wasmer, Wasmtime]

    -t, --time-budget <time-budget>
            The max runtime of one synthesis or optimization step in minutes. [default: 5]


ARGS:
    <FILE>

SUBCOMMANDS:
    enumerative
    help           Prints this message or the help of the given subcommand(s)
    stoke          Stochastic search specific options.
```

```shell
$> cargo run -- stoke --help
USAGE:
    rocinante <FILE> stoke [FLAGS] [OPTIONS]

FLAGS:
    -e, --no-enforce-stack-check
    -h, --help                      Prints help information
    -V, --version                   Prints version information

OPTIONS:
    -b, --beta <beta>           [default: 0.2]
    -s, --sampler <sampler>    The sampler algorithm to use [default: MCMC]  [possible values: Random, MCMC]
```

```shell
$> cargo run -- enumerative --help
USAGE:
    rocinante <FILE> enumerative

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
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
