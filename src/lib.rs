#![feature(proc_macro_span)]

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, Expr};

fn print_type_of<T>(_: &T) -> String {
    format!("{}", std::any::type_name::<T>())
}

#[proc_macro_attribute]
pub fn rdd_debug(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut item: syn::Item = syn::parse(input).unwrap();
    let fn_item = match &mut item {
        syn::Item::Fn(fn_item) => fn_item,
        _ => panic!("expected this attribute on a function got {}", print_type_of(&item))
    };

    let fn_item_clone = fn_item.clone();

    let fn_name = fn_item.sig.ident.to_string();
    let precursor =  attr.to_string();
    let fn_span: proc_macro::Span = fn_item.sig.ident.span().unwrap();
    let file_name = format!("{}", fn_span.source_file().path().display());
    let line_start = fn_span.start().line();

    for (idx, statement) in fn_item_clone.block.stmts.iter().enumerate() {
        match statement {
            syn::Stmt::Expr(expr, out) => {
                match expr {
                    syn::Expr::If(if_extr) => {
                        // let if_token_item = syn::Item::Verbatim(if_extr.if_token.to_token_stream());
                        // let if_file = syn::File {
                        //     attrs: vec![],
                        //     items: vec![if_token_item],
                        //     shebang: None
                        // };
                        // let pretty_if_name = prettyplease::unparse(&if_file);
                        // let in_if_block =  syn::parse(quote!{println!("{}\nfn {}\nline: {}\nfile: {}\n{}\n", #precursor, #pretty_if_name, #line_start, #file_name, #precursor);}.into()).unwrap();
                        let mut new_if = if_extr.clone();
                        let if_line_start =  if_extr.span().start().line;

                        let x: syn::Stmt = syn::parse(quote!{println!("{} if statement at line: {} in file: {}\n", #precursor, #if_line_start, #file_name);}.into()).unwrap();
                        new_if.then_branch.stmts.insert(0, x);
                        
                        let new_else_branch = if let Some(mut else_branch) = new_if.else_branch.clone() {
                            match *else_branch.1.to_owned() {
                                syn::Expr::Block(mut block) => {
                                    let x: syn::Stmt = syn::parse(quote!{println!("{} else statement at line: {} in file: {}\n", #precursor, #if_line_start, #file_name);}.into()).unwrap();
                                    block.block.stmts.insert(0, x);
                                    else_branch.1 = Box::new(syn::Expr::Block(block));
                                    Some(else_branch)
                                }
                                syn::Expr::If(mut if_else) => {
                                    let x: syn::Stmt = syn::parse(quote!{println!("{} if else statement at line: {} in file: {}\n", #precursor, #if_line_start, #file_name);}.into()).unwrap();
                                    if_else.then_branch.stmts.insert(0, x);
                                    else_branch.1 = Box::new(syn::Expr::If(if_else));
                                    Some(else_branch)
                                }
                                _ => None,
                            }
                        } else {
                            None
                        };

                        new_if.else_branch = new_else_branch;
                        println!("{new_if:#?}");

                        let new_if_statment = syn::Stmt::Expr(syn::Expr::If(new_if), out.clone());
                        fn_item.block.stmts.insert(idx, new_if_statment);
                        fn_item.block.stmts.remove(idx+1);
                    }
                    _ => {},
                }
            }
            _ => {},
        }
    }

    let x: syn::Stmt = syn::parse(quote!{println!("{}\nfn {}\nline: {}\nfile: {}\n{}\n", #precursor, #fn_name, #line_start, #file_name, #precursor);}.into()).unwrap();
    fn_item.block.stmts.insert(0,x);
    item.into_token_stream().into()
}