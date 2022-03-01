use qsk_macros;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    qsk_macros::layer!(
        ModLayer[Active]: {
            Y -> HOME,
            F -> TT(Navigation, F),
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
    )?;
    Ok(())
}
