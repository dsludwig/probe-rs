use probe_rs_debug::{ColumnType, SourceLocation, debug_info::DebugInfo};
use std::path::PathBuf;
use typed_path::{TypedPath, UnixPathBuf};

const TEST_DATA: [(u64, u64, ColumnType); 8] = [
    // Target address, line, column
    (0x80006EA, 240, ColumnType::Column(28)),
    (0x8000764, 248, ColumnType::Column(21)),
    (0x8000856, 252, ColumnType::Column(27)),
    (0x8000958, 256, ColumnType::Column(40)),
    (0x800098E, 275, ColumnType::Column(65)),
    (0x8000A34, 292, ColumnType::Column(26)),
    (0x8000BB4, 309, ColumnType::Column(28)),
    (0x8000D6A, 408, ColumnType::Column(55)),
];

#[test]
fn breakpoint_location_absolute() {
    let di = DebugInfo::from_file("tests/probe-rs-debugger-test").unwrap();

    // Here we test with an absolute path, i.e. the combination of compilation directory
    // and relative path to the actual source file.
    let path = UnixPathBuf::from("/Users/jacknoppe/dev/probe-rs-debugger-test/src/main.rs")
        .to_typed_path_buf();

    for (addr, line, col) in TEST_DATA.iter() {
        let col = if let ColumnType::Column(c) = col {
            Some(*c)
        } else {
            None
        };

        assert_eq!(
            *addr,
            di.get_breakpoint_location(path.to_path(), *line, col)
                .expect("Failed to find breakpoint location.")
                .address,
            "Addresses do not match for data path={:?}, line={:?}, col={:?}",
            &path,
            line,
            col,
        );
    }
}

#[test]
fn breakpoint_location_inexact() {
    // test getting breakpoint location for an inexact location,
    // i.e. no exact entry exists for line 277 and column 1, but we find one for column 10.
    let test_data = [(0x80009AC, 277, ColumnType::LeftEdge)];

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("probe-rs-debugger-test");

    let di = DebugInfo::from_file(&path).unwrap();

    let path = UnixPathBuf::from("/Users/jacknoppe/dev/probe-rs-debugger-test/src/main.rs")
        .to_typed_path_buf();

    for (addr, line, col) in test_data.iter() {
        let col = if let ColumnType::Column(c) = col {
            Some(*c)
        } else {
            None
        };

        assert_eq!(
            *addr,
            di.get_breakpoint_location(path.to_path(), *line, col)
                .expect("Failed to find valid breakpoint locations.")
                .address,
            "Addresses do not match for data path={:?}, line={:?}, col={:?}",
            &path,
            line,
            col,
        );
    }
}

#[test]
fn source_location() {
    let di = DebugInfo::from_file("tests/probe-rs-debugger-test").unwrap();

    let path = UnixPathBuf::from("/Users/jacknoppe/dev/probe-rs-debugger-test/src/main.rs")
        .to_typed_path_buf();

    for (addr, line, col) in TEST_DATA.iter() {
        assert_eq!(
            Some(SourceLocation {
                line: Some(*line),
                column: Some(*col),
                path: path.clone(),
                address: Some(*addr)
            }),
            di.get_source_location(*addr)
        );
    }
}

#[test]
fn find_non_existing_unit_by_path() {
    let unit_path =
        UnixPathBuf::from("/Users/jacknoppe/dev/probe-rs-debugger-test/src/non-existent-path.rs")
            .to_typed_path_buf();

    let debug_info = DebugInfo::from_file("tests/probe-rs-debugger-test").unwrap();

    assert!(
        debug_info
            .get_breakpoint_location(unit_path.to_path(), 14, None)
            .is_err()
    );
}

/// Regression test for the C++ prologue heuristic (commit 37e5b487).
///
/// g++/clang++ omit `DW_LNS_set_prologue_end` from the line program, so before
/// the heuristic was extended to C++ every instruction was classified as
/// "prologue" and *no* valid breakpoint locations existed for any C++ function
/// -- `get_breakpoint_location` returned an error for every line. This test
/// pins that breakpoints can be placed inside representative C++ functions
/// (free function, function template, member function, namespaced code).
///
/// The fixture is built from `tests/cpp-example/` with a pinned toolchain; see
/// that directory's README for details.
#[test]
fn cpp_breakpoints_have_halt_locations() {
    let di = DebugInfo::from_file("tests/cpp-example/cpp-example.elf").unwrap();

    // comp_dir is neutralized to `/cpp-example` via -ffile-prefix-map.
    let path = UnixPathBuf::from("/cpp-example/main.cpp").to_typed_path_buf();

    // Lines inside C++ functions that must each resolve to a halt location.
    // 77: `return value;`  inside the `clamp<T>` template
    // 83: `return a + b;`  inside an overloaded free function
    // 97: `thermo.update(42);`  member-function call inside `compute`
    // 99: `int32_t total = thermo.read();`  inside `compute`
    for line in [77, 83, 97, 99] {
        let location = di
            .get_breakpoint_location(path.to_path(), line, None)
            .unwrap_or_else(|e| panic!("no breakpoint location for C++ line {line}: {e:?}"));

        // The fixture is linked at 0x08000000; a resolved halt location must
        // land in that executable range, proving a real (post-prologue)
        // instruction address was found rather than nothing.
        assert!(
            location.address >= 0x0800_0000,
            "breakpoint for line {line} resolved to unexpected address {:#x}",
            location.address,
        );
    }
}

#[test]
fn regression_pr2324() {
    let path = "C:\\_Hobby\\probe-rs-test-c-firmware/Atmel/hpl/core/hpl_init.c";

    let di = DebugInfo::from_file("tests/debug-unwind-tests/atsamd51p19a.elf").unwrap();
    let path = TypedPath::derive(path);

    let addr = di.get_breakpoint_location(path, 58, None).unwrap();

    assert_eq!(addr.address, 0x2e4);
}
