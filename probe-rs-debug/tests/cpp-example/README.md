# C++ test fixture

A small freestanding C++ program used as a DWARF fixture for the
`probe-rs-debug` test suite. It is **never executed** — only its debug info is
parsed — so it has no real startup code, linker script, or runtime.

`main.cpp` deliberately exercises the C++ constructs the debugger must
understand: namespaces, classes, single inheritance, virtual methods,
scoped (`enum class`) and unscoped enums, a function template instantiated
with two types, reference parameters, and overloaded free functions.

## Rebuilding

The checked-in `cpp-example.elf` is committed so tests run without a C++
toolchain in CI. **The DWARF layout — and therefore the addresses/paths the
tests assert — depends on the exact compiler version.** Rebuild only with the
pinned toolchain, then re-run (and if necessary re-review) the affected tests.

- Pinned toolchain: **Arm GNU Toolchain `arm-none-eabi-g++` 14.3.Rel1** (14.3.1)
- Target: thumbv7em (Cortex-M4), DWARF 4, `-O0 -g3`

```sh
make            # produces cpp-example.elf
make clean
```

`-ffile-prefix-map` rewrites the compilation directory to `/cpp-example`, so
the recorded source path is `/cpp-example/main.cpp` regardless of where it was
built. Tests reference that stable path.

## Consumed by

- `tests/source_location.rs` — breakpoint / prologue coverage
- (planned) `src/debug_info.rs` variable-rendering tests via `MockMemory`
