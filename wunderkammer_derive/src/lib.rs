use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(ComponentSet)]
pub fn component_set_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).expect("Components Derive: Can't parse derive input!");
    impl_component_set(&ast)
}

fn impl_component_set(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let syn::Data::Struct(data_struct) = &ast.data else {
        panic!("Components Derive: Not a data struct!")
    };
    let members = data_struct.fields.members();

    let gen = quote! {
        impl ComponentSet for #name {
            fn despawn(&mut self, entity: Entity) {
                #(self.#members.remove(entity);)*
            }
        }
    };
    gen.into()
}
