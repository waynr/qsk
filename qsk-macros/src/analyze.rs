use std::collections::{HashMap, BTreeSet};
use std::str::FromStr;

use proc_macro_error::{abort, abort_call_site};

use qsk_types::{LayerComposer, LayerRef, KeyCode, ControlCode};

use crate::parse;
use crate::parse::{Ast, LayerBody};

const VALID_KEY_FUNCTIONS: [&'static str; 3] = ["TT", "TapToggle", "Exit"];

impl From<parse::KeyFunctionParameter> for LayerRef {
    fn from(parsed: parse::KeyFunctionParameter) -> Self {
        match parsed {
            parse::KeyFunctionParameter::StringParameter(ident) => LayerRef::ByName(ident.to_string()),
        }
    }
}

impl From<parse::KeyFunctionParameter> for KeyCode {
    fn from(parsed: parse::KeyFunctionParameter) -> Self {
        match parsed {
            parse::KeyFunctionParameter::StringParameter(param) => {
                let mut kc_str = param.to_string();
                if !kc_str.starts_with("KC_") {
                    kc_str = "KC_".to_owned() + &kc_str;
                }
                match KeyCode::from_str(&kc_str) {
                    Ok(kc) => kc,
                    Err(e) => {
                        // ../tests/fail/analyze/invalid-key-code-in-key-function.rs
                        abort!(
                            param.span(),
                            format!("invalid key code when converting parse::KeyFunctionParameter to KeyCode: {:?}", e),
                    )},
                }
            },
        }
    }
}

impl From<&parse::KeyFunction> for ControlCode {
    fn from(parsed: &parse::KeyFunction) -> Self {
        let mut params = parsed.params.clone().0.into_iter();
        match parsed.name.to_string().as_str() {
            "Exit" => {
                match params.next() {
                    Some(param) => abort!(
                        // ../tests/fail/analyze/exit-unexpected-arguments.rs
                        param.span(),
                        "unexpected argument",
                        ),
                    None => (),
                }
                ControlCode::Exit
            },
            "TT" | "TapToggle" => {
                let layer_ref = params
                    .next()
                    .unwrap_or_else(|| abort!(
                        // ../tests/fail/analyze/tap-toggle-missing-layer-ref-argument.rs
                        parsed.name.0.span(),
                        "missing layer ref argument"
                    ))
                    .into();
                let key = params
                    .next()
                    .unwrap_or_else(|| abort!(
                        // ../tests/fail/analyze/tap-toggle-missing-keycode-argument.rs
                        parsed.name.0.span(),
                        "missing key code argument"
                    ))
                    .into();
                match params.next() {
                    Some(param) => abort!(
                        // ../tests/fail/analyze/tap-toggle-unexpected-arguments.rs
                        param.span(),
                        "unexpected argument",
                        ),
                    None => (),
                }
                ControlCode::TapToggle(layer_ref, key)
            },
            _ => {
                abort!(
                    // ../tests/fail/analyze/unsupported-key-function.rs
                    parsed.name.span(),
                    "invalid key function";
                    help = format!("valid key functions include: {:?}", VALID_KEY_FUNCTIONS));
            },
        }
    }
}

impl From<&parse::Key> for ControlCode {
    fn from(parsed: &parse::Key) -> ControlCode {
        match KeyCode::from_str(&parsed.to_string()) {
            Ok(kc) => ControlCode::KeyMap(kc),
            Err(e) => abort!(
                // ../tests/fail/analyze/invalid-key-code-control-code.rs
                parsed.span(),
                format!("invalid key code when converting parse::Key to ControlCode: {:?}", e),
            ),
        }
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

impl From<&parse::Key> for KeyCode {
    fn from(parsed: &parse::Key) -> Self {
        match KeyCode::from_str(&parsed.to_string()) {
            Ok(kc) => kc,
            Err(e) => abort!(
                // ../tests/fail/analyze/invalid-key-code-keymap-key.rs
                parsed.span(),
                format!("invalid key code when converting parse::Key to KeyCode: {:?}", e),
            ),
        }
    }
}

impl From<&parse::LayerBody> for HashMap<KeyCode, Vec<ControlCode>> {
    fn from(parsed: &LayerBody) -> Self {
        parsed.maps.iter()
            .map(|km| (
                    KeyCode::from(&km.lhs),
                    vec![ControlCode::from(&km.rhs)])
                )
            .collect()
    }
}

const VALID_LAYER_OPTIONS: [&'static str; 1] = ["Active"];

impl From<&parse::Layer> for qsk_types::Layer {
    fn from(parsed: &parse::Layer) -> Self {
        let body = &parsed.body;
        let mut layer = qsk_types::Layer::from_hashmap(parsed.name.to_string(), body.into(), false);
        match &parsed.opts {
            Some(layer_opts) => {
                for opt in layer_opts.opts.iter() {
                    match opt.to_string().as_str() {
                        "Active" => layer.activate(),
                        _ => {
                            // ../tests/fail/analyze/invalid-layer-option.rs
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

impl From<&Ast> for LayerComposer {
    fn from(parsed: &Ast) -> Self {
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

// Validate references against Ast rather than LayerComposer since this allows us to produce better
// error messages using spans found on the Ast.
pub fn validate_references(ast: &Ast) {
    // first construct set of all valid layer names
    let valid_layer_names: BTreeSet<String> = ast.iter()
        .map(|layer| layer.name.to_string())
        .collect();

    // then iterate over all keymaps looking for all KeyFunctions that take a LayerRef
    for layer in ast.iter() {
        for keymaps in layer.body.iter() {
            match &keymaps.rhs {
                parse::ControlCode::Function(kf) => {
                    match kf.name.to_string().as_str() {
                        "TapToggle" | "TT" => {
                            let layer_ref = &kf.params.0[0];
                            match layer_ref {
                                parse::KeyFunctionParameter::StringParameter(sp) => {
                                    if !valid_layer_names.contains(sp.to_string().as_str()) {
                                        abort!(
                                            layer_ref.span(),
                                            "layer reference does not exist";
                                            help = format!("existing layers include: {:?}", valid_layer_names)
                                        )
                                    }
                                },
                            }
                        },
                        _ => continue,
                    }
                }
                _ => continue,
            }
        }
    }
}

pub fn analyze(ast: Ast) -> LayerComposer {
    let lc = LayerComposer::from(&ast);
    validate_references(&ast);
    lc
}
