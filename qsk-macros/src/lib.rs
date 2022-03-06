//! # qsk-macros
//!
//! `qsk-macros` contains the `remap!` macro that enables `qsk` users to concisely define their
//! keyboard remapping layers. Usage looks like the following:
//!
//! ```
//! qsk_macros::remap!(
//!     ModLayer[Active]: {
//!         F -> TT(Navigation, F),
//!     },
//!     Navigation: {
//!         END -> Exit(),
//!         Y -> HOME,
//!         U -> PAGEDOWN,
//!         I -> PAGEUP,
//!         O -> END,
//!         H -> LEFT,
//!         J -> DOWN,
//!         K -> UP,
//!         SEMICOLON -> RIGHT,
//!     },
//! ).unwrap();
//! ```
//!
//! This mini keyboard-remapping DSL expands into a Rust expression with the type
//! [`Result`](qsk_types::Result)<[`LayerComposer`](qsk_types::LayerComposer)>, which can is used
//! in `qsk`'s remapping engine to actually perform keyboard transformations on input.
//!
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

mod analyze;
mod codegen;
mod lower;
mod parse;

#[proc_macro]
#[proc_macro_error]
pub fn remap(ts: TokenStream) -> TokenStream {
    let ast = parse::parse(ts.clone().into());
    let model = analyze::analyze(ast);
    let ir = lower::lower(model);
    let rust = codegen::codegen(ir).into();
    rust
}
