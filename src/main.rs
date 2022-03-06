use std::error;

use qsk_macros;

use qsk::entrypoint;

fn main() -> Result<(), Box<dyn error::Error>> {
    let lc = qsk_macros::remap!(
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

    entrypoint(lc)?;

    Ok(())
}
