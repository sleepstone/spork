# spork
Spork is a simple C build system.

## Disclaimer
I created spork for my personal use, since I was fed up with having to maintain Makefiles or the complexity of CMake. It is different
from other build tools in that it doesn't generate another system's files (no Ninja, Make, Visual Studio), it only supports one
language, and only supports one compiler.

Feel free to use it for your projects, but I can't guarantee top notch support though.

## Features
- Heavily inspired by Cargo
- Cross compilation by leveraging `zig cc`
- TOML configuration format

## Dependencies
Spork requires [zig](https://ziglang.org/) to compile C files.

## Building
The only dependency required for building is Rust, which can be installed using [rustup](https://rustup.rs/).
```sh
git clone https://github.com/sleepstone/spork.git
cd spork
cargo install --path .
```

## Documentation
Documentation can be found [here](docs/main.md)
