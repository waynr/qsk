use qsk_macros::layer;

fn main() {
    layer!(
        ModLayer[Meow]: {
            MEOW -> F,
        },
    );
}
