use qsk_macros::remap;

fn main() {
    remap!(
        ModLayer: {
            F -> TapToggle(),
        },
    );
}
