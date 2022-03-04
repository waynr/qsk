use proc_macro2::TokenStream;
use quote::quote;

use crate::lower;

impl From<&lower::ControlCode> for TokenStream {
    fn from(cc: &lower::ControlCode) -> TokenStream {
        match cc {
            lower::ControlCode::Key(control_code_path) => {
                quote!(
                    vec![qsk_types::ControlCode::KeyMap(#control_code_path)]
                )
            },
            lower::ControlCode::TapToggle(tt) => {
                let tt_name = &tt.name;
                let layer_ref_path = &tt.layer_ref.path;
                let layer_ref_name = &tt.layer_ref.name;
                let tap_key = &tt.tap_key;
                quote!(
                    vec![#tt_name(#layer_ref_path(#layer_ref_name.to_string()), #tap_key)]
                )
            },
            lower::ControlCode::Exit(path) => {
                quote!(
                    vec![#path]
                )
            },
        }
    }
}

impl From<&lower::KeyMap> for TokenStream {
    fn from(km: &lower::KeyMap) -> Self {
        let key_path = &km.key;
        km.control_code
            .iter()
            .map(|cc| cc.into())
            .map(|cc_ts: TokenStream| {
                quote!(
                    (#key_path, #cc_ts)
                )
            })
            .collect()
    }
}

impl From<&lower::Layer> for TokenStream {
    fn from(layer: &lower::Layer) -> Self {
        let name = &layer.name;
        let active = &layer.active;
        let maps: Vec<TokenStream> = layer.maps
            .iter()
            .map(TokenStream::from)
            .collect();

        quote!(
            qsk_types::Layer::from_hashmap(
                String::from(#name),
                std::collections::HashMap::from([
                    #(#maps),*
                ]),
                #active,
            )
        )
    }
}

pub fn codegen(ir: lower::Ir) -> TokenStream {
    let layers_quoted: Vec<TokenStream> = ir.layers
        .iter()
        .map(TokenStream::from)
        .collect();

    quote!(
        qsk_types::LayerComposer::from_layers(
            vec![#(#layers_quoted),*]
        )
    )
}
