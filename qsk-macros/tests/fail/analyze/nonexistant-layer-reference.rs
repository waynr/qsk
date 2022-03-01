use qsk_macros::remap;

fn main() {
    remap!(
        ModLayer[Active]: {
            Y -> HOME,
            F -> TT(Meow, F),
        },
        Navigation: {
            END -> Exit(),
        },
    );
}
