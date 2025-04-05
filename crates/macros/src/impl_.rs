use darling::util::Flag;
use darling::FromAttributes;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{Ident, ItemImpl, Lit};

use crate::constant::PhpConstAttribute;
use crate::function::{Args, CallType, Function, MethodReceiver};
use crate::helpers::get_docs;
use crate::parsing::{PhpRename, RenameRule, Visibility};
use crate::prelude::*;

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

#[derive(FromAttributes, Debug, Default)]
#[darling(attributes(php), default)]
pub struct PhpImpl {
    /// Rename methods to match the given rule.
    rename_methods: Option<RenameRule>,
    /// Rename constants to match the given rule.
    rename_constants: Option<RenameRule>,
}

pub fn parser(mut input: ItemImpl) -> Result<TokenStream> {
    let args = PhpImpl::from_attributes(&input.attrs)?;
    input.attrs.retain(|attr| !attr.path().is_ident("php"));
    let path = match &*input.self_ty {
        syn::Type::Path(ty) => &ty.path,
        _ => {
            bail!(input.self_ty => "The `#[php_impl]` attribute is only valid for struct implementations.")
        }
    };

    let mut parsed = ParsedImpl::new(
        path,
        args.rename_methods.unwrap_or(RenameRule::Camel),
        args.rename_constants
            .unwrap_or(RenameRule::ScreamingSnakeCase),
    );
    parsed.parse(input.items.iter_mut())?;

    let php_class_impl = parsed.generate_php_class_impl();
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
    vis: Visibility,
    /// Method type.
    ty: MethodTy,
}

#[derive(FromAttributes, Default, Debug)]
#[darling(default, attributes(php), forward_attrs(doc))]
pub struct PhpFunctionImplAttribute {
    #[darling(flatten)]
    rename: PhpRename,
    defaults: HashMap<Ident, Lit>,
    optional: Option<Ident>,
    vis: Option<Visibility>,
    attrs: Vec<syn::Attribute>,
    getter: Flag,
    setter: Flag,
    constructor: Flag,
    abstract_method: Flag,
}

impl MethodArgs {
    fn new(name: String, attr: PhpFunctionImplAttribute) -> Self {
        let ty = if name == "__construct" || attr.constructor.is_present() {
            MethodTy::Constructor
        } else if attr.getter.is_present() {
            MethodTy::Getter
        } else if attr.setter.is_present() {
            MethodTy::Setter
        } else if attr.abstract_method.is_present() {
            MethodTy::Abstract
        } else {
            MethodTy::Normal
        };

        Self {
            name,
            optional: attr.optional,
            defaults: attr.defaults,
            vis: attr.vis.unwrap_or(Visibility::Public),
            ty,
        }
    }
}

#[derive(Debug)]
struct ParsedImpl<'a> {
    path: &'a syn::Path,
    rename_methods: RenameRule,
    rename_constants: RenameRule,
    functions: Vec<FnBuilder>,
    constructor: Option<Function<'a>>,
    constants: Vec<Constant<'a>>,
}

#[derive(Debug)]
struct FnBuilder {
    /// Tokens which represent the `FunctionBuilder` for this function.
    pub builder: TokenStream,
    /// The visibility of this method.
    pub vis: Visibility,
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
    fn new(path: &'a syn::Path, rename_methods: RenameRule, rename_constants: RenameRule) -> Self {
        Self {
            path,
            rename_methods,
            rename_constants,
            functions: Vec::default(),
            constructor: Option::default(),
            constants: Vec::default(),
        }
    }

    /// Parses an impl block from `items`, populating `self`.
    fn parse(&mut self, items: impl Iterator<Item = &'a mut syn::ImplItem>) -> Result<()> {
        for items in items {
            match items {
                syn::ImplItem::Const(c) => {
                    let attr = PhpConstAttribute::from_attributes(&c.attrs)?;
                    let name = self.rename_constants.rename(c.ident.to_string());
                    let name = attr.rename.rename(name);
                    let docs = get_docs(&attr.attrs)?;
                    c.attrs.retain(|attr| !attr.path().is_ident("php"));

                    self.constants.push(Constant {
                        name,
                        ident: &c.ident,
                        docs,
                    });
                }
                syn::ImplItem::Fn(method) => {
                    let attr = PhpFunctionImplAttribute::from_attributes(&method.attrs)?;
                    let name = self.rename_methods.rename(method.sig.ident.to_string());
                    let name = attr.rename.rename(name);
                    let docs = get_docs(&attr.attrs)?;
                    method.attrs.retain(|attr| !attr.path().is_ident("php"));

                    let opts = MethodArgs::new(name, attr);
                    let args = Args::parse_from_fnargs(method.sig.inputs.iter(), opts.defaults)?;
                    let mut func = Function::new(&method.sig, opts.name, args, opts.optional, docs);

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
                                .is_some_and(|arg| arg.name == "self_")
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
                        let builder = func.function_builder(call_type);
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
    fn generate_php_class_impl(&self) -> TokenStream {
        let path = &self.path;
        let functions = &self.functions;
        let constructor = self
            .constructor
            .as_ref()
            .map(|func| func.constructor_meta(self.path))
            .option_tokens();
        let constants = self.constants.iter().map(|c| {
            let name = &c.name;
            let ident = c.ident;
            let docs = &c.docs;
            quote! {
                (#name, &#path::#ident, &[#(#docs),*])
            }
        });

        quote! {
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
        }
    }
}

impl quote::ToTokens for FnBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let builder = &self.builder;
        // TODO(cole_d): allow more flags via attributes
        let mut flags = vec![];
        flags.push(match self.vis {
            Visibility::Public => quote! { ::ext_php_rs::flags::MethodFlags::Public },
            Visibility::Protected => quote! { ::ext_php_rs::flags::MethodFlags::Protected },
            Visibility::Private => quote! { ::ext_php_rs::flags::MethodFlags::Private },
        });
        if self.r#abstract {
            flags.push(quote! { ::ext_php_rs::flags::MethodFlags::Abstract });
        }
        quote! {
            (#builder, #(#flags)|*)
        }
        .to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use super::RenameRule;

    #[test]
    fn test_rename_magic() {
        for &(magic, expected) in &[
            ("__construct", "__construct"),
            ("__destruct", "__destruct"),
            ("__call", "__call"),
            ("__call_static", "__callStatic"),
            ("__get", "__get"),
            ("__set", "__set"),
            ("__isset", "__isset"),
            ("__unset", "__unset"),
            ("__sleep", "__sleep"),
            ("__wakeup", "__wakeup"),
            ("__serialize", "__serialize"),
            ("__unserialize", "__unserialize"),
            ("__to_string", "__toString"),
            ("__invoke", "__invoke"),
            ("__set_state", "__set_state"),
            ("__clone", "__clone"),
            ("__debug_info", "__debugInfo"),
        ] {
            assert_eq!(magic, RenameRule::None.rename(magic));
            assert_eq!(expected, RenameRule::Camel.rename(magic));
            assert_eq!(expected, RenameRule::Snake.rename(magic));
        }
    }

    #[test]
    fn test_rename_php_methods() {
        let &(original, camel, snake) = &("get_name", "getName", "get_name");
        assert_eq!(original, RenameRule::None.rename(original));
        assert_eq!(camel, RenameRule::Camel.rename(original));
        assert_eq!(snake, RenameRule::Snake.rename(original));
    }
}
