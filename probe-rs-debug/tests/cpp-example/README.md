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

## Hardware coredump fixture (`firmware.cpp`)

`firmware.cpp` is a *runnable* EFM32PG1B100F256GM32 image (proper Cortex-M
vector table, startup, and memory map — see `efm32pg1.ld`). Unlike `main.cpp`,
it is flashed and executed: it builds a deep C++ call stack
(`Reset_Handler -> mid -> leaf_spin`) and hits a `BKPT`, so a captured coredump
yields a multi-frame C++ backtrace with live locals. It drives the
`efm32pg1b_cpp_unwind` case of the `full_unwind` test in `src/debug_info.rs`.

Build it with `make firmware`.

### Capturing the coredump

Flash + run + dump via the probe-rs `debug` REPL (which requires a TTY, so it is
driven through a PTY, e.g. with `expect`):

1. `probe-rs debug --chip EFM32PG1B100F256GM32 --connect-under-reset --launch firmware.elf`
   (EFM32 Series 1 *requires* connect-under-reset — see the `EFM32xG1` sequence).
2. `c` — continue from the reset halt; the firmware runs into the `BKPT` and halts.
3. `dump 0x20007fb8 0x48 <path>` — only the live stack `[SP, top-of-RAM)` at the
   BKPT is needed (the unwinder reads code/CFI from the ELF, not the coredump),
   which is why the checked-in `.coredump` is only ~650 bytes.

The fixture is toolchain/layout sensitive: if you rebuild `firmware.elf`, the
stack pointer / addresses change, so re-derive the dump range from a fresh
backtrace and re-review the `efm32pg1b_cpp_unwind` snapshot.

### Known C++ shortcomings documented by the snapshot

The `efm32pg1b_cpp_unwind` snapshot intentionally captures current limitations:

- **A mid-stack frame's function name is unresolved** (`<unknown function @
  0x…>`) even though its locals and source line resolve — a return-address →
  function-name lookup gap (likely not C++-specific).
- **No polymorphism / vtable awareness** — a `Thermometer` accessed through a
  `Sensor*` renders the (now flattened) `Sensor` base members but not the
  dynamic derived fields, and the vtable pointer is not shown.

Inheritance (base-class members flattened into the derived object), references
(rendered as `T&`), and `bool` (`true`/`false`) are now handled and visible in
the snapshot.

## Consumed by

- `tests/source_location.rs` — breakpoint / prologue coverage
- `src/debug_info.rs` — `cpp_function_names_are_demangled` (demangling) and
  `cpp_static_variables` (class/enum/base-type rendering via `MockMemory`)
- `src/debug_info.rs` — `full_unwind` case `efm32pg1b_cpp_unwind` (hardware
  coredump: multi-frame C++ unwind + live locals)
