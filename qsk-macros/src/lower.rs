use proc_macro2::Span;
use proc_macro_error::abort_call_site;
use syn::{LitBool, LitStr, Ident, Path, PathSegment, PathArguments};
use syn::punctuated::Punctuated;

use qsk_types;

pub struct LayerRefByName {
    pub(crate) path: Path,
    pub(crate) name: LitStr,
}

pub struct TapToggle {
    pub(crate) name: Path,
    pub(crate) layer_ref: LayerRefByName,
    pub(crate) tap_key: Path,
}

pub enum ControlCode {
    Key(Path),
    TapToggle(TapToggle),
    Exit(Path),
}

impl From<&qsk_types::ControlCode> for ControlCode {
    fn from(cc: &qsk_types::ControlCode) -> ControlCode {
        match cc {
            qsk_types::ControlCode::KeyMap(kc) => {
                ControlCode::Key(
                    keycode_path(&kc.to_string())
                )
            },
            qsk_types::ControlCode::TapToggle(layer_ref, kc) => {
                ControlCode::TapToggle(TapToggle{
                    name: control_code_path("TapToggle"),
                    layer_ref: match layer_ref {
                        qsk_types::LayerRef::ByName(name) => {
                            LayerRefByName{
                                path: path_from_vec_str(vec!["qsk_types", "LayerRef", "ByName"]),
                                name: LitStr::new(name, Span::call_site()),
                            }
                        },
                        qsk_types::LayerRef::ByIndex(_) => {
                            abort_call_site!("referencing layers by index is unsupported")
                        },
                    },
                    tap_key: keycode_path(&kc.to_string()),
                    })
            },
            qsk_types::ControlCode::Exit => {
                ControlCode::Exit(
                    control_code_path("Exit"),
                )
            },
            _ => {
                abort_call_site!("unsupported control code")
            },
        }
    }
}

pub struct KeyMap {
    pub(crate) key: Path,
    pub(crate) control_code: Vec<ControlCode>,
}

pub struct Layer {
    pub(crate) name: LitStr,
    pub(crate) active: LitBool,
    pub(crate) maps: Vec<KeyMap>,
}

impl From<&qsk_types::Layer> for Layer {
    fn from(layer: &qsk_types::Layer) -> Layer {
        Layer{
            name: LitStr::new(&layer.name, Span::call_site()),
            active: LitBool::new(layer.active, Span::call_site()),
            maps: layer
                .iter()
                .map(|(k, v)| KeyMap{
                    key: event_code_to_path(k.into()),
                    control_code: v
                        .iter()
                        .map(|cc| {
                            cc.into()
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

pub struct Ir {
    pub(crate) layers: Vec<Layer>,
}

impl From<qsk_types::LayerComposer> for Ir {
    fn from(lc: qsk_types::LayerComposer) -> Ir {
        Ir {
            layers: lc
                .iter()
                .map(|layer| layer.into())
                .collect(),
        }
    }
}

pub fn lower(model: qsk_types::LayerComposer) -> Ir {
    model.into()
}

fn event_code_to_path(ec: &qsk_types::EventCode) -> Path {
    match ec {
        qsk_types::EventCode::KeyCode(kc) => {
            keycode_path(&kc.to_string())
        },
        qsk_types::EventCode::SynCode(_) => {
            abort_call_site!("syn code not allowed in proc macro")
        },
    }
}

fn keycode_path(keycode_str: &str) -> Path {
    path_from_vec_str(vec![
                      "qsk_types", "KeyCode", keycode_str,
    ])
}

fn control_code_path(variant_str: &str) -> Path {
    path_from_vec_str(vec!["qsk_types", "ControlCode", variant_str])
}

fn path_from_vec_str(parts: Vec<&str>) -> Path {
    Path{ 
        leading_colon: None,
        segments: Punctuated::from_iter(parts.into_iter()
            .map(|part| PathSegment{
                ident: Ident::new(part, Span::call_site()),
                arguments: PathArguments::None,
            }))
    }
}
