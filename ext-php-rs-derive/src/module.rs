use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{ItemFn, Signature};

use crate::{function::Arg, Result};

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub args: Vec<Arg>,
    pub output: Option<(String, bool)>,
}

pub fn parser(input: ItemFn) -> Result<TokenStream> {
    let ItemFn { sig, block, .. } = input;
    let Signature { output, inputs, .. } = sig;
    let stmts = &block.stmts;

    let builders = crate::STATE.with(|state| {
        let mut state = state.lock().unwrap();

        if state.built_module {
            return Err("You may only define a module with the `#[php_module]` attribute once.");
        }

        state.built_module = true;
        Ok(state
            .functions
            .iter()
            .map(|func| func.get_builder())
            .collect::<Vec<_>>())
    })?;

    let result = quote! {
        #[no_mangle]
        pub extern "C" fn get_module() -> *mut ::ext_php_rs::php::module::ModuleEntry {
            fn internal(#inputs) #output {
                #(#stmts)*
            }

            let mut builder = ::ext_php_rs::php::module::ModuleBuilder::new(
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            )
            #(.function(#builders.unwrap()))*
            ;

            // TODO allow result return types
            let builder = internal(builder);

            match builder.build() {
                Ok(module) => module.into_raw(),
                Err(e) => panic!("Failed to build PHP module: {:?}", e),
            }
        }
    };
    Ok(result)
}

impl Function {
    #[inline]
    fn get_name_ident(&self) -> Ident {
        Ident::new(&self.name, Span::call_site())
    }

    fn get_builder(&self) -> TokenStream {
        let name = &self.name;
        let name_ident = self.get_name_ident();
        let args = self
            .args
            .iter()
            .map(|arg| arg.get_arg_definition())
            .collect::<Vec<_>>();
        let output = self.output.as_ref().map(|(ty, nullable)| {
            let ty = Ident::new(ty, Span::call_site());
            // TODO allow reference returns?
            quote! {
                .returns(::ext_php_rs::php::enums::DataType::#ty, false, #nullable)
            }
        });

        quote! {
            ::ext_php_rs::php::function::FunctionBuilder::new(#name, #name_ident)
                #(.arg(#args))*
                #output
                .build()
        }
    }
}
