use qsk_macros::layer;

fn main() {
    layer!(
        ModLayer: {
            F -> TT(Navigation, MEOW),
        },
    );
}
