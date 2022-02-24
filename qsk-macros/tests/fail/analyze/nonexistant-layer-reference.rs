use qsk_macros::layer;

fn main() {
    layer!(
        ModLayer[Active]: {
            Y -> HOME,
            F -> TT(Meow, F),
        },
        Navigation: {
            END -> Exit(),
        },
    );
}
