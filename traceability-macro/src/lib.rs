use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn, LitStr};

/// `#[rulebook("§6.3", "§6.42")]` marks a `#[test]` function as covering
/// specific rulebook sections.  The entry is appended to
/// `target/rulebook_entries.jsonl` so the traceability test can discover
/// annotated tests without source scanning.
#[proc_macro_attribute]
pub fn rulebook(attr: TokenStream, item: TokenStream) -> TokenStream {
    let sections = parse_macro_input!(attr with syn::punctuated::Punctuated::<LitStr, syn::Token![,]>::parse_terminated);
    let func = parse_macro_input!(item as ItemFn);
    let fn_name = &func.sig.ident;
    let fn_name_str = fn_name.to_string();

    let section_strs: Vec<String> = sections.iter().map(|s| s.value()).collect();

    if let Err(e) = write_entry(&fn_name_str, &section_strs) {
        eprintln!("cargo:warning=rulebook: failed to write entry for {}: {}", fn_name_str, e);
    }

    TokenStream::from(quote::quote! { #func })
}

fn write_entry(test_name: &str, sections: &[String]) -> std::io::Result<()> {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let workspace_root = manifest_dir.parent().unwrap_or(&manifest_dir);
    let path = workspace_root.join("target/rulebook_entries.jsonl");

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;

    write_json_line(&mut file, test_name, sections)?;
    writeln!(file)?;

    Ok(())
}

fn write_json_line(
    w: &mut impl Write,
    test_name: &str,
    sections: &[String],
) -> std::io::Result<()> {
    write!(w, "{{")?;
    write!(w, "\"test_name\":")?;
    write_json_string(w, test_name)?;
    write!(w, ",\"sections\":[")?;
    for (i, s) in sections.iter().enumerate() {
        if i > 0 {
            write!(w, ",")?;
        }
        write_json_string(w, s)?;
    }
    write!(w, "]}}")
}

fn write_json_string(w: &mut impl Write, s: &str) -> std::io::Result<()> {
    write!(w, "\"")?;
    for c in s.chars() {
        match c {
            '"' => write!(w, "\\\"")?,
            '\\' => write!(w, "\\\\")?,
            '\n' => write!(w, "\\n")?,
            '\r' => write!(w, "\\r")?,
            '\t' => write!(w, "\\t")?,
            c if c < '\x20' => write!(w, "\\u{:04x}", c as u32)?,
            c => write!(w, "{}", c)?,
        }
    }
    write!(w, "\"")
}
