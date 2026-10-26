//! Runs d3's translated tests against the Rust port and compares two verdicts per assertion:
//! d3's own semantics (exact, or its built-in delta/path rounding) and the tolerant intermediate format.

mod generated;
mod prelude;

use std::collections::{BTreeMap, HashSet};
use std::panic;

fn main() {
    panic::set_hook(Box::new(|_| {}));
    let mut crashed: Vec<(&str, String)> = Vec::new();
    for &(_, test, f) in generated::TESTS {
        if let Err(e) = panic::catch_unwind(f) {
            let msg = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
            crashed.push((test, msg));
        }
    }
    let records = prelude::RECORDS.with(|r| r.borrow().clone());
    let ran: HashSet<usize> = records.iter().map(|r| r.id).collect();

    #[derive(Default)]
    struct Row { total: usize, skipped: usize, translated: usize, not_reached: usize, d3: usize, tolerant: usize, tolerant_only: usize, d3_only: usize }
    let mut rows: BTreeMap<&str, Row> = BTreeMap::new();
    for &(id, module, _, status, _) in generated::ASSERTS {
        let row = rows.entry(module).or_default();
        row.total += 1;
        if status == "skip" { row.skipped += 1; continue; }
        row.translated += 1;
        if !ran.contains(&id) { row.not_reached += 1; }
    }
    let module_of: BTreeMap<usize, &str> = generated::ASSERTS.iter().map(|a| (a.0, a.1)).collect();
    for r in &records {
        let row = rows.get_mut(module_of[&r.id]).expect("known module");
        row.d3 += r.d3_pass as usize;
        row.tolerant += r.tolerant_pass as usize;
        row.tolerant_only += (r.tolerant_pass && !r.d3_pass) as usize;
        row.d3_only += (r.d3_pass && !r.tolerant_pass) as usize;
    }
    println!("{:8} {:>6} {:>8} {:>11} {:>12} {:>9} {:>14} {:>15}", "module", "total", "skipped", "translated", "not reached", "d3 pass", "tolerant pass", "tolerant only");
    for (m, r) in &rows {
        println!("{:8} {:>6} {:>8} {:>11} {:>12} {:>9} {:>14} {:>15}", m, r.total, r.skipped, r.translated, r.not_reached, r.d3, r.tolerant, r.tolerant_only);
        assert_eq!(r.d3_only, 0, "an assertion passed d3 semantics but failed the tolerant format");
    }
    println!("\nfails under d3 semantics, passes tolerant (first 12):");
    for r in records.iter().filter(|r| r.tolerant_pass && !r.d3_pass).take(100) {
        println!("  #{} {:?}: actual {:?} expected {:?}", r.id, r.kind, r.actual, r.expected);
    }
    println!("\nfails tolerant:");
    for r in records.iter().filter(|r| !r.tolerant_pass) {
        let test = generated::ASSERTS.iter().find(|a| a.0 == r.id).map(|a| a.2).unwrap_or("?");
        println!("  #{} [{}] {:?}: actual {:?} expected {:?}", r.id, test, r.kind, r.actual, r.expected);
    }
    println!("\ntests that panicked: {}", crashed.len());
    for (t, m) in &crashed { println!("  {t}: {m}"); }
}
