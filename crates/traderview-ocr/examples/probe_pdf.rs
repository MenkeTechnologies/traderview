use lopdf::Document;
use std::env;

fn main() {
    let path = env::args().nth(1).expect("pdf path");
    let doc = Document::load(&path).expect("load pdf");
    let pages = doc.get_pages();
    let total = pages.len() as u32;
    println!("== {} pages ==", total);
    for p in 1..=total {
        match doc.extract_text(&[p]) {
            Ok(t) => {
                println!("--- page {p} ---");
                for line in t.lines().take(80) { println!("{line}"); }
            }
            Err(e) => println!("page {p} err: {e}"),
        }
    }
}
