use proc_macro::TokenStream;
use quote::quote;
use syn::ItemFn;

#[proc_macro_attribute]
pub fn embed_r(
  _input: TokenStream, annotated_item: TokenStream,
) -> TokenStream {
  let item_fn = syn::parse_macro_input!(annotated_item as ItemFn);
  let ItemFn { attrs: fn_attrs, vis: fn_vis, sig: fn_sig, block: fn_block } =
    &item_fn;

  let before_code = quote::quote!(unsafe {
    Rf_initEmbeddedR(0, [0_i8 as _; 0].as_mut_ptr());
  });

  let after_code = quote::quote!(unsafe {
    Rf_endEmbeddedR(0);
  });

  let output = quote!(
    #(#fn_attrs)*
    #fn_vis #fn_sig {

      #before_code

      let result = (|| #fn_block )();

      #after_code

      result
    }
  );

  // let output = quote! {
  //     // #(fn_attrs*)
  //     #fn_vis fn #fn_sig {
  //         #before_code
  //         let result = (|| {
  //             #fn_block
  //         })();
  //         #after_code
  //         result
  //     }
  // };

  TokenStream::from(output)
}
