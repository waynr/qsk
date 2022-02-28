use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

mod analyze;
mod codegen;
mod lower;
mod parse;

#[proc_macro]
#[proc_macro_error]
pub fn layer(ts: TokenStream) -> TokenStream {
    let ast = parse::parse(ts.clone().into());
    let model = analyze::analyze(ast);
    let ir = lower::lower(model);
    let rust = codegen::codegen(ir).into();
    rust
}
