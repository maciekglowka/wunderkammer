use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemType, Type, TypeTuple};

#[proc_macro_attribute]
pub fn storage(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let item_type = parse_macro_input!(input as ItemType);

    let storage_name = &item_type.ident;

    let component_types = match *item_type.ty {
        Type::Tuple(TypeTuple { elems, .. }) => elems,
        _ => panic!("expected type tuple for a component list"),
    };

    let mut component_fields = vec![];
    let mut component_names = vec![];
    let mut handler_impls = vec![];

    for (i, ty) in component_types.iter().enumerate() {
        let name = format_ident!("component_{i}");
        let mask = 1u128 << i;

        component_fields.push(quote! {
            pub #name: ComponentStorage<#ty>,
        });
        component_names.push(quote! {
            #name
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

    let serialize_stmt = if cfg!(feature = "serialize") {
        let h = quote!(#);
        Some(quote! {
            #h [derive(serde::Serialize, serde::Deserialize)]
        })
    } else {
        None
    };

    let gen = quote! {
        type #storage_name = wunderkammer::storage::Storage<wunderkammer_private::Components>;

        mod wunderkammer_private {

            use super::*;

            use wunderkammer::storage::{Storage, ComponentHandler, ComponentStorage};

            #[derive(Default)]
            #serialize_stmt
            pub struct Components {
                #(#component_fields)*
            }
            impl wunderkammer::storage::ComponentSet for Components {
                fn drop_all_components(&mut self, entity: &wunderkammer::Entity) {
                    let _ = #(self.#component_names.remove(entity);)*
                }
            }

            #(#handler_impls)*
        }
    };

    gen.into()
}
