use quote::ToTokens;
use syn::ReturnType;

pub struct ReturnValueSetter<'a>(pub &'a ReturnType);

impl<'a> ToTokens for ReturnValueSetter<'a> {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        match self.0 {
            ReturnType::Default => {}
            ReturnType::Type(_, _) => {
                quote!(__res.set_return_value(__ret.into());).to_tokens(tokens);
            }
        };
    }
}
