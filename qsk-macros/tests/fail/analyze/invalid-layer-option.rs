use qsk_macros::layer;

fn main() {
    layer!(
        ModLayer[InvalidLayerOption]: {
            F -> F,
        },
    );
}
