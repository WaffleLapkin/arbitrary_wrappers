//! Crate for converting a `impl Wrapper<Type> { ... }` into a `impl Type { ...
//! }` + arbitrary self types feature used.
//!
//! See [`use_ast`] for more.
//!
//! [`use_ast`]: crate::use_ast

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{
    parse::{Parse, ParseStream},
    FnArg, Ident, ImplItem, ImplItemMethod, Pat, PatIdent, PatType, Receiver, Signature, Token,
    TypeReference,
};

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}

/// Macro for converting inherent impls for wrappers into inherent impls for
/// types, but with `arbitrary self types`.
///
/// ## Examples
///
/// This:
/// ```
/// # #![feature(arbitrary_self_types)]
/// struct Wrapper<T>(T);
///
/// impl<T> core::ops::Deref for Wrapper<T> {
///     type Target = T;
///     /* ... */
///     # fn deref(&self) -> &Self::Target { &self.0 }
/// }
///
/// struct Type;
/// struct ReturnTy;
///
/// #[arbitrary_wrappers::use_ast]
/// impl Wrapper<Type> {
///     fn method_a(self) -> ReturnTy {
///         /* body */
///         # ReturnTy
///     }
///
///     fn method_b(&mut self) -> ReturnTy {
///         /* body */
///         # ReturnTy
///     }
/// }
/// ```
///
/// This will become:
///
/// ```
/// #![feature(arbitrary_self_types)]
/// // macro can't enable features, so don't forget to add smt like
/// // `#![feature(...)]` to crate root
/// # struct Wrapper<T>(T);
/// #
/// # impl<T> core::ops::Deref for Wrapper<T> {
/// #    type Target = T;
///     # fn deref(&self) -> &Self::Target { &self.0 }
/// # }
/// #
/// # struct Type;
/// # struct ReturnTy;
/// /* ... */
/// impl Type {
///     fn method_a(self: Wrapper<Type>) -> ReturnTy {
///         /* body */
///         # ReturnTy
///     }
///
///     fn method_b(self: &mut Wrapper<Type>) -> ReturnTy {
///         /* body */
///         # ReturnTy
///     }
/// }
/// ```
///
/// You can also use this macro with [`cfg_attr`] to achieve switching between
/// nigthly-only `arbitrary_self_types` and common impls (this is also useful if
/// you want the methods to appear in type's docs, not the wrapper's)
///
/// [`cfg_attr`]: https://doc.rust-lang.org/reference/conditional-compilation.html#the-cfg_attr-attribute
#[proc_macro_attribute]
pub fn use_ast(_args: TokenStream, input: TokenStream) -> TokenStream {
    let syn::ItemImpl {
        attrs,
        defaultness: _, // inherent impl can't be default
        unsafety: _,    // nor can it be unsafe
        impl_token: _,  // don't need this
        generics,
        trait_,
        self_ty: full_ty,
        brace_token: _, // no, really I don't need any tokens, I have my own tokens
        items,
    } = syn::parse_macro_input!(input as syn::ItemImpl);

    if generics.type_params().count() != 0 || generics.lifetimes().count() != 0 {
        unimplemented!(concat!(
            "Generics and lifetimes are currently not supported (yet?). Please fill an issue at ",
            "`https://github.com/wafflelapkin/arbitrary_wrappers` describing your use case."
        ));
    }

    if trait_.is_some() {
        // `Option::expect_none` is unstable >:(
        panic!(
            "Expected inherent impl, found trait impl. This macro works only for inherent impls, \
             not trait impls."
        );
    }

    let items = items.iter().cloned().map(|it| match it {
         ImplItem::Method(
             ImplItemMethod {
                 attrs,
                 vis,
                 defaultness,
                 sig: Signature {
                     constness,
                     asyncness,
                     unsafety,
                     abi,
                     fn_token,
                     ident,
                     generics,
                     paren_token: _,
                     mut inputs,
                     variadic: _, // variadics are only for foreign functions
                     output
                 },
                 block
             }
         ) => {
             let slf = inputs
                 .first_mut()
                 .expect(
                     "Expected non-static method, found method with 0 arguments. Consider moving this function to another impl block."
                 );
             take_mut::take(slf, |slf| match slf {
                 FnArg::Receiver(
                     Receiver {
                         attrs,
                         reference,
                         mutability,
                         self_token
                     }
                 ) => {
                     let mutab = if reference.is_none() { mutability } else { None };
                     let ty = match reference {
                         None => full_ty.clone(),
                         Some((_, lifetime)) => Box::new(syn::Type::Reference(TypeReference {
                             and_token: Token![&](Span::call_site()), // TODO
                             lifetime,
                             mutability,
                             elem: full_ty.clone(),
                         }))
                     };

                     FnArg::Typed(PatType {
                         attrs,
                         pat: Box::new(Pat::Ident(PatIdent {
                             attrs: vec![],
                             by_ref: None,
                             mutability: mutab,
                             ident: Ident::new("self", self_token.span),
                             subpat: None,
                         })),
                         colon_token: Token![:](Span::call_site()), // TODO
                         ty,
                     })
                 },
                 FnArg::Typed(
                     PatType {
                         attrs: _,
                         pat: _,
                         colon_token: _,
                         ty: _,
                     }
                 ) => {
                     // TODO: Support `self: Box<_>`-like methods
                     panic!("all methods should be non-static")
                 }
             });

             let inputs = inputs.into_iter();
             quote::quote! {
                #( #attrs )*
                #vis #defaultness #constness #asyncness #unsafety #abi #fn_token #ident #generics ( #( #inputs )* ) #output #block
             }
         },
         item => {
             panic!(
                 "only methods are supported, consider moving {item:?} to another impl block",
                 item = item,
             );
         }
    });

    let Wrapped {
        wrapped: self_ty, ..
    } = syn::parse2(quote::quote! { #full_ty }).unwrap();

    let res = quote::quote! {
        #( #attrs )*
        impl #self_ty {
            #( #items )*
        }
    };

    res.into()
}

#[allow(dead_code)] // for never-used fields
struct Wrapped {
    wrapper: Ident,
    a: Token![<],
    wrapped: Ident,
    b: Token![>],
}

impl Parse for Wrapped {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Wrapped {
            wrapper: input.parse()?,
            a: input.parse()?,
            wrapped: input.parse()?,
            b: input.parse()?,
        })
    }
}
