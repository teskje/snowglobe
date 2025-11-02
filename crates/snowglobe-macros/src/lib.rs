use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn scene(_args: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let name_ident = &func.sig.ident;

    let expanded = quote! {
        #func

        const _: () = {
            use ::snowglobe::cli::__private::{SCENES, Scene, linkme};

            #[linkme::distributed_slice(SCENES)]
            #[linkme(crate = linkme)]
            static SCENE: Scene = Scene {
                module: module_path!(),
                name: stringify!(#name_ident),
                func: #name_ident,
            };
        };
    };

    expanded.into()
}
