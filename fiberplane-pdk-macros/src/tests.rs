//! Tests for the macro

#[test]
fn schema_generation() {
    let t = trybuild::TestCases::new();
    t.pass("tests/schema/multiple_strings.rs");
    t.compile_fail("tests/schema/array_field.rs");
}
