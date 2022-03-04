use proc_macro2::TokenStream;
use quote::quote;
use syn::Path;

use crate::lower;

impl lower::ControlCode {
    fn to_tokenstream(&self, key_path: &Path) -> TokenStream {
        match self {
            lower::ControlCode::Key(control_code_path) => {
                quote!(
                    (#key_path, vec![qsk_types::ControlCode::KeyMap(#control_code_path)])
                )
            },
            lower::ControlCode::TapToggle(tt) => {
                let tt_name = &tt.name;
                let layer_ref_path = &tt.layer_ref.path;
                let layer_ref_name = &tt.layer_ref.name;
                let tap_key = &tt.tap_key;
                quote!(
                    (#key_path, vec![#tt_name(#layer_ref_path(#layer_ref_name.to_string()), #tap_key)])
                )
            },
            lower::ControlCode::Exit(path) => {
                quote!(
                    (#key_path, vec![#path])
                )
            },
        }
    }
}

impl From<&lower::KeyMap> for TokenStream {
    fn from(km: &lower::KeyMap) -> Self {
        km.control_code
            .iter()
            .map(|cc| cc.to_tokenstream(&km.key))
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
