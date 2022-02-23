use qsk_macros::layer;

fn main() {
    layer!(
        ModLayer[Meow]: {
            F -> MEOW(Navigation),
        },
    );
}
