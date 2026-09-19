use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::Parser, parse_macro_input, punctuated::Punctuated, ItemType, Token, Type, TypeTuple,
};

#[proc_macro_attribute]
pub fn storage(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let item_type = parse_macro_input!(input as ItemType);

    let storage_name = &item_type.ident;

    let component_types = match *item_type.ty {
        Type::Tuple(TypeTuple { elems, .. }) => elems,
        _ => panic!("expected type tuple for a component list"),
    };

    let mut component_fields = vec![];
    let mut handler_impls = vec![];

    for (i, ty) in component_types.iter().enumerate() {
        let name = format_ident!("component_{i}");
        let mask = 1u128 << i;

        component_fields.push(quote! {
            pub #name: ComponentStorage<#ty>,
        });
        handler_impls.push(quote! {
            unsafe impl ComponentHandler<#ty> for Components {
                const MASK: u128 = #mask;

                fn storage(&self) -> &ComponentStorage<#ty> {
                    &self.#name
                }
                fn storage_mut(&mut self) -> &mut ComponentStorage<#ty> {
                    &mut self.#name
                }
                unsafe fn storage_raw(c: *mut Self) -> *mut ComponentStorage<#ty> {
                    &raw mut (*c).#name
                }
            }
        });
    }

    let gen = quote! {
        type #storage_name = wunderkammer::Storage<wunderkammer_private::Components>;

        mod wunderkammer_private {

            use wunderkammer::{Storage, ComponentHandler, ComponentStorage};

            #[derive(Default)]
            pub struct Components {
                #(#component_fields)*
            }

            #(#handler_impls)*
        }
    };

    gen.into()
}

#[proc_macro]
pub fn _storage(input: TokenStream) -> TokenStream {
    let parser = Punctuated::<Type, Token![,]>::parse_terminated;
    let component_types = parser.parse(input).unwrap();

    let mut component_fields = vec![];
    let mut handler_impls = vec![];

    for (i, ty) in component_types.iter().enumerate() {
        let name = format_ident!("component_{i}");
        let mask = 1u128 << i;

        component_fields.push(quote! {
            pub #name: #ty,
        });
        handler_impls.push(quote! {
            impl ComponentHandler<#ty> for Components {
                const MASK: u128 = #mask;

                fn storage(&self) -> &ComponentStorage<#ty> {
                    &self.#name
                }
                fn storage_mut(&mut self) -> &mut ComponentStorage<#ty> {
                    &mut self.#name
                }
                fn storage_raw(c: *mut self) -> *mut ComponentStorage<#ty> {
                    &raw mut self.#name
                }
            }
        });
    }

    let gen = quote! {
        wunderkammer::Storage<wunderkammer_private::Components>;

        mod wunderkammer_private {
            #[derive(Default)]
            pub struct Components {
                #(#component_fields)*
            }

            #(#handler_impls)*
        }
    };

    gen.into()
}

// #[proc_macro_derive(ComponentSet)]
// pub fn component_set_derive(input: TokenStream) -> TokenStream {
//     let ast = syn::parse(input).expect("Components Derive: Can't parse derive
// input!");     impl_component_set(&ast)
// }

// fn impl_component_set(ast: &syn::DeriveInput) -> TokenStream {
//     let name = &ast.ident;

//     let syn::Data::Struct(data_struct) = &ast.data else {
//         panic!("Components Derive: Not a data struct!")
//     };
//     let members_despawn = data_struct.fields.members();
//     let members_entities = data_struct.fields.members();

//     let gen = quote! {
//         impl ComponentSet for #name {
//             fn remove_all_components(&mut self, entity: Entity) {
//                 #(self.#members_despawn.remove(entity);)*
//             }

//             fn entities_str(&self, component: &str) -> Vec<&Entity> {
//                 match component {
//                     #(stringify!(#members_entities) =>
// self.#members_entities.entities().collect(),)*                     _ =>
// Vec::new()                 }
//             }
//         }
//     };
//     gen.into()
// }
