use std::collections::HashMap;
use std::str::FromStr;

use proc_macro_error::abort;

use qsk_types::{LayerComposer, KeyCode, ControlCode};

use crate::parse;
use crate::parse::{Ast, LayerBody};

impl From<&parse::ControlCode> for ControlCode {
    fn from(parsed: &parse::ControlCode) -> Self {
        match parsed {
            parse::ControlCode::Key(ident) => 
               ControlCode::KeyMap(KeyCode::from_str(&ident.to_string()).unwrap()),
            parse::ControlCode::Function(kf) => 
                ControlCode::KeyMap(KeyCode::KC_F),
        }
    }
}

impl From<parse::LayerBody> for HashMap<KeyCode, Vec<ControlCode>> {
    fn from(parsed: LayerBody) -> Self {
        parsed.maps.iter()
            .map(|km| (
                    KeyCode::from_str(&km.lhs.to_string()).unwrap(),
                    vec![ControlCode::from(&km.rhs)])
                )
            .collect()
    }
}

const VALID_OPTIONS: [&'static str; 1] = ["Active"];

impl From<parse::Layer> for qsk_types::Layer {
    fn from(parsed: parse::Layer) -> Self {
        let mut layer = qsk_types::Layer::from_hashmap(parsed.body.into(), false);
        layer.set_name(parsed.name.to_string());
        match parsed.opts {
            Some(layer_opts) => {
                for opt in layer_opts.opts.iter() {
                    match opt.to_string().as_str() {
                        "Active" => layer.set_active(true),
                        _ => {
                            abort!(
                                opt.span(),
                                "invalid layer option";
                                help = format!("valid layer options include: {:?}", VALID_OPTIONS));
                        },
                    }
                }
            },
            _ => (),
        }
        layer
    }
}

impl From<Ast> for LayerComposer {
    fn from(parsed: Ast) -> Self {
        LayerComposer::from_layers(
            parsed.iter()
                .map(|layer| layer.into())
                .collect(),
        )
    }
}

pub fn analyze(ast: Ast) -> LayerComposer {
    LayerComposer::from(ast)
}
