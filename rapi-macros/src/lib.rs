use proc_macro::TokenStream;
use quote::ToTokens;
use syn::ItemFn;

#[proc_macro_attribute]
pub fn embed_r(
  _input: TokenStream, annotated_item: TokenStream,
) -> TokenStream {
  let mut item_fn = syn::parse_macro_input!(annotated_item as ItemFn);
  let fn_name = &item_fn.sig.ident;
  let fn_block = &mut item_fn.block;

  fn_block.stmts.insert(
    0,
    syn::parse2(quote::quote!(unsafe {
      Rf_initEmbeddedR(0, [0_i8 as _; 0].as_mut_ptr());
    }))
    .unwrap(),
  );

  fn_block.stmts.insert(
    fn_block.stmts.len() - 1,
    syn::parse2(quote::quote!(unsafe {
      Rf_endEmbeddedR(0);
    }))
    .unwrap(),
  );

  item_fn.into_token_stream().into()
}
