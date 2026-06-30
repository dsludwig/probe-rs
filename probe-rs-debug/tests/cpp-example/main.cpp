// C++ test fixture for probe-rs-debug.
//
// This is a freestanding (bare-metal) program that is never executed -- it
// exists only so its DWARF debug info can be parsed by the test suite. It is
// deliberately written to exercise the C++ language constructs that the
// debugger needs to understand: namespaces, classes, inheritance, virtual
// methods, scoped/unscoped enums, templates, references and overloads.
//
// See README.md for how to (re)build the checked-in ELF. The program is
// compiled at -O0 -g so that every construct survives into the debug info and
// each source line maps to a breakpoint location.

#include <stdint.h>

// Minimal freestanding new/delete so the compiler-emitted deleting destructors
// for the virtual classes below have something to link against. Never called.
void* operator new(unsigned int size) { (void)size; return 0; }
void operator delete(void* ptr) noexcept { (void)ptr; }
void operator delete(void* ptr, unsigned int size) noexcept { (void)ptr; (void)size; }

namespace sensors {

enum class Unit : uint8_t {
    Celsius = 0,
    Fahrenheit = 1,
    Kelvin = 2,
};

enum Status {
    STATUS_OK = 0,
    STATUS_WARN = 1,
    STATUS_ERROR = 2,
};

// A base class with a virtual method, so the DWARF carries a vtable pointer.
class Sensor {
public:
    explicit Sensor(int32_t id) : id_(id) {}
    virtual ~Sensor() {}

    virtual int32_t read() const { return id_; }

    int32_t id() const { return id_; }

protected:
    int32_t id_;
};

// A derived class adding members, to exercise DW_TAG_inheritance.
class Thermometer : public Sensor {
public:
    Thermometer(int32_t id, Unit unit) : Sensor(id), unit_(unit), last_(0) {}

    int32_t read() const override { return last_; }

    void update(int32_t raw) { last_ = raw; }

    Unit unit() const { return unit_; }

private:
    Unit unit_;
    int32_t last_;
};

} // namespace sensors

// A function template instantiated with two distinct types below, so the line
// program contains multiple address ranges for the same source line.
template <typename T>
T clamp(T value, T lo, T hi) {
    if (value < lo) {
        return lo;
    }
    if (value > hi) {
        return hi;
    }
    return value;
}

// Overloaded free functions sharing a name -- exercises demangling of
// distinct mangled symbols.
int32_t combine(int32_t a, int32_t b) {
    return a + b;
}

int32_t combine(int32_t a, int32_t b, int32_t c) {
    return a + b + c;
}

// Takes a reference argument, exercising DW_TAG_reference_type.
void accumulate(int32_t& acc, int32_t delta) {
    acc += delta;
}

extern "C" int compute() {
    sensors::Thermometer thermo(7, sensors::Unit::Celsius);
    thermo.update(42);

    int32_t total = thermo.read();
    accumulate(total, thermo.id());

    int32_t clamped_i = clamp<int32_t>(total, 0, 100);
    uint8_t clamped_u = clamp<uint8_t>(static_cast<uint8_t>(clamped_i), 0, 50);

    int32_t a = combine(clamped_i, clamped_u);
    int32_t b = combine(a, clamped_i, clamped_u);

    sensors::Status status = (b > 0) ? sensors::STATUS_OK : sensors::STATUS_ERROR;

    return b + static_cast<int32_t>(status);
}

// Minimal reset entry point. Never actually executed by the tests.
extern "C" void _start() {
    volatile int result = compute();
    (void)result;
    for (;;) {
    }
}
