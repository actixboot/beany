use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, FieldsNamed};

struct Field {
  ident: syn::Ident,
  ty: syn::Type,
}

#[proc_macro_derive(Bean)]
pub fn derive_bean(input: TokenStream) -> TokenStream {
  impl_derive_bean(parse_macro_input!(input as DeriveInput))
    .unwrap_or_else(|err| err.to_compile_error().into())
}

fn impl_derive_bean(input: DeriveInput) -> syn::Result<TokenStream> {
  let ident = &input.ident;
  let fields = get_fields(&input)?
    .iter()
    .map(|field| {
      let field_ident = &field.ident;
      let field_ty = &field.ty;

      if is_arc_type(field_ty) {
        quote! {
            #field_ident: context.get()
        }
      } else {
        quote! {
            #field_ident: (*context.get::<#field_ty>()).clone()
        }
      }
    })
    .collect::<Vec<_>>();

  Ok(
    quote! {
        impl beany::Bean for #ident {
            fn create(context: &beany::BeansContext) -> Self {
                Self {
                    #(#fields),*
                }
            }
        }
    }
    .into(),
  )
}

fn is_arc_type(ty: &syn::Type) -> bool {
  if let syn::Type::Path(type_path) = ty {
    if let Some(segment) = type_path.path.segments.last() {
      return segment.ident == "Arc";
    }
  }
  false
}

fn get_fields(input: &DeriveInput) -> syn::Result<Vec<Field>> {
  let Data::Struct(DataStruct { ref fields, .. }) = input.data else {
    return Err(syn::Error::new_spanned(
      input,
      "Bean can only be used on structs",
    ));
  };

  let Fields::Named(FieldsNamed { named, .. }) = fields else {
    return Ok(vec![]);
  };

  Ok(
    named
      .iter()
      .map(|field| Field {
        ident: field.ident.clone().unwrap(),
        ty: field.ty.clone(),
      })
      .collect(),
  )
}
