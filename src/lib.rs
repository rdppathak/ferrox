use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Attribute macro for HTTP methods
/// Usage: #[http_method(GET, "/users")]
/// Generates inventory registration directly
#[proc_macro_attribute]
pub fn http_method(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    // Parse the attribute arguments using simple string processing
    let args_str = args.to_string();
    let args_content = args_str.trim_matches(|c| c == '(' || c == ')').trim();

    let (method_part, path_part) = if let Some(comma_pos) = args_content.find(',') {
        let method_str = args_content[..comma_pos].trim();
        let path_str = args_content[comma_pos + 1..].trim();
        (method_str, Some(path_str))
    } else {
        (args_content, None)
    };

    // Determine method and path
    let method_str = match method_part {
        "GET" | "\"GET\"" => "GET",
        "POST" | "\"POST\"" => "POST",
        "PUT" | "\"PUT\"" => "PUT",
        "PATCH" | "\"PATCH\"" => "PATCH",
        "DELETE" | "\"DELETE\"" => "DELETE",
        "HEAD" | "\"HEAD\"" => "HEAD",
        "OPTIONS" | "\"OPTIONS\"" => "OPTIONS",
        _ => "GET",
    };

    let path_str = path_part.unwrap_or("/").trim_matches('"');

    // Generate inventory registration code directly
    let fn_name = &input_fn.sig.ident;
    let register_code = format!(
        "inventory::submit!(RouteRegistration {{
            method: \"{}\",
            path: \"{}\",
            handler_fn: || std::sync::Arc::new({}),
        }});",
        method_str, path_str, fn_name
    );

    let register_stmt: syn::Stmt = syn::parse_str(&register_code).unwrap();

    // Create new function with constants added at the beginning
    let method_const = format!("const METHOD: &str = \"{}\";", method_str);
    let path_const = format!("const PATH: &str = \"{}\";", path_str);

    let method_const_stmt: syn::Stmt = syn::parse_str(&method_const).unwrap();
    let path_const_stmt: syn::Stmt = syn::parse_str(&path_const).unwrap();

    let mut new_block = input_fn.block.clone();
    new_block.stmts.insert(0, method_const_stmt);
    new_block.stmts.insert(1, path_const_stmt);

    let new_fn = ItemFn {
        attrs: input_fn.attrs,
        vis: input_fn.vis,
        sig: input_fn.sig,
        block: new_block,
    };

    let expanded = quote! {
        #new_fn

        // Automatically register this route via inventory
        #register_stmt
    };

    TokenStream::from(expanded)
}
