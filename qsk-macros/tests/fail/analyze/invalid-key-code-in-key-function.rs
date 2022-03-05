use qsk_macros::remap;

fn main() {
    remap!(
        ModLayer: {
            F -> TT(Navigation, MEOW),
        },
    );
}
