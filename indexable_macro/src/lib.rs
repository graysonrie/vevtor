use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident, Meta, NestedMeta, Type};

#[proc_macro_derive(Indexable, attributes(indexable))]
pub fn derive_indexable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;
    let mut id_field = None;
    let mut collection_field = None;
    let mut embed_field = None;

    // Parse attributes under `indexable`
    for attr in input.attrs.iter() {
        if let Ok(meta) = attr.parse_meta() {
            if let Meta::List(meta_list) = meta {
                if meta_list.path.is_ident("indexable") {
                    for nested_meta in meta_list.nested.iter() {
                        if let NestedMeta::Meta(Meta::NameValue(meta_name_value)) = nested_meta {
                            if let Some(ident) = meta_name_value.path.get_ident() {
                                match ident.to_string().as_str() {
                                    "id_field" => {
                                        if let syn::Lit::Str(lit_str) = &meta_name_value.lit {
                                            id_field =
                                                Some(Ident::new(&lit_str.value(), lit_str.span()));
                                        }
                                    }
                                    "collection_field" => {
                                        if let syn::Lit::Str(lit_str) = &meta_name_value.lit {
                                            collection_field =
                                                Some(Ident::new(&lit_str.value(), lit_str.span()));
                                        }
                                    }
                                    "embed_field" => {
                                        if let syn::Lit::Str(lit_str) = &meta_name_value.lit {
                                            embed_field =
                                                Some(Ident::new(&lit_str.value(), lit_str.span()));
                                        }
                                    }
                                    _ => (),
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Ensure required fields are provided
    let id_field = id_field.expect("id_field attribute is required.");
    let collection_field = collection_field.expect("collection_field attribute is required.");
    let embed_field = embed_field.expect("embed_field attribute is required.");

    // Check the type of id_field by examining struct fields
    let mut id_is_string = false;
    if let syn::Data::Struct(data_struct) = &input.data {
        for field in &data_struct.fields {
            if let Some(field_ident) = &field.ident {
                if field_ident == &id_field {
                    if let Type::Path(type_path) = &field.ty {
                        if type_path.path.segments.last().unwrap().ident == "String" {
                            id_is_string = true;
                        }
                    }
                }
            }
        }
    }

    // Conditional logic in get_id based on type of id_field
    let get_id_impl = if id_is_string {
        quote! {
            fn get_id(&self) -> u64 {
                string_to_u64(&self.#id_field)
            }
        }
    } else {
        quote! {
            fn get_id(&self) -> u64 {
                self.#id_field
            }
        }
    };

    let expanded = quote! {
        use std::collections::HashMap;
        use ::vevtor::qdrant_client::{qdrant::Value, Payload};  // Fully qualify qdrant_client through main crate
        use ::vevtor::twox_hash::XxHash64;                      // Fully qualify twox_hash

        impl Indexable for #struct_name {
            fn as_map(&self) -> HashMap<String, Value> {
                let value: serde_json::Value = serde_json::to_value(self).expect("Serialization failed");

                if let serde_json::Value::Object(map) = value {
                    map.into_iter().map(|(k, v)| (k, v.into())).collect()
                } else {
                    HashMap::new()
                }
            }

            #get_id_impl

            fn embed_label(&self) -> &str {
                &self.#embed_field
            }

            fn collection(&self) -> String {
                self.#collection_field.to_string()
            }

            fn from_qdrant_payload(payload: &HashMap<String, Value>) -> Result<Self, String> {
                let json_value: serde_json::Value = serde_json::to_value(payload)
                    .map_err(|err| format!("Failed to convert payload to json {}", err))?;

                let model: #struct_name = serde_json::from_value(json_value)
                    .map_err(|err| format!("Failed to convert JSON to Indexable {}", err))?;

                Ok(model)
            }
        }

        impl From<#struct_name> for Payload {
            fn from(val: #struct_name) -> Self {
                Payload::from(val.as_map())
            }
        }

        impl From<Payload> for #struct_name {
            fn from(payload: Payload) -> Self {
                let json_value: serde_json::Value =
                    serde_json::to_value(payload).expect("Failed to convert payload to JSON");

                let model: #struct_name =
                    serde_json::from_value(json_value).expect("Failed to convert JSON to Indexable");

                model
            }
        }

        impl From<HashMap<String, Value>> for #struct_name {
            fn from(payload: HashMap<String, Value>) -> Self {
                let json_value: serde_json::Value =
                    serde_json::to_value(payload).expect("Failed to convert payload to JSON");

                let model: #struct_name =
                    serde_json::from_value(json_value).expect("Failed to convert JSON to Indexable");

                model
            }
        }
        use std::hash::{Hash, Hasher};
        fn string_to_u64(s: &str) -> u64 {
            let mut hasher = XxHash64::default();
            s.hash(&mut hasher);
            hasher.finish()
        }
    };

    TokenStream::from(expanded)
}

// Helper function for ID hashing
