use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

mod analyze;
mod codegen;
mod lower;
mod parse;

#[proc_macro]
#[proc_macro_error]
pub fn layer(ts: TokenStream) -> TokenStream {
    let _ = parse::parse(ts.clone().into());
    //let model = analyze::analyze(ast);
    //let ir = lower::lower(model);
    //let rust = codegen::codegen(ir).into();
    TokenStream::new()
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
