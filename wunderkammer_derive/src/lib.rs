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
    let members_despawn = data_struct.fields.members();
    let members_entities = data_struct.fields.members();

    let gen = quote! {
        impl ComponentSet for #name {
            fn despawn(&mut self, entity: Entity) {
                #(self.#members_despawn.remove(entity);)*
            }

            #[cfg(feature = "string")]
            fn entities_str(&self, component: &str) -> std::collections::HashSet<Entity> {
                match component {
                    #(stringify!(#members_entities) => self.#members_entities.entities(),)*
                    _ => std::collections::HashSet::new()
                }
            }
        }
    };
    gen.into()
}
