use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Components)]
pub fn components_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("Components Derive: Can't parse derive input!");
    impl_components(&ast)
}

fn impl_components(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let syn::Data::Struct(data_struct) = &ast.data else {
        panic!("Components Derive: Not a data struct!")
    };
    let members = data_struct.fields.members();

    let gen = quote! {
        impl Components for #name {
            fn despawn(&mut self, entity: Entity) {
                #(self.#members.remove(entity);)*
            }
        }
    };
    gen.into()
}
