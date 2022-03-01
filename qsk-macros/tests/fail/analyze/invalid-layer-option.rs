use qsk_macros::remap;

fn main() {
    remap!(
        ModLayer[InvalidLayerOption]: {
            F -> F,
        },
    );
}
