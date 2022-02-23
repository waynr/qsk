use qsk_macros::layer;

fn main() {
    layer!(
        ModLayer: {
            F -> TapToggle(Navigation, F, MEOW),
        },
    );
}
