#![feature(proc_macro_span)]

use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{ExprIf, ExprMatch, Stmt, spanned::Spanned};

fn print_type_of<T>(_: &T) -> String {
    format!("{}", std::any::type_name::<T>())
}

fn check_statements(statements: Vec<Stmt>, precursor: &str, file_name: &str) -> Vec<Stmt> {
    let mut statement_crawler = statements.clone();

    for (idx, statement) in statements.iter().enumerate() {
        match statement {
            syn::Stmt::Expr(expr, out) => match expr {
                syn::Expr::If(if_extr) => {
                    let new_if_statement = syn::Stmt::Expr(
                        syn::Expr::If(crawl_if_branches(if_extr, precursor, file_name, false)),
                        *out,
                    );
                    statement_crawler.insert(idx, new_if_statement);
                    statement_crawler.remove(idx + 1);
                }
                syn::Expr::Match(match_extr) => {
                    crawl_match_arms(match_extr, precursor, file_name);
                }
                _ => {}
            },
            _ => {}
        }
    }

    statement_crawler
}

fn crawl_if_branches(if_extr: &ExprIf, precursor: &str, file_name: &str, recurse: bool) -> ExprIf {
    let mut new_if = if_extr.clone();
    let if_line_start = if_extr.span().start().line;

    let print_if = if recurse { "if else" } else { "if" };

    let new_stmt = check_statements(new_if.then_branch.stmts, precursor, file_name);
    new_if.then_branch.stmts = new_stmt;
    let x: syn::Stmt = syn::parse(quote!{println!("{}{} statement at line: {} in file: {}\n", #precursor, #print_if, #if_line_start, #file_name);}.into()).unwrap();
    new_if.then_branch.stmts.insert(0, x);

    let new_else_branch = if let Some(mut else_branch) = new_if.else_branch.clone() {
        match *else_branch.1.to_owned() {
            syn::Expr::Block(block) => {
                let mut new_block = block.clone();
                let line_number = else_branch.1.span().start().line;
                let new_stmt = check_statements(new_block.block.stmts, precursor, file_name);
                new_block.block.stmts = new_stmt;
                let x: syn::Stmt = syn::parse(quote!{println!("{}else statement at line: {} in file: {}\n", #precursor, #line_number, #file_name);}.into()).unwrap();
                new_block.block.stmts.insert(0, x);
                else_branch.1 = Box::new(syn::Expr::Block(new_block));
                Some(else_branch)
            }
            syn::Expr::If(mut if_else) => {
                let if_else_expr = crawl_if_branches(&if_else, precursor, file_name, true);
                else_branch.1 = Box::new(syn::Expr::If(if_else_expr));
                Some(else_branch)
            }
            _ => Some(else_branch),
        }
    } else {
        None
    };

    new_if.else_branch = new_else_branch;

    new_if
}

fn crawl_match_arms(match_expr: &ExprMatch, precursor: &str, file_name: &str) -> ExprMatch {
    let mut new_match = match_expr.clone();

    //println!("{match_expr:#?}");
    //we print the first match expression before inspecting the arms
    for (idx, arm) in match_expr.arms.iter().enumerate() {
        // print which arm we are in (guard needs work)
        println!(
            "{}{}",
            arm.pat.to_token_stream().to_string(),
            arm.fat_arrow_token.to_token_stream().to_string()
        );
    }

    new_match
}

#[proc_macro_attribute]
pub fn print_debug(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut item: syn::Item = syn::parse(input).unwrap();
    let fn_item = match &mut item {
        syn::Item::Fn(fn_item) => fn_item,
        _ => panic!(
            "expected this attribute on a function got {}",
            print_type_of(&item)
        ),
    };

    let fn_item_clone = fn_item.clone();

    let fn_name = fn_item.sig.ident.to_string();
    let precursor = attr.to_string();
    let fn_span: proc_macro::Span = fn_item.sig.ident.span().unwrap();
    let file_name = format!("{}", fn_span.source_file().path().display());
    let line_start = fn_span.start().line();

    let new_stmt = check_statements(fn_item_clone.block.stmts, &precursor, &file_name);
    fn_item.block.stmts = new_stmt;

    let x: syn::Stmt = syn::parse(quote!{println!("{}\nfn {}\nline: {}\nfile: {}\n{}\n", #precursor, #fn_name, #line_start, #file_name, #precursor);}.into()).unwrap();
    fn_item.block.stmts.insert(0, x);
    item.into_token_stream().into()
}
