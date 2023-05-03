use proc_macro::TokenStream;
use quote::quote;

//TODO: Note this is not suited for `[test]` yet.
#[proc_macro_attribute]
pub fn embed_r(
  _input: TokenStream, annotated_item: TokenStream,
) -> TokenStream {
  let item_fn = syn::parse_macro_input!(annotated_item as syn::ItemFn);
  let syn::ItemFn {
    attrs: fn_attrs,
    vis: fn_vis,
    sig: fn_sig,
    block: fn_block,
  } = &item_fn;

  let before_code = quote::quote!(unsafe {
    rsys::Rf_initEmbeddedR(0, [0_i8 as _; 0].as_mut_ptr());
  });

  let after_code = quote::quote!(unsafe {
    rsys::Rf_endEmbeddedR(0);
  });

  let output = quote!(
    #(#fn_attrs)*
    #fn_vis #fn_sig {

      #before_code

      let result = std::panic::catch_unwind(|| {
        (|| #fn_block )()
      });

      #after_code

      result.unwrap()
    }
  );

  output.into()
}
