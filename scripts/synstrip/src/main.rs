//! Item stripper for Rust source.
//!
//! Modes:
//!   synstrip <file> <marker> [marker...]   drop top-level items whose token
//!                                          stream contains any marker
//!   synstrip --line <file> <line>          drop the smallest top-level (or
//!                                          inline-module) item whose span
//!                                          contains <line>
//! Uses `syn` (real parser). Preserves all text outside dropped items.
use quote::ToTokens;
use syn::{spanned::Spanned, Item};

fn contains_any(item: &Item, markers: &[String]) -> bool {
    let tokens = item.to_token_stream().to_string();
    markers.iter().any(|m| tokens.contains(m.as_str()))
}

/// Find the smallest item (recursing into inline modules) whose span contains `line` (1-based).
fn find_item<'a>(items: &'a [Item], line: usize) -> Option<&'a Item> {
    for it in items {
        let sp = it.span();
        let (s, e) = (sp.start().line, sp.end().line);
        if s != 0 && s <= line && line <= e {
            if let Item::Mod(m) = it {
                if let Some((_, inner)) = &m.content {
                    if let Some(found) = find_item(inner, line) {
                        return Some(found);
                    }
                }
            }
            return Some(it);
        }
    }
    None
}

fn mark_item(remove: &mut [bool], lines: &[&str], item: &Item, dropped: &mut Vec<String>) {
    let sp = item.span();
    let start = sp.start().line;
    let end = sp.end().line;
    if start == 0 || end == 0 {
        eprintln!("WARN: zero span for an item");
        return;
    }
    let mut s = start - 1;
    let e = end - 1;
    while s > 0 {
        let prev = lines[s - 1].trim_start();
        if prev.starts_with("#[")
            || prev.starts_with("#![")
            || prev.starts_with("///")
            || prev.starts_with("//!")
            || prev.starts_with("//")
            || prev.is_empty()
        {
            s -= 1;
        } else {
            break;
        }
    }
    dropped.push(lines[start - 1].trim().chars().take(90).collect());
    for i in s..=e {
        if i < remove.len() {
            remove[i] = true;
        }
    }
}

fn write_out(path: &str, lines: &[&str], remove: &[bool]) {
    let mut out = String::new();
    let mut blank_run = 0usize;
    for (i, line) in lines.iter().enumerate() {
        if remove[i] {
            continue;
        }
        if line.trim().is_empty() {
            blank_run += 1;
            if blank_run > 1 {
                continue;
            }
        } else {
            blank_run = 0;
        }
        out.push_str(line);
        out.push('\n');
    }
    std::fs::write(path, &out).expect("write file");
}


/// Classify the item containing `line`: (kind, name, is_test).
fn classify(item: &Item) -> (String, String, bool) {
    let attrs_test = |attrs: &[syn::Attribute]| {
        attrs.iter().any(|a| {
            let p = a.path();
            if p.is_ident("test") {
                return true;
            }
            // tokio::test / async_std::test etc.
            let segs: Vec<String> = p.segments.iter().map(|s| s.ident.to_string()).collect();
            segs.last().map(|s| s == "test").unwrap_or(false)
        })
    };
    match item {
        Item::Fn(f) => {
            let name = f.sig.ident.to_string();
            let is_test = attrs_test(&f.attrs) || name.starts_with("test_") || name.starts_with("test");
            ("fn".into(), name, is_test)
        }
        Item::Use(_) => ("use".into(), String::new(), false),
        Item::Struct(s) => ("struct".into(), s.ident.to_string(), false),
        Item::Impl(i) => (
            "impl".into(),
            i.self_ty.to_token_stream().to_string(),
            false,
        ),
        Item::Mod(m) => ("mod".into(), m.ident.to_string(), false),
        Item::Enum(e) => ("enum".into(), e.ident.to_string(), false),
        Item::Const(c) => ("const".into(), c.ident.to_string(), false),
        Item::Static(s) => ("static".into(), s.ident.to_string(), false),
        Item::Type(t) => ("type".into(), t.ident.to_string(), false),
        Item::Trait(t) => ("trait".into(), t.ident.to_string(), false),
        _ => ("other".into(), String::new(), false),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: synstrip <file> <marker...> | synstrip --line <file> <line>");
        std::process::exit(2);
    }
    if args[1] == "--line-info" {
        let path = &args[2];
        let line: usize = args[3].parse().expect("line number");
        let src = std::fs::read_to_string(path).expect("read");
        let file = syn::parse_file(&src).expect("parse");
        match find_item(&file.items, line) {
            Some(item) => {
                let (kind, name, is_test) = classify(item);
                println!("kind={} name={} is_test={}", kind, name, is_test);
            }
            None => {
                println!("kind=none name= is_test=false");
                std::process::exit(3);
            }
        }
        return;
    }
    if args[1] == "--line" {
        let path = &args[2];
        let line: usize = args[3].parse().expect("line number");
        let src = std::fs::read_to_string(path).expect("read");
        let file = syn::parse_file(&src).expect("parse");
        let lines: Vec<&str> = src.split('\n').collect();
        let mut remove = vec![false; lines.len()];
        let mut dropped = Vec::new();
        match find_item(&file.items, line) {
            Some(item) => {
                mark_item(&mut remove, &lines, item, &mut dropped);
                write_out(path, &lines, &remove);
                eprintln!("synstrip --line {}: dropped item starting: {}", line, dropped.join(" | "));
            }
            None => {
                eprintln!("synstrip --line {}: no enclosing item found", line);
                std::process::exit(3);
            }
        }
        return;
    }
    let path = &args[1];
    let markers: Vec<String> = args[2..].to_vec();
    let src = std::fs::read_to_string(path).expect("read");
    let file = syn::parse_file(&src).expect("parse");
    let lines: Vec<&str> = src.split('\n').collect();
    let mut remove = vec![false; lines.len()];
    let mut dropped = Vec::new();
    for item in &file.items {
        if contains_any(item, &markers) {
            mark_item(&mut remove, &lines, item, &mut dropped);
        }
    }
    write_out(path, &lines, &remove);
    eprintln!("synstrip: dropped {} items", dropped.len());
}
