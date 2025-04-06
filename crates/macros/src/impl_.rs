use darling::ast::NestedMeta;
use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{Ident, ItemImpl, Lit};

use crate::function::{Args, CallType, FnArgs, Function, MethodReceiver};
use crate::helpers::{get_docs, GetDocs, Rename, RenameRule};
use crate::prelude::*;

const MAGIC_METHOD: [&'static str; 17] = [
    "__construct",
    "__destruct",
    "__call",
    "__call_static",
    "__get",
    "__set",
    "__isset",
    "__unset",
    "__sleep",
    "__wakeup",
    "__serialize",
    "__unserialize",
    "__to_string",
    "__invoke",
    "__set_state",
    "__clone",
    "__debug_info",
];

trait RenameMethod {
    fn rename_method(self, rule: &RenameRule) -> String;
}

impl<T: AsRef<str>> RenameMethod for T {
    fn rename_method(self, rule: &RenameRule) -> String {
        if MAGIC_METHOD.contains(&self.as_ref()) {
            self.as_ref().to_string()
        } else {
            self.as_ref().to_string().renmae(rule)
        }
    }
}

/// Method visibilities.
#[derive(Debug)]
enum MethodVis {
    Public,
    Private,
    Protected,
}

/// Method types.
#[derive(Debug)]
enum MethodTy {
    /// Regular PHP method.
    Normal,
    /// Constructor method.
    Constructor,
    /// Property getter method.
    Getter,
    /// Property setter method.
    Setter,
    /// Abstract method.
    Abstract,
}

#[derive(Default, Debug, FromMeta)]
#[darling(default)]
pub struct AttrArgs {
    rename_methods: Option<RenameRule>,
}

/// Attribute arguments for `impl` blocks.
#[derive(Debug, Default, FromMeta)]
#[darling(default)]
pub struct ImplArgs {
    /// How the methods are renamed.
    rename_methods: RenameRule,
}

pub fn parser(args: TokenStream, mut input: ItemImpl) -> Result<TokenStream> {
    let meta = NestedMeta::parse_meta_list(args)?;
    let args = match ImplArgs::from_list(&meta) {
        Ok(args) => args,
        Err(e) => bail!(input => "Failed to parse impl attribute arguments: {:?}", e),
    };
    let path = match &*input.self_ty {
        syn::Type::Path(ty) => &ty.path,
        _ => {
            bail!(input.self_ty => "The `#[php_impl]` attribute is only valid for struct implementations.")
        }
    };

    let mut parsed = ParsedImpl::new(path, args.rename_methods);
    parsed.parse(input.items.iter_mut())?;

    let php_class_impl = parsed.generate_php_class_impl()?;
    Ok(quote::quote! {
        #input
        #php_class_impl
    })
}

/// Arguments applied to methods.
#[derive(Debug)]
struct MethodArgs {
    /// Method name. Only applies to PHP (not the Rust method name).
    name: String,
    /// The first optional argument of the function signature.
    optional: Option<Ident>,
    /// Default values for optional arguments.
    defaults: HashMap<Ident, Lit>,
    /// Visibility of the method (public, protected, private).
    vis: MethodVis,
    /// Method type.
    ty: MethodTy,
}

impl MethodArgs {
    fn new(name: String) -> Self {
        let ty = if name == "__construct" {
            MethodTy::Constructor
        } else {
            MethodTy::Normal
        };
        Self {
            name,
            optional: Default::default(),
            defaults: Default::default(),
            vis: MethodVis::Public,
            ty,
        }
    }

