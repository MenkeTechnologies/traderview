use traderview_expense::{
    amazon::AmazonParser, apple::AppleCardParser, boa::BofaParser, chase::ChaseParser, Parser,
};

fn run(label: &str, parser: &dyn Parser, path: &str) {
    println!("==== {label}: {path}");
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            println!("  open: {e}");
            return;
        }
    };
    match parser.parse(&bytes) {
        Ok(rows) => {
            println!("  rows: {}", rows.len());
            for r in rows.iter().take(3) {
                println!(
                    "    {} {} {} · {}",
                    r.posted_at.format("%Y-%m-%d"),
                    r.amount,
                    r.currency,
                    r.merchant_raw.chars().take(70).collect::<String>(),
                );
            }
            if rows.len() > 3 {
                println!("    ...");
            }
            let total: rust_decimal::Decimal = rows.iter().map(|r| r.amount).sum();
            println!("  total: {total}");
        }
        Err(e) => println!("  err: {e}"),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "/Users/wizard/Desktop/taxes2024".into());
    let p = |f: &str| format!("{dir}/{f}");

    run("amazon csv", &AmazonParser, &p("amazon2024filtered.csv"));
    run("amazon xlsx", &AmazonParser, &p("amazon2024filtered.xlsx"));
    run(
        "chase csv",
        &ChaseParser,
        &p("Chase9314_Activity20240101_20241231_20250114.CSV"),
    );
    run("bofa csv", &BofaParser, &p("stmt.csv"));
    run("bofa xlsx", &BofaParser, &p("boa.xlsx"));
    run(
        "apple pdf",
        &AppleCardParser,
        &p("Apple Card Statement - January 2024.pdf"),
    );
}
