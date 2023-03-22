//! Tests for the macro

#[test]
fn schema_generation() {
    let t = trybuild::TestCases::new();
    t.pass("tests/schema/pass/*.rs");
    t.compile_fail("tests/schema/fail/*.rs");
}
