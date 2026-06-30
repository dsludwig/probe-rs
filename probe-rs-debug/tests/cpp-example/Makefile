# Builds the checked-in C++ test fixture ELF.
#
# Toolchain is PINNED: the DWARF layout (and therefore the addresses asserted
# in the tests) depends on the exact compiler version. Rebuild only with the
# version recorded in README.md, then re-review the affected snapshots/tests.
#
#   make        # build cpp-example.elf
#   make clean

CXX      := arm-none-eabi-g++
TARGET   := cpp-example.elf
SRC      := main.cpp

# thumbv7em (Cortex-M4), bare-metal, no startup files / libc.
# -O0 -g3 keeps every construct in the debug info and gives one line -> one
# breakpoint location. -fno-omit-frame-pointer keeps unwinding simple.
# -ffile-prefix-map rewrites the recorded compilation directory to a stable,
# machine-independent value so the checked-in ELF (and the paths asserted in
# tests) don't embed whoever built it absolute home directory.
CXXFLAGS := -mcpu=cortex-m4 -mthumb \
            -O0 -g3 -gdwarf-4 \
            -ffreestanding -fno-exceptions -fno-rtti \
            -fno-omit-frame-pointer \
            -ffile-prefix-map=$(CURDIR)=/cpp-example \
            -nostdlib -nostartfiles

# Place .text at a flash-like address so the ELF looks like real firmware.
LDFLAGS  := -Wl,-Ttext=0x08000000 -Wl,-e,_start

$(TARGET): $(SRC) Makefile
	$(CXX) $(CXXFLAGS) $(LDFLAGS) -o $@ $(SRC)

.PHONY: clean
clean:
	rm -f $(TARGET)
