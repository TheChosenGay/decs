use proc_macro::TokenStream;

#[proc_macro_derive(Component)]
pub fn component_macro(input: TokenStream) -> TokenStream {
   let ast = syn::parse_macro_input!(input as syn::DeriveInput);
   handle_component(ast)
}

fn handle_component(ast: syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generated = quote::quote! {
        impl Component for #name {
        }
    };
    generated.into()
}
