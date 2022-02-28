use proc_macro2::TokenStream;
use quote::quote;

use crate::lower;

pub fn codegen(ir: lower::Ir) -> TokenStream {
    let layers_quoted: Vec<TokenStream> = ir.layers.into_iter()
        .map(|layer| {
            let name = layer.name;
            let active = layer.active;
            let maps: Vec<TokenStream> = layer.maps.into_iter()
                .map(|map| {
                    let key_path = map.key;
                    map.control_code
                        .iter()
                        .map(|cc| {
                            match cc {
                                lower::ControlCode::Key(control_code_path) => {
                                    quote!(
                                        #key_path => vec![qsk_types::ControlCode::KeyMap(#control_code_path)]
                                    )
                                },
                                lower::ControlCode::TapToggle(tt) => {
                                    let tt_name = &tt.name;
                                    let layer_ref_path = &tt.layer_ref.path;
                                    let layer_ref_name = &tt.layer_ref.name;
                                    let tap_key = &tt.tap_key;
                                    quote!(
                                        #key_path => vec![#tt_name(#layer_ref_path(#layer_ref_name.to_string()), #tap_key)]
                                    )
                                },
                                lower::ControlCode::Exit(path) => {
                                    quote!(
                                        #key_path => vec![#path]
                                    )
                                },
                            }
                        })
                        .collect()
                })
                .collect();
            quote!(
                qsk_types::Layer::from_hashmap(
                    String::from(#name),
                    hashmap!(
                        #(#maps),*
                    ),
                    #active,
                )
            )
        })
        .collect();

    quote!(
        qsk_types::LayerComposer::from_layers(
            vec![#(#layers_quoted),*]
        )
    )
}
