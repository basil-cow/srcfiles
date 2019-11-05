use quote::ToTokens;
use std::string::ToString;

pub trait ToTokenString {
    fn to_token_string(&self) -> String;
}

impl<T: ToTokens> ToTokenString for T {
    fn to_token_string(&self) -> String {
        self.to_token_stream().to_string()
    }
}
