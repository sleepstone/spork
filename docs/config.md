# Configuration format of Spork.toml

## `[project]`
This table details information about your project.

### `name` (String)
The name of your project. Used as the name of your output file(s).

### `kind` (String)
What type of Spork project that will be built. Valid values are `"executable"` and `"library"`.

### `target` (String[])
A list of **target specifiers**. Spork will cross compile the project once for each target.

A target specifier uses the format `ARCHITECTURE-OS`.

Valid architectures are:
- `x86_64` - 64-bit Intel x86 processor compatible
- `x86` - 32-bit Intel x86 processor compatible

Valid operating systems are:
- `freestanding` - No dependencies on any OS - Useful for writing kernels, drivers, etc.
- `windows` - Windows NT compatible (.exe / .dll)
- `linux` - Linux compatible
