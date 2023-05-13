# Project Configuration

## `[project]`
This table details information about your project.

### `name`
The name of your project. Used as the name of your output file(s).

### `kind`
What type of Spork project that will be built. Valid values are `"executable"` and `"library"`.

### `target`
Optional.
A list of **target specifiers** (as strings). Spork will cross compile the project once for each target.

A target specifier uses the format `ARCHITECTURE-OS`.

Valid architectures are:
- `x86_64` - 64-bit Intel x86 processor compatible
- `x86` - 32-bit Intel x86 processor compatible

Valid operating systems are:
- `freestanding` - No dependencies on any OS - Useful for writing kernels, drivers, etc.
- `windows` - Windows NT compatible (.exe / .dll)
- `linux` - Linux compatible

# Workspace Configuration

## `[workspace]`
This table details all of the members of your workspace.
A member can be added like this:

```toml
[workspace]
example = "remote link to git repository..."
```

You can add multiple members by adding more keys to `[workspace]`.

```toml
[workspace]
foo = "..."
bar = "..."
baz = "..."
```
