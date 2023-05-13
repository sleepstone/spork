# Getting started
Starting a new C project is as easy as:
```sh
spork init
```
This will initialize a Spork project (and Git repository) in the current directory with the following structure:
```
<root>
  - src
    - main.c
  - .clang-format
  - .gitignore
  - Spork.toml
```
Alternatively, you can create a library project like this:
```sh
spork init --lib
```
Library projects contain an additional `include` folder (for public headers) and are always built as shared libraries (.dll on Windows, .so on Mac/Linux).

Compiling and running your executable project is done like this:
```sh
$ spork run
Hello, world!
```

Nice! You can run an optimized build too:
```sh
$ spork run --release
Hello, world!
```

Note that Spork will automatically find all the C files in `src` and compile them.

## Spork.toml
99% of all your project/build configuration is handled in `Spork.toml`. The initial file starts out very minimal:
```toml
[project]
name = "example"
kind = "executable"
```

**TODO: add more tutorial**
