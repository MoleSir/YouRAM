use syn::{parse::{Parse, ParseStream}, Token, Ident, LitStr};

pub struct ModuleAttr {
    pub ports: Vec<PortDefine>,
}

// address: ("addr{address_width}", Input),
pub struct PortDefine {
    pub name: Ident,              // input / output
    pub pattern: String,          // "A{n}"
    pub direction: Ident,         // Input / Output / InOut
}

impl Parse for ModuleAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut ports = Vec::new();
        while !input.is_empty() {
            let name: Ident = input.parse()?;
            let _: Token![:] = input.parse()?;
            let content;
            syn::parenthesized!(content in input);
            let pattern: LitStr = content.parse()?;
            let _: Token![,] = content.parse()?;
            let direction: Ident = content.parse()?;

            ports.push(PortDefine { name, pattern: pattern.value(), direction });

            // optional trailing comma
            let _ = input.parse::<Token![,]>();
        }
        Ok(ModuleAttr { ports })
    }
}
