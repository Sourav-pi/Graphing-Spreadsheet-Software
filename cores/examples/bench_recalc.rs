//! Benchmark: DAG-based selective recalculation vs. naive full-sheet recompute.
//!
//! Uses only the public API of `cores` (Sheet::new, update_cell_data, get_formula).
//! Run with: cargo run --release --example bench_recalc -p cores

use cores::Sheet;
use std::time::Instant;

const ROWS: usize = 300;
const COLS: usize = 300;
const DEFAULT_NUM_DEPENDENTS: usize = 7500;
const SELECTIVE_TRIALS: usize = 20;
const NAIVE_TRIALS: usize = 3;

fn main() {
    let num_dependents: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_NUM_DEPENDENTS);

    let mut sheet = Sheet::new(ROWS, COLS);

    // 2. Fill every cell with an independent constant formula.
    for r in 1..=ROWS {
        for c in 1..=COLS {
            sheet.update_cell_data(r, c, "1".to_string());
        }
    }

    // 3. Make a small number of cells directly depend on A1.
    //    Spread them across the sheet (skip A1 itself, row=1,col=1).
    assert!(num_dependents < ROWS * COLS, "num_dependents must leave room for A1 itself");
    let mut dependent_cells: Vec<(usize, usize)> = Vec::with_capacity(num_dependents);
    let mut r = 1;
    let mut c = 2;
    while dependent_cells.len() < num_dependents {
        sheet.update_cell_data(r, c, "=A1+0".to_string());
        dependent_cells.push((r, c));
        c += 1;
        if c > COLS {
            c = 1;
            r += 1;
        }
    }

    // 4. Benchmark selective recalc: repeatedly update A1, timing each call.
    let mut selective_total_ms = 0.0f64;
    for i in 0..SELECTIVE_TRIALS {
        let formula = format!("{}", i + 1);
        let start = Instant::now();
        sheet.update_cell_data(1, 1, formula);
        selective_total_ms += start.elapsed().as_secs_f64() * 1000.0;
    }
    let selective_avg_ms = selective_total_ms / SELECTIVE_TRIALS as f64;

    // 5. Benchmark naive full-sheet recompute: re-apply every cell's existing
    //    formula through the same public API, simulating an engine with no
    //    dependency tracking that must touch every cell on any edit.
    let mut naive_total_ms = 0.0f64;
    for _ in 0..NAIVE_TRIALS {
        let start = Instant::now();
        for r in 1..=ROWS {
            for c in 1..=COLS {
                let formula = sheet.get_formula(r, c);
                sheet.update_cell_data(r, c, formula);
            }
        }
        naive_total_ms += start.elapsed().as_secs_f64() * 1000.0;
    }
    let naive_avg_ms = naive_total_ms / NAIVE_TRIALS as f64;

    let speedup = naive_avg_ms / selective_avg_ms;

    println!("Sheet size: {}x{} ({} cells)", ROWS, COLS, ROWS * COLS);
    println!("Dependent subgraph size (direct dependents of A1): {}", num_dependents);
    println!("Selective recalc (update A1, {} trials): avg {:.4} ms", SELECTIVE_TRIALS, selective_avg_ms);
    println!("Naive full-sheet recompute ({} trials): avg {:.4} ms", NAIVE_TRIALS, naive_avg_ms);
    println!("Speedup (naive / selective): {:.2}x", speedup);
    // machine-readable line for scripted collection
    println!("CSV,{},{},{:.6},{:.6},{:.6}", ROWS * COLS, num_dependents, selective_avg_ms, naive_avg_ms, speedup);
}
