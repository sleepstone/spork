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

However you can add more options to it as you need them.

For example, Spork can cross compile for multiple targets:

```toml
[project]
name = "example"
kind = "executable"
targets = [
  "x86_64-windows",
  "x86-linux",
  "x86-windows",
]
```

Spork uses a [target triple](https://wiki.osdev.org/Target_Triplet) to specify the output target. It details the architecture of the CPU and the OS that your program will run on top of. A list of all supported targets can be found in [config.md](config.md#target).

Without the `targets` field, Spork will compile for your host platform. Otherwise, it will compile for the first target in the list.

If you want to compile for all targets, use:
```sh
spork build --all
```
