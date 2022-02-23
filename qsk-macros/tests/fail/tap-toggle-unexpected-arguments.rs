use qsk_macros::layer;

fn main() {
    layer!(
        ModLayer[Meow]: {
            F -> TapToggle(Navigation, F, MEOW),
        },
    );
}
