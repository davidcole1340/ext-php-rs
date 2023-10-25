use std::collections::HashMap;

use crate::{
    function::{Args, CallType, Function, MethodReceiver},
    prelude::*,
};
use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{AttributeArgs, Ident, Lit};

pub fn parser(args: AttributeArgs, mut input: syn::ItemImpl) -> Result<TokenStream> {
    let args = match ImplArgs::from_list(&args) {
        Ok(args) => args,
        Err(e) => bail!("Failed to parse impl attribute arguments: {:?}", e),
    };
    let path = match &*input.self_ty {
        syn::Type::Path(ty) => &ty.path,
        _ => bail!(input.self_ty => "The `#[php_impl]` attribute is only valid on structs."),
    };

    let mut parsed = ParsedImpl::new(path, args.rename_methods);
    parsed.parse(input.items.iter_mut())?;

    let php_class_impl = parsed.generate_php_class_impl()?;
    Ok(quote::quote! {
        #input
        #php_class_impl
    })
}

/// Attribute arguments for `impl` blocks.
#[derive(Debug, Default, FromMeta)]
#[darling(default)]
pub struct ImplArgs {
    /// How the methods are renamed.
    rename_methods: RenameRule,
}

/// Different types of rename rules for methods.
#[derive(Debug, Default, Copy, Clone, FromMeta)]
pub enum RenameRule {
    /// Methods won't be renamed.
    #[darling(rename = "none")]
    None,
    /// Methods will be conveted to camelCase.
    #[darling(rename = "camelCase")]
    #[default]
    Camel,
    /// Methods will be converted to snake_case.
    #[darling(rename = "snake_case")]
    Snake,
}

impl RenameRule {
    /// Change case of an identifier.
    ///
    /// Magic methods are handled specially to make sure they're always cased
    /// correctly.
    pub fn rename(&self, name: String) -> String {
        match self {
            RenameRule::None => name,
            rule => match name.as_str() {
                "__construct" => "__construct".to_string(),
                "__destruct" => "__destruct".to_string(),
                "__call" => "__call".to_string(),
                "__call_static" => "__callStatic".to_string(),
                "__get" => "__get".to_string(),
                "__set" => "__set".to_string(),
                "__isset" => "__isset".to_string(),
                "__unset" => "__unset".to_string(),
                "__sleep" => "__sleep".to_string(),
                "__wakeup" => "__wakeup".to_string(),
                "__serialize" => "__serialize".to_string(),
                "__unserialize" => "__unserialize".to_string(),
                "__to_string" => "__toString".to_string(),
                "__invoke" => "__invoke".to_string(),
                "__set_state" => "__set_state".to_string(),
                "__clone" => "__clone".to_string(),
                "__debug_info" => "__debugInfo".to_string(),
                field => match rule {
                    Self::Camel => ident_case::RenameRule::CamelCase.apply_to_field(field),
                    Self::Snake => ident_case::RenameRule::SnakeCase.apply_to_field(field),
                    Self::None => unreachable!(),
                },
            },
        }
    }
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
}

impl MethodArgs {
    fn parse(&mut self, attrs: &mut Vec<syn::Attribute>) -> Result<()> {
        let mut unparsed = vec![];
        unparsed.append(attrs);
        for attr in unparsed {
            if attr.path.is_ident("optional") {
                if self.optional.is_some() {
                    bail!(attr => "Only one `#[optional]` attribute is valid per method.");
                }
                let optional = attr.parse_args().map_err(
                    |e| err!(attr => "Invalid arguments passed to `#[optional]` attribute. {}", e),
                )?;
                self.optional = Some(optional);
            } else if attr.path.is_ident("defaults") {
                let meta = attr
                    .parse_meta()
                    .map_err(|e| err!(attr => "Failed to parse metadata from attribute. {}", e))?;
                let defaults = HashMap::from_meta(&meta).map_err(
                    |e| err!(attr => "Invalid arguments passed to `#[defaults]` attribute. {}", e),
                )?;
                self.defaults = defaults;
            } else if attr.path.is_ident("public") {
                self.vis = MethodVis::Public;
            } else if attr.path.is_ident("protected") {
                self.vis = MethodVis::Protected;
            } else if attr.path.is_ident("private") {
                self.vis = MethodVis::Private;
            } else if attr.path.is_ident("rename") {
                let lit: syn::Lit = attr.parse_args().map_err(|e| err!(attr => "Invalid arguments passed to the `#[rename]` attribute. {}", e))?;
                match lit {
                    Lit::Str(name) => self.name = name.value(),
                    _ => bail!(attr => "Only strings are valid method names."),
                };
            } else if attr.path.is_ident("getter") {
                self.ty = MethodTy::Getter;
            } else if attr.path.is_ident("setter") {
                self.ty = MethodTy::Setter;
            } else if attr.path.is_ident("constructor") {
                self.ty = MethodTy::Constructor;
            } else if attr.path.is_ident("abstract_method") {
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
                        if attr.path.is_ident("rename") {
                            let lit: syn::Lit = attr.parse_args().map_err(|e| err!(attr => "Invalid arguments passed to the `#[rename]` attribute. {}", e))?;
                            match lit {
                                Lit::Str(str) => name = Some(str.value()),
                                _ => bail!(attr => "Only strings are valid constant names."),
                            };
                        } else {
                            c.attrs.push(attr);
                        }
                    }

                    self.constants.push(Constant {
                        name: name.unwrap_or_else(|| c.ident.to_string()),
                        ident: &c.ident,
                    });
                }
                syn::ImplItem::Method(method) => {
                    let name = self.rename.rename(method.sig.ident.to_string());
                    let mut opts = MethodArgs::new(name);
                    opts.parse(&mut method.attrs)?;

                    let args = Args::parse_from_fnargs(method.sig.inputs.iter(), opts.defaults)?;
                    let mut func =
                        Function::parse(&method.sig, Some(opts.name), args, opts.optional)?;

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
            quote! {
                (#name, &#path::#ident)
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

                fn get_constants(self) -> &'static [(&'static str, &'static dyn ::ext_php_rs::convert::IntoZvalDyn)] {
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
