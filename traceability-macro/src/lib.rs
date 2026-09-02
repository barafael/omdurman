use proc_macro::TokenStream;
use syn::{ItemFn, LitStr, parse_macro_input};

/// `#[rulebook("§6.3", "§6.42")]` marks a `#[test]` function as covering
/// specific rulebook sections.
///
/// The attribute emits the item unchanged but also attaches a rustdoc comment
/// (via `#[doc = "..."]`) listing the covered sections, so the traceability
/// info shows up in `cargo doc` output. Test discovery is done by source
/// scanning in `tools/traceability-lsp` (`scan_test_entries`), which matches
/// the `#[rulebook("§...")]` text directly -- nothing is written to disk at
/// compile time. (An earlier revision appended entries to
/// `target/rulebook_entries.jsonl`; that file was never consumed because
/// source scanning is build-order independent, so the writer was removed.)
#[proc_macro_attribute]
pub fn rulebook(attr: TokenStream, item: TokenStream) -> TokenStream {
    let sections = parse_macro_input!(
        attr with syn::punctuated::Punctuated::<LitStr, syn::Token![,]>::parse_terminated
    );
    let func = parse_macro_input!(item as ItemFn);
    let section_list: Vec<String> = sections.iter().map(|lit| lit.value()).collect();
    let doc = format!("**Rulebook sections covered:** {}", section_list.join(", "));
    TokenStream::from(quote::quote! {
        #[doc = #doc]
        #func
    })
}
