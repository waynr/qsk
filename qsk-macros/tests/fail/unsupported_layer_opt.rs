use qsk_macros::layer;

fn main() {
    layer!(
        ModLayer[Meow]: {
            Y -> HOME,
            F -> TT(Navigation),
        },
        Navigation: {
            END -> Exit(),
            Y -> HOME,
            U -> PAGEDOWN,
            I -> PAGEUP,
            O -> END,
            H -> LEFT,
            J -> DOWN,
            K -> UP,
            SEMICOLON -> RIGHT,
        },
    );
}
