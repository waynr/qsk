use proc_macro2::{TokenStream, TokenTree, Span};
use syn::{braced, bracketed, Result, Token, Ident, parse2};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use proc_macro_error::abort;

#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StringParameter(Ident);

impl ToString for StringParameter {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl StringParameter {
    pub(crate) fn span(&self) -> Span {
        self.0.span()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyParameter {
    StringParameter(StringParameter),
}

impl Parse for KeyParameter {
    fn parse(stream: ParseStream) -> Result<Self> {
        Ok(KeyParameter::StringParameter(StringParameter(stream.parse()?)))
    }
}

impl KeyParameter {
    pub(crate) fn span(&self) -> Span {
        match self {
            Self::StringParameter(ident) => ident.span(),
        }
    }
}

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq)]
pub struct KeyFunctionName(pub Ident);

impl ToString for KeyFunctionName {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl KeyFunctionName {
    pub(crate) fn span(&self) -> Span {
        self.0.span()
    }
}

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq)]
pub struct Key(pub Ident);

impl ToString for Key {
    fn to_string(&self) -> String {
        let mut s = self.0.to_string();
        if !s.starts_with("KC_") {
            s = "KC_".to_owned() + &s;
        }
        s
    }
}

impl Key {
    pub(crate) fn span(&self) -> Span {
        self.0.span()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyFunctionParameters(pub(crate) Punctuated<KeyParameter, Token![,]>);

impl Parse for KeyFunctionParameters {
    fn parse(stream: ParseStream) -> Result<Self> {
        Ok(KeyFunctionParameters(stream.parse_terminated(KeyParameter::parse)?))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct KeyFunction {
    pub(crate) name: KeyFunctionName,
    pub(crate) params: KeyFunctionParameters,
}

impl KeyFunction {
    fn name_only(name: Ident) -> Self {
        KeyFunction{
            name: KeyFunctionName(name),
            params: KeyFunctionParameters(Punctuated::new()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ControlCode {
    Key(Key),
    Function(KeyFunction),
}

impl Parse for ControlCode {
    fn parse(stream: ParseStream) -> Result<Self> {
        stream.step(|cursor| {
            let name: Ident;
            let mut rest = *cursor;

            // first token should always be an Ident, either the name of the key or the name of the function
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
                        return Ok((ControlCode::Key(Key(name)), rest))
                    },
                    // match key function params, eg 'F -> TT(Navigation),'
                    //                                       ^^^^^^^^^^^^
                    TokenTree::Group(group) => {
                        if group.stream().is_empty() {
                            funccall = KeyFunction::name_only(name.clone());
                            rest = next;
                        } else {
                            match parse2::<KeyFunctionParameters>(group.stream()) {
                                Ok(kps) => {
                                    funccall = KeyFunction{
                                        name: KeyFunctionName(name.clone()),
                                        params: kps,
                                    };
                                    rest = next;
                                },
                                Err(e) => return Err(e),
                            }
                        }
                    },
                    _ => return Err(cursor.error(format!("unexpected token tree: {:?}", name))),
                }
            } else {
                // if there is no additional token, then we have ControlCode::Key
                return Ok((ControlCode::Key(Key(name)), rest))
            }

            // if we find any additional token trees then something is wrong.
            if let Some((tt, _)) = rest.token_tree() {
                match &tt {
                    // match comma at end of straight keymap, eg 'Y -> EXIT(),'
                    //                                                       ^
                    TokenTree::Punct(punct) => {
                        if punct.as_char() != ',' {
                            return Err(cursor.error("unexpected punctuation"))
                        }
                        return Ok((ControlCode::Key(Key(name)), rest))
                    },
                    _ => return Err(cursor.error(format!("unexpected token tree: {:?}", name))),
                }
            }

            return Ok((ControlCode::Function(funccall), rest));
        })
    }
}

pub struct KeyMaps {
    pub(crate) lhs: Key,
    pub(crate) rhs: ControlCode,
}

impl Parse for KeyMaps {
    fn parse(stream: ParseStream) -> Result<Self> {
        let lhs = Key(stream.parse()?);
        stream.parse::<Token![->]>()?; // discard operator for now
        let rhs = stream.parse()?;
        Ok(KeyMaps{
            lhs,
            rhs,
        })
    }
}

pub struct LayerBody {
    pub(crate) maps: Punctuated<KeyMaps, Token![,]>,
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
    pub(crate) opts: Punctuated<Ident, Token![,]>,
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
    pub(crate) name: Ident,
    pub(crate) opts: Option<LayerOpts>,
    pub(crate) body: LayerBody,
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
    pub(crate) layers: Punctuated<Layer, Token![,]>,
}

impl Ast {
    pub fn iter(self) -> impl Iterator<Item = Layer> {
        self.layers.into_iter()
    }
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

    use galvanic_assert::matchers::*;
    use galvanic_assert::*;
    use syn::Result;

    use super::*;
    #[test]
    fn valid_syntax() {
        parse(
            quote!(
                ModLayer[Active]: {
                    F -> TT(Navigation, F),
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

    #[test]
    fn parse_control_code_key() -> Result<()> {
        // validate that F is parsed and ToString impl outputs "KC_F"
        let ts = quote!(F);
        let parsed = parse2::<ControlCode>(ts)?;
        let expected = ControlCode::Key(Key(Ident::new("F", Span::call_site())));
        assert_that!(&parsed, eq(expected));

        if let ControlCode::Key(key) = parsed {
            assert_that!(&key.to_string(), eq(String::from("KC_F")));
        } else {
            assert!(false);
        }

        // validate that KC_F is parsed and ToString impl outputs "KC_F"
        let ts = quote!(KC_F);
        let parsed = parse2::<ControlCode>(ts)?;
        let expected = ControlCode::Key(Key(Ident::new("KC_F", Span::call_site())));
        assert_that!(&parsed, eq(expected));

        if let ControlCode::Key(key) = parsed {
            assert_that!(&key.to_string(), eq(String::from("KC_F")));
        } else {
            assert!(false);
        }
        Ok(())
    }

    #[test]
    fn parse_control_code_function() -> Result<()> {
        let ts = quote!(TapToggle(Navigation, F));
        let _ = parse2::<ControlCode>(ts)?;
        Ok(())
    }
}
