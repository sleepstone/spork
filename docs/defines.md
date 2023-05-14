# Spork #defines

To make conditional compilation in C code easier, Spork defines a number of useful object-like macros.

## General

### SPORK_DEBUG
If --release is not passed, then this value will be defined.

### SPORK_EXPORT
If this header is built as part of a library, then this value will be defined. 

## Platform

### SPORK_OS_FREESTANDING
If the target OS is `freestanding`, then this value will be defined.

### SPORK_OS_WINDOWS
If the target OS is `windows`, then this value will be defined.

### SPORK_OS_LINUX
If the target OS is `linux`, then this value will be defined.
