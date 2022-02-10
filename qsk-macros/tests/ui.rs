use trybuild;
use qsk_macros::layer;

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail/*.rs");
    t.pass("tests/pass/*.rs");
}
