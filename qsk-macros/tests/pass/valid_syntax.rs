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
        TestGT32KeyMaps: {
            A -> B,
            B -> C,
            C -> D,
            D -> E,
            E -> F,
            F -> G,
            G -> H,
            H -> I,
            I -> J,
            J -> K,
            K -> L,
            L -> M,
            M -> N,
            N -> O,
            O -> P,
            P -> Q,
            Q -> R,
            R -> S,
            S -> T,
            T -> U,
            U -> V,
            V -> W,
            W -> X,
            X -> Y,
            Y -> Z,
            Z -> A,
            EQUAL -> ESC,
            RESERVED -> ESC,
            MINUS -> ESC,
            BACKSPACE -> ESC,
            TAB -> ESC,
            LEFTCTRL -> ESC,
            ENTER -> ESC,
            SEMICOLON -> ESC,
            APOSTROPHE -> ESC,
            GRAVE -> ESC,
            LEFTSHIFT -> ESC,
            BACKSLASH -> ESC,
            COMMA -> ESC,
            DOT -> ESC,
            SLASH -> ESC,
            RIGHTSHIFT -> ESC,
            KPASTERISK -> ESC,
            LEFTALT -> ESC,
            SPACE -> ESC,
            CAPSLOCK -> ESC,
            F1 -> ESC,
            F2 -> ESC,
            F3 -> ESC,
            F4 -> ESC,
            F5 -> ESC,
            F6 -> ESC,
            F7 -> ESC,
            F8 -> ESC,
            F9 -> ESC,
            F10 -> ESC,
            F11 -> ESC,
            F12 -> ESC,
        },
        // TODO: support number keys in various positions
        // TestNumberKeys: {
        //     0 -> TT(ModLayer, 0),
        //     1 -> 3,
        //     2 -> C,
        //     3 -> D,
        //     4 -> E,
        //     5 -> F,
        //     6 -> G,
        //     7 -> H,
        //     8 -> I,
        //     9 -> J,
        // },
    )?;
    Ok(())
}
