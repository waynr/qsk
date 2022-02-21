use std::collections::HashMap;
use std::str::FromStr;

use proc_macro_error::{abort, abort_call_site};

use qsk_types::{LayerComposer, LayerRef, KeyCode, ControlCode};

use crate::parse;
use crate::parse::{Ast, LayerBody};

const VALID_KEY_FUNCTIONS: [&'static str; 3] = ["TT", "TapToggle", "Exit"];

impl From<parse::KeyParameter> for LayerRef {
    fn from(parsed: parse::KeyParameter) -> Self {
        match parsed {
            parse::KeyParameter::Ident(ident) => LayerRef::ByName(ident.to_string()),
        }
    }
}

impl From<parse::KeyParameter> for KeyCode {
    fn from(parsed: parse::KeyParameter) -> Self {
        match parsed {
            parse::KeyParameter::Ident(ident) => {
                match KeyCode::from_str(&ident.to_string()) {
                    Ok(kc) => kc,
                    Err(e) => abort!(
                        ident.span(),
                        format!("invalid key code: {:?}", e),
                    ),
                }
            },
        }
    }
}

impl From<&parse::KeyFunction> for ControlCode {
    fn from(parsed: &parse::KeyFunction) -> Self {
        match parsed.name.to_string().as_str() {
            "Exit" => {
                ControlCode::Exit
            },
            "TT" | "TapToggle" => {
                // TODO: get layer ref and keycode from KeyParameters
                let mut params = parsed.params.clone().into_iter().rev();
                let layer_ref = params
                    .next()
                    .unwrap_or_else(|| abort!(
                        parsed.name.0.span(),
                        "missing layer ref argument"
                    ))
                    .into();
                let key = params
                    .next()
                    .unwrap_or_else(|| abort!(
                        parsed.name.0.span(),
                        "missing key code argument"
                    ))
                    .into();
                match params.next() {
                    Some(param) => abort!(
                        param.span(),
                        "unexpected argument",
                        ),
                    None => (),
                }
                ControlCode::TapToggle(layer_ref, key)
            },
            _ => {
                abort!(
                    parsed.name.0.span(),
                    "invalid key function";
                    help = format!("valid key functions include: {:?}", VALID_KEY_FUNCTIONS));
            },
        }
    }
}

impl From<&parse::KeyFunctionName> for ControlCode {
    fn from(parsed: &parse::KeyFunctionName) -> ControlCode {
        ControlCode::KeyMap(KeyCode::from_str(&parsed.to_string()).unwrap())
    }
}

impl From<&parse::Key> for ControlCode {
    fn from(parsed: &parse::Key) -> ControlCode {
        ControlCode::KeyMap(KeyCode::from_str(&parsed.to_string()).unwrap())
    }
}

impl From<&parse::ControlCode> for ControlCode {
    fn from(parsed: &parse::ControlCode) -> Self {
        match parsed {
            parse::ControlCode::Key(key) => key.into(),
            parse::ControlCode::Function(kf) => kf.into(),
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

const VALID_LAYER_OPTIONS: [&'static str; 1] = ["Active"];

impl From<parse::Layer> for qsk_types::Layer {
    fn from(parsed: parse::Layer) -> Self {
        let mut layer = qsk_types::Layer::from_hashmap(parsed.name.to_string(), parsed.body.into(), false);
        match parsed.opts {
            Some(layer_opts) => {
                for opt in layer_opts.opts.iter() {
                    match opt.to_string().as_str() {
                        "Active" => layer.activate(),
                        _ => {
                            abort!(
                                opt.span(),
                                "invalid layer option";
                                help = format!("valid layer options include: {:?}", VALID_LAYER_OPTIONS));
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
        match LayerComposer::from_layers(
            parsed.iter()
                .map(|layer| layer.into())
                .collect()) {
            Ok(lc) => lc,
            Err(e) => {
                abort_call_site!(format!("invalid layer composer: {:?}", e));
            }
        }
    }
}

pub fn analyze(ast: Ast) -> LayerComposer {
    LayerComposer::from(ast)
}
