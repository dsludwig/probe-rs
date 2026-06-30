use probe_rs::MemoryInterface;

use crate::{
    DebugError, Variable, VariableCache, VariableName, VariableType, VariableValue,
    function_die::FunctionDie,
    language::{ProgrammingLanguage, c::C},
};

/// C++.
///
/// C++ shares the bulk of its value- and type-handling with [`C`] (base types,
/// bitfields, pointer/array formatting), so those operations are delegated.
/// The C++-specific behaviour layered on top is demangling of function names.
#[derive(Debug, Clone)]
pub struct Cpp;

impl ProgrammingLanguage for Cpp {
    fn read_variable_value(
        &self,
        variable: &Variable,
        memory: &mut dyn MemoryInterface,
        variable_cache: &VariableCache,
    ) -> VariableValue {
        C.read_variable_value(variable, memory, variable_cache)
    }

    fn update_variable(
        &self,
        variable: &Variable,
        memory: &mut dyn MemoryInterface,
        new_value: &str,
    ) -> Result<(), DebugError> {
        C.update_variable(variable, memory, new_value)
    }

    fn format_enum_value(&self, type_name: &VariableType, value: &VariableName) -> VariableValue {
        C.format_enum_value(type_name, value)
    }

    fn format_array_type(&self, item_type: &str, length: usize) -> String {
        C.format_array_type(item_type, length)
    }

    fn format_pointer_type(&self, pointee: Option<&str>) -> String {
        C.format_pointer_type(pointee)
    }

    fn process_tag_with_no_type(&self, variable: &Variable, tag: gimli::DwTag) -> VariableValue {
        C.process_tag_with_no_type(variable, tag)
    }

    fn format_function_name(
        &self,
        function_name: &str,
        function_die: &FunctionDie<'_>,
        debug_info: &crate::DebugInfo,
    ) -> String {
        // `DW_AT_name` carries only the unqualified function name (e.g. `read`),
        // without its namespace/class qualification or overload signature. When
        // a mangled linkage name is present, demangle it to recover the
        // fully-qualified signature (e.g. `sensors::Thermometer::read() const`),
        // which is what a debugger user expects to see in a backtrace.
        //
        // `extern "C"` functions and similar have no linkage name to demangle,
        // in which case we fall back to the plain `DW_AT_name`.
        if let Some(linkage_name) = linkage_name(function_die, debug_info)
            && let Some(demangled) = addr2line::demangle(&linkage_name, gimli::DW_LANG_C_plus_plus)
        {
            return demangled;
        }

        function_name.to_string()
    }
}

/// Read the `DW_AT_linkage_name` (mangled symbol) of a function DIE, if present.
fn linkage_name(function_die: &FunctionDie<'_>, debug_info: &crate::DebugInfo) -> Option<String> {
    let attribute = function_die.attribute(debug_info, gimli::DW_AT_linkage_name)?;
    let gimli::AttributeValue::DebugStrRef(name_ref) = attribute.value() else {
        return None;
    };
    let raw = debug_info.dwarf.string(name_ref).ok()?;
    Some(String::from_utf8_lossy(&raw).to_string())
}