    fn parse(&mut self, attrs: &mut Vec<syn::Attribute>) -> Result<()> {
        let mut unparsed = vec![];
        unparsed.append(attrs);
        for attr in unparsed {
            let path = &attr.path();

            if path.is_ident("optional") {
                // x
                if self.optional.is_some() {
                    bail!(attr => "Only one `#[optional]` attribute is valid per method.");
                }
                let optional = attr.parse_args().map_err(
                    |e| err!(e.span() => "Invalid arguments passed to `#[optional]` attribute. {}", e),
                )?;
                self.optional = Some(optional);
            } else if path.is_ident("defaults") {
                let defaults = HashMap::from_meta(&attr.meta).map_err(
                    |e| err!(e.span() => "Invalid arguments passed to `#[defaults]` attribute. {}", e),
                )?;
                self.defaults = defaults;
            } else if path.is_ident("public") {
                // x
                self.vis = MethodVis::Public;
            } else if path.is_ident("protected") {
                // x
                self.vis = MethodVis::Protected;
            } else if path.is_ident("private") {
                // x
                self.vis = MethodVis::Private;
            } else if path.is_ident("rename") {
                let lit: syn::Lit = attr.parse_args().map_err(|e| err!(attr => "Invalid arguments passed to the `#[rename]` attribute. {}", e))?;
                match lit {
                    Lit::Str(name) => self.name = name.value(),
                    _ => bail!(attr => "Only strings are valid method names."),
                };
            } else if path.is_ident("getter") {
                // x
                self.ty = MethodTy::Getter;
            } else if path.is_ident("setter") {
                // x
                self.ty = MethodTy::Setter;
            } else if path.is_ident("constructor") {
                // x
                self.ty = MethodTy::Constructor;
            } else if path.is_ident("abstract_method") {
                // x
                self.ty = MethodTy::Abstract;
            } else {
                attrs.push(attr);
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct ParsedImpl<'a> {
    path: &'a syn::Path,
    rename: RenameRule,
    functions: Vec<FnBuilder>,
    constructor: Option<Function<'a>>,
    constants: Vec<Constant<'a>>,
}

#[derive(Debug)]
struct FnBuilder {
    /// Tokens which represent the FunctionBuilder for this function.
    pub builder: TokenStream,
    /// The visibility of this method.
    pub vis: MethodVis,
    /// Whether this method is abstract.
    pub r#abstract: bool,
}

#[derive(Debug)]
struct Constant<'a> {
    /// Name of the constant in PHP land.
    name: String,
    /// Identifier of the constant in Rust land.
    ident: &'a syn::Ident,
    /// Documentation for the constant.
    docs: Vec<String>,
}

impl<'a> ParsedImpl<'a> {
    /// Create a new, empty parsed impl block.
    ///
    /// # Parameters
    ///
    /// * `path` - Path of the type the `impl` block is for.
    /// * `rename` - Rename rule for methods.
    fn new(path: &'a syn::Path, rename: RenameRule) -> Self {
        Self {
            path,
            rename,
            functions: Default::default(),
            constructor: Default::default(),
            constants: Default::default(),
        }
    }

    /// Parses an impl block from `items`, populating `self`.
    fn parse(&mut self, items: impl Iterator<Item = &'a mut syn::ImplItem>) -> Result<()> {
        for items in items {
            match items {
                syn::ImplItem::Const(c) => {
                    let mut name = None;
                    let mut unparsed = vec![];
                    unparsed.append(&mut c.attrs);
                    for attr in unparsed {
                        if attr.path().is_ident("rename") {
                            let lit: syn::Lit = attr.parse_args().map_err(|e| err!(attr => "Invalid arguments passed to the `#[rename]` attribute. {}", e))?;
                            match lit {
                                Lit::Str(str) => name = Some(str.value()),
                                _ => bail!(attr => "Only strings are valid constant names."),
                            };
                        } else {
                            c.attrs.push(attr);
                        }
                    }
                    let docs = get_docs(&c.attrs);

                    self.constants.push(Constant {
                        name: name.unwrap_or_else(|| c.ident.to_string()),
                        ident: &c.ident,
                        docs,
                    });
                }
                syn::ImplItem::Fn(method) => {
                    let name = method.sig.ident.to_string().renmae(&self.rename);
                    let docs = method.attrs.as_slice().get_docs();

                    let mut opts = MethodArgs::new(name);
                    opts.parse(&mut method.attrs)?;

                    let args = Args::parse_from_fnargs(method.sig.inputs.iter(), opts.defaults)?;
                    let mut func =
                        Function::new(&method.sig, Some(opts.name), args, opts.optional, docs)?;

                    if matches!(opts.ty, MethodTy::Constructor) {
                        if self.constructor.replace(func).is_some() {
                            bail!(method => "Only one constructor can be provided per class.");
                        }
                    } else {
                        let call_type = CallType::Method {
                            class: self.path,
                            receiver: if func.args.receiver.is_some() {
                                // `&self` or `&mut self`
                                MethodReceiver::Class
                            } else if func
                                .args
                                .typed
                                .first()
                                .map(|arg| arg.name == "self_")
                                .unwrap_or_default()
                            {
                                // `self_: &[mut] ZendClassObject<Self>`
                                // Need to remove arg from argument list
                                func.args.typed.pop();
                                MethodReceiver::ZendClassObject
                            } else {
                                // Static method
                                MethodReceiver::Static
                            },
                        };
                        let builder = func.function_builder(call_type)?;
                        self.functions.push(FnBuilder {
                            builder,
                            vis: opts.vis,
                            r#abstract: matches!(opts.ty, MethodTy::Abstract),
                        });
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Generates an `impl PhpClassImpl<Self> for PhpClassImplCollector<Self>`
    /// block.
    fn generate_php_class_impl(&self) -> Result<TokenStream> {
        let path = &self.path;
        let functions = &self.functions;
        let constructor = match &self.constructor {
            Some(func) => Some(func.constructor_meta(self.path)?),
            None => None,
        }
        .option_tokens();
        let constants = self.constants.iter().map(|c| {
            let name = &c.name;
            let ident = c.ident;
            let docs = &c.docs;
            quote! {
                (#name, &#path::#ident, &[#(#docs),*])
            }
        });

        Ok(quote! {
            impl ::ext_php_rs::internal::class::PhpClassImpl<#path>
                for ::ext_php_rs::internal::class::PhpClassImplCollector<#path>
            {
                fn get_methods(self) -> ::std::vec::Vec<
                    (::ext_php_rs::builders::FunctionBuilder<'static>, ::ext_php_rs::flags::MethodFlags)
                > {
                    vec![#(#functions),*]
                }

                fn get_method_props<'a>(self) -> ::std::collections::HashMap<&'static str, ::ext_php_rs::props::Property<'a, #path>> {
                    todo!()
                }

                fn get_constructor(self) -> ::std::option::Option<::ext_php_rs::class::ConstructorMeta<#path>> {
                    #constructor
                }

                fn get_constants(self) -> &'static [(&'static str, &'static dyn ::ext_php_rs::convert::IntoZvalDyn, &'static [&'static str])] {
                    &[#(#constants),*]
                }
            }
        })
    }
}

impl quote::ToTokens for FnBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let builder = &self.builder;
        // TODO(cole_d): allow more flags via attributes
        let mut flags = vec![];
        flags.push(match self.vis {
            MethodVis::Public => quote! { ::ext_php_rs::flags::MethodFlags::Public },
            MethodVis::Protected => quote! { ::ext_php_rs::flags::MethodFlags::Protected },
            MethodVis::Private => quote! { ::ext_php_rs::flags::MethodFlags::Private },
        });
        if self.r#abstract {
            flags.push(quote! { ::ext_php_rs::flags::MethodFlags::Abstract });
        }
        quote! {
            (#builder, #(#flags)*)
        }
        .to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use super::RenameMethod;
    use super::RenameRule;
    use super::MAGIC_METHOD;

    #[test]
    fn test_rename_magic() {
        for magic in MAGIC_METHOD {
            assert_eq!(magic, magic.rename_method(&RenameRule::None));
            assert_eq!(magic, magic.rename_method(&RenameRule::Camel));
            assert_eq!(magic, magic.rename_method(&RenameRule::Snake));
        }
    }

    #[test]
    fn test_rename_php_methods() {
        let &(original, camel, snake) = &("get_name", "getName", "get_name");
        assert_eq!(original, original.rename_method(&RenameRule::None));
        assert_eq!(camel, original.rename_method(&RenameRule::Camel));
        assert_eq!(snake, original.rename_method(&RenameRule::Snake));
    }
}
