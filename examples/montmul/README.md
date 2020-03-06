This is the montgomery multiplication benchmark from Stochastic 
Superoptimization (ASPLOS' 13). 

Compile .cc to .wasm
- Install [emsdk](https://github.com/emscripten-core/emsdk)
- `emcc -o mont_mul.wasm mont_mul.cc  -s DEMANGLE_SUPPORT=1 -s MODULARIZE=1 -s
  WASM=1 -Os`

To extract only the `mont_mul()` function
- Install [binaryen](https://github.com/WebAssembly/binaryen)
- wasm-dis mont_mul.wasm > mont_mul.wat

Find the function that is exported as 'mont_mul'
