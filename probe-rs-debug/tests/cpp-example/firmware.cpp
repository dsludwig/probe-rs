// C++ firmware fixture for an EFM32PG1B100F256GM32 (Pearl Gecko Series 1,
// Cortex-M4). Unlike main.cpp (a generic DWARF-only fixture that is never run),
// this program is flashed and executed on real hardware: it builds a deep C++
// call stack and then spins, so that halting the core and capturing a coredump
// yields a multi-frame C++ backtrace with live local variables.
//
// It deliberately exercises constructs the debugger does NOT fully support yet
// (virtual dispatch / vtable pointers, reference parameters, polymorphism
// through a base pointer, template-typed locals) in addition to the supported
// ones, so the captured `*_cpp_unwind` snapshot documents exactly where we fall
// short. See README.md for how to build/flash/capture.

#include <stdint.h>

// Minimal freestanding new/delete for the virtual classes' deleting dtors.
void* operator new(unsigned int size) noexcept { (void)size; return 0; }
void operator delete(void* ptr) noexcept { (void)ptr; }
void operator delete(void* ptr, unsigned int size) noexcept { (void)ptr; (void)size; }

namespace sensors {

enum class Unit : uint8_t { Celsius = 0, Fahrenheit = 1, Kelvin = 2 };
enum Status { STATUS_OK = 0, STATUS_WARN = 1, STATUS_ERROR = 2 };

class Sensor {
public:
    explicit Sensor(int32_t id) : id_(id) {}
    virtual ~Sensor() {}
    virtual int32_t read() const { return id_; }
    int32_t id() const { return id_; }

protected:
    int32_t id_;
};

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

template <typename T>
T clamp(T value, T lo, T hi) {
    if (value < lo) return lo;
    if (value > hi) return hi;
    return value;
}

// Reference parameter: exercises DW_TAG_reference_type with a live value.
__attribute__((noinline)) void accumulate(int32_t& acc, int32_t delta) {
    acc += delta;
}

// Sink to keep stores live so locals are not optimized away even at -O0.
volatile int32_t g_sink;

// Deepest frame: virtual dispatch through a base pointer, a reference param,
// and a template-typed local, then spin forever so a halt lands here.
__attribute__((noinline)) int32_t leaf_spin(sensors::Sensor* sensor, int32_t& acc) {
    int32_t reading = sensor->read(); // virtual call via vtable
    accumulate(acc, reading);
    int32_t clamped = clamp<int32_t>(acc, 0, 100); // template instantiation
    g_sink = clamped;
    // Halt here with the full C++ stack live (leaf_spin <- mid <- Reset_Handler).
    // BKPT halts the core into debug state so a coredump can be captured at a
    // deterministic point, no timing/continue race required.
    __asm__ volatile("bkpt #0");
    for (;;) {
        g_sink = clamped;
    }
}

__attribute__((noinline)) int32_t mid(sensors::Thermometer& thermo, int32_t seed) {
    sensors::Status status = (seed > 0) ? sensors::STATUS_OK : sensors::STATUS_ERROR;
    int32_t acc = seed + static_cast<int32_t>(status);
    return leaf_spin(&thermo, acc); // pass derived as base*, acc by reference
}

extern "C" void Reset_Handler();

// --- Minimal Cortex-M startup ------------------------------------------------

extern uint32_t _sidata; // .data init image in flash
extern uint32_t _sdata;
extern uint32_t _edata;
extern uint32_t _sbss;
extern uint32_t _ebss;
extern uint32_t _estack;

static void init_memory() {
    uint32_t* src = &_sidata;
    for (uint32_t* dst = &_sdata; dst < &_edata;) {
        *dst++ = *src++;
    }
    for (uint32_t* dst = &_sbss; dst < &_ebss;) {
        *dst++ = 0;
    }
}

extern "C" void Reset_Handler() {
    init_memory();

    sensors::Thermometer thermo(7, sensors::Unit::Celsius); // vtable object on stack
    thermo.update(42);

    g_sink = mid(thermo, thermo.id());

    for (;;) {
    }
}

// Vector table: only the two entries needed to boot (initial SP + reset).
typedef void (*vector_t)();
__attribute__((section(".vectors"), used)) const vector_t vector_table[] = {
    reinterpret_cast<vector_t>(&_estack),
    Reset_Handler,
};
