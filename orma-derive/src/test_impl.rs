use proc_macro2::TokenStream;
use quote::quote;

fn maybe_ident(arg: &syn::FnArg) -> Option<&syn::Ident> {
    match arg {
        syn::FnArg::Typed(syn::PatType { pat, .. }) => match pat.as_ref() {
            syn::Pat::Ident(ident) => Some(&ident.ident),
            _ => None,
        },
        _ => None,
    }
}

pub fn test_impl(mut item: syn::ItemFn) -> TokenStream {
    let aa = std::mem::take(&mut item.sig.inputs);
    let vis = &item.vis;
    let sig = &item.sig;
    let block = &item.block;
    let ident = format!("{}", sig.ident);

    let args = aa.iter().filter_map(maybe_ident);

    quote! {
        #[::tokio::test]
        #vis #sig {
            #(
                let #args = #args (#ident);
            )*
            #block
        }
    }
}
