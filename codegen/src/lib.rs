mod attr;

use attr::ModuleAttr;
use proc_macro::TokenStream;
use quote::{quote, format_ident};
use regex::Regex;
use convert_case::{Case, Casing};

#[proc_macro_attribute]
pub fn module(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemStruct);
    let attr = syn::parse_macro_input!(attr as ModuleAttr);

    let struct_name = &input.ident;
    let struct_name_scase = struct_name.to_string().to_case(Case::Snake);
    
    let attrs = &input.attrs;

    let user_fields = match &input.fields {
        syn::Fields::Named(fields_named) => &fields_named.named,
        _ => panic!("Module struct must have named fields"),
    };

    let arg_struct_name = format_ident!("{}Arg", struct_name);

    let mut port_name_functions = Vec::new();
    let mut add_port_codes = Vec::new();
    for port_define in attr.ports.iter() {
        let function_name = format_ident!("{}_pn", port_define.name);

        let fields = extract_placeholders(&port_define.pattern);
        match fields.len() {
            0 => {
                let format_arg = port_define.pattern.clone();
                let code = quote! {
                    pub fn #function_name() -> crate::circuit::ShrString {
                        use crate::format_shr;
                        format_shr!(#format_arg)
                    }
                };
                port_name_functions.push(code);

                let direction = &port_define.direction;
                let code = quote! {
                    module.add_port(#struct_name::#function_name(), crate::circuit::PortDirection::#direction)?;
                };
                let code = if let Some(cond) = &port_define.condition {
                    quote! {
                        if #cond {
                            #code
                        }
                    }
                } else {
                    code
                };

                add_port_codes.push(code);
            }
            1 => {
                let format_arg = port_define.pattern.clone();
                let field_ident = format_ident!("{}", fields[0]);
                let code = quote! {
                    pub fn #function_name(#field_ident: usize) -> crate::circuit::ShrString {
                        use crate::format_shr;
                        format_shr!(#format_arg)
                    }
                };
                port_name_functions.push(code);

                let direction = &port_define.direction;
                let code = quote! {
                    for v in 0..module.args.#field_ident {
                        module.add_port(#struct_name::#function_name(v), crate::circuit::PortDirection::#direction)?;
                    }
                };
                let code = if let Some(cond) = &port_define.condition {
                    let code = quote! {
                        if #cond {
                            #code
                        }
                    };
                    code
                } else {
                    code
                };

                
                add_port_codes.push(code);
            }
            2 => {
                let format_arg = port_define.pattern.clone();
                let f1 = format_ident!("{}", fields[0]);
                let f2 = format_ident!("{}", fields[1]);
                let direction = &port_define.direction;
        
                let code = quote! {
                    pub fn #function_name(#f1: usize, #f2: usize) -> crate::circuit::ShrString {
                        use crate::format_shr;
                        format_shr!(#format_arg)
                    }
                };
                port_name_functions.push(code);
        
                let mut code = quote! {
                    for v1 in 0..module.args.#f1 {
                        for v2 in 0..module.args.#f2 {
                            module.add_port(
                                #struct_name::#function_name(v1, v2),
                                crate::circuit::PortDirection::#direction
                            )?;
                        }
                    }
                };
        
                if let Some(cond) = &port_define.condition {
                    code = quote! {
                        if #cond {
                            #code
                        }
                    };
                }
        
                add_port_codes.push(code);
            }
            _ => panic!("Unsupport"),
        }
    }
    
    let explicit_field_names: Vec<_> = user_fields.iter()
        .filter(|f| f.attrs.is_empty())
        .map(|f| &f.ident)
        .collect();


    let field_fmt: String = explicit_field_names
        .iter()
        .map(|_| "{}")
        .collect::<Vec<_>>()
        .join("_");


    let format_string = format!("{}_{}", struct_name_scase, field_fmt);
    let format_lit = syn::LitStr::new(&format_string, struct_name.span());
    let self_fields: Vec<proc_macro2::TokenStream> = explicit_field_names
        .iter()
        .map(|ident| quote! { self.#ident })
        .collect();

    quote! {
        pub use derive_new::new;
        #[derive(Debug, new)]
        pub struct #arg_struct_name {
            #user_fields
        }

        #(#attrs)*
        pub type #struct_name = crate::circuit::Module<#arg_struct_name>;

        impl crate::circuit::ModuleArg for #arg_struct_name {
            fn create_module(self, factory: &mut crate::circuit::CircuitFactory) -> crate::YouRAMResult<#struct_name> {
                let name = self.module_name();
                let mut module = #struct_name::new(name, self);
                #(#add_port_codes)*
                module.build(factory)?;
                Ok(module)
            }

            fn module_name(&self) -> crate::circuit::ShrString {
                use crate::format_shr;
                format_shr!(#format_lit, #(#self_fields),*)
            }
        }

        impl crate::circuit::Module<#arg_struct_name> {
            #(#port_name_functions)*
        }
    }.into()
}


fn extract_placeholders(s: &str) -> Vec<String> {
    let re = Regex::new(r"\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap();
    re.captures_iter(s)
        .map(|cap| cap[1].to_string())
        .collect()
}