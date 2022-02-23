use qsk_macros::layer;

fn main() {
    layer!(
        ModLayer: {
            F -> Exit(MEOW),
        },
    );
}
