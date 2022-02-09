use proc_macro2::{TokenStream, TokenTree};
use syn::{braced, bracketed, Result, Token, Ident, parse2, parenthesized};
use syn::buffer::Cursor;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use proc_macro_error::abort;

pub enum KeyParameter {
    Ident(Ident),
}

impl Parse for KeyParameter {
    fn parse(stream: ParseStream) -> Result<Self> {
        Ok(KeyParameter::Ident(stream.parse()?))
    }
}

pub struct KeyFunction {
    name: Ident,
    params: Punctuated<KeyParameter, Token![,]>,
}

impl KeyFunction {
    fn name_only(name: Ident) -> Self {
        KeyFunction{
            name,
            params: Punctuated::new(),
        }
    }
}

impl Parse for KeyFunction {
    fn parse(stream: ParseStream) -> Result<Self> {
        let name = stream.parse()?;
        //let content;
        //parenthesized!(content in stream);
        Ok(KeyFunction{
            name,
            params: stream.parse_terminated(KeyParameter::parse)?,
        })
    }
}

pub enum ControlCode {
    Key(Ident),
    Function(KeyFunction),
}

impl Parse for ControlCode {
    fn parse(stream: ParseStream) -> Result<Self> {
        stream.step(|cursor| {
            let name: Ident;
            let mut rest = *cursor;

            // first token should always be an Ident, either the name of the key or the name of the 
            if let Some((tt, next)) = rest.token_tree() {
                match tt {
                    TokenTree::Ident(ident) => {
                        name = ident;
                        rest = next;
                    }
                    _ => return Err(cursor.error("expected control code identifier missing")),
                }
            } else {
                return Err(cursor.error("no control code was found"))
            }

            let funccall: KeyFunction;
            // if there is a second token tree that's not a punct and it's a group
            if let Some((tt, next)) = rest.token_tree() {
                match &tt {
                    // match comma at end of straight keymap, eg 'Y -> HOME,'
                    //                                                     ^
                    TokenTree::Punct(punct) => {
                        if punct.as_char() != ',' {
                            return Err(cursor.error("unexpected punctuation"))
                        }
                        return Ok((ControlCode::Key(name), rest))
                    },
                    // match key function, eg 'F -> TT(Navigation),'
                    //                              ^^^^^^^^^^^^^^
                    TokenTree::Group(group) => {
                        if group.stream().is_empty() {
                            funccall = KeyFunction::name_only(name.clone());
                            rest = next;
                        } else {
                            match parse2::<KeyFunction>(group.stream()) {
                                Ok(kf) => {
                                    funccall = kf;
                                    rest = next;
                                },
                                Err(e) => return Err(e),
                            }
                        }
                    },
                    _ => return Err(cursor.error(format!("unexpected token tree: {:?}", name))),
                }
            } else {
                return Err(cursor.error("missing tokens"));
            }

            if let Some((tt, next)) = rest.token_tree() {
                match &tt {
                    // match comma at end of straight key func, eg 'Y -> EXIT(),'
                    //                                                         ^
                    TokenTree::Punct(punct) => {
                        if punct.as_char() != ',' {
                            return Err(cursor.error("unexpected punctuation"))
                        }
                        return Ok((ControlCode::Function(funccall), rest))
                    },
                    _ => return Err(cursor.error(format!("unexpected token tree: {:?}", name))),
                }
            }

            Err(cursor.error("missing tokens"))
        })
    }
}

pub struct KeyMaps {
    lhs: Ident,
    op: Token![->],
    rhs: ControlCode,
}

impl Parse for KeyMaps {
    fn parse(stream: ParseStream) -> Result<Self> {
        Ok(KeyMaps{
            lhs: stream.parse()?,
            op: stream.parse()?,
            rhs: stream.parse()?,
        })
    }
}

pub struct LayerBody {
    maps: Punctuated<KeyMaps, Token![,]>,
}

impl Parse for LayerBody {
    fn parse(stream: ParseStream) -> Result<Self> {
        let content;
        braced!(content in stream);
        Ok(LayerBody{
            maps: content.parse_terminated(KeyMaps::parse)?,
        })
    }
}

pub struct LayerOpts {
    opts: Punctuated<Ident, Token![,]>,
}

impl Parse for LayerOpts {
    fn parse(stream: ParseStream) -> Result<Self> {
        let content;
        bracketed!(content in stream);
        Ok(LayerOpts{
            opts: content.parse_terminated(Ident::parse)?,
        })
    }
}

pub struct Layer {
    name: Ident,
    opts: Option<LayerOpts>,
    body: LayerBody,
}

impl Parse for Layer {
    fn parse(stream: ParseStream) -> Result<Self> {
        let name = stream.parse()?;
        let mut opts: Option<LayerOpts> = None;
        if let Ok(o) = stream.parse() {
            opts = Some(o);
        }
        stream.parse::<Token![:]>()?;
        Ok(Layer {
            name,
            opts,
            body: stream.parse()?,
        })
    }
}

pub struct Ast {
    layers: Punctuated<Layer, Token![,]>,
}

impl Parse for Ast {
    fn parse(stream: ParseStream) -> Result<Self> {
        let layers = stream.parse_terminated(Layer::parse)?;
        Ok(Ast {
            layers,
        })
    }
}

pub fn parse(ts: TokenStream) -> Ast {
    match parse2::<Ast>(ts) {
        Ok(ast) => ast,
        Err(e) => {
            abort!(e.span(), e)
        },
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;

    use super::*;
    #[test]
    fn valid_syntax() {
        parse(
            quote!(
                ModLayer[Active]: {
                    F -> TT(Navigation),
                },
                Navigation: {
                    END -> Exit(),
                    Y -> HOME,
                    U -> PAGEDOWN,
                    I -> PAGEUP,
                    O -> END,
                    H -> LEFT,
                    J -> DOWN,
                    K -> UP,
                    SEMICOLON -> RIGHT,
                },
            ),
        );
    }
}
