//! Command-line interface for the spreadsheet application.
//!
//! This module provides a terminal-based interface for interacting with the spreadsheet.
//! It supports operations such as:
//! - Viewing spreadsheet data
//! - Entering formulas and values into cells
//! - Navigating through the spreadsheet using keyboard commands
//! - Scrolling to specific cells

use cores::Error;
use cores::Sheet;
use cores::convert_to_index;
use std::cmp;
use std::env;
use std::io;
use std::io::Write;

/// Controls debug output throughout the application
const DEBUG: bool = false;

/// Converts a numeric column index to an alphabetic column name.
///
/// This function transforms column indices (1-based) into Excel-style
/// column identifiers (A, B, ..., Z, AA, AB, etc.).
///
/// # Parameters
/// * `col` - The column index to convert (1-based)
///
/// # Returns
/// A string representing the column name
pub fn column_to_letter(col: usize) -> String {
    let mut result = String::new();
    let mut col_num = col;

    while col_num > 0 {
        let remainder = (col_num - 1) % 26;
        result.insert(0, (b'A' + remainder as u8) as char);
        col_num = (col_num - 1) / 26;
    }

    result
}

/// Displays a portion of the spreadsheet in the terminal.
///
/// This function prints a 10x10 section of the spreadsheet starting from
/// the specified row and column indices. It formats the output as a table
/// with row and column headers.
///
/// # Parameters
/// * `sheet` - Reference to the spreadsheet to display
/// * `row` - Total number of rows in the spreadsheet
/// * `col` - Total number of columns in the spreadsheet
/// * `rowi` - Starting row index for display
/// * `coli` - Starting column index for display
fn display_sheet(sheet: &Sheet, row: usize, col: usize, rowi: usize, coli: usize) {
    let mut i = coli;
    print!(" \t ");
    while i < coli + 10 && i < col {
        print!("{}\t ", column_to_letter(i));
        i += 1;
    }
    println!();
    i = rowi;
    while i < rowi + 10 && i < row {
        print!("{}\t ", i);
        let mut j = coli;
        while j < coli + 10 && j < col {
            let value = sheet.get_value(i as i32, j as i32);
            if sheet.grid[i][j].formula.flag.is_div_by_zero() == 1 {
                print!("ERR\t ");
            } else {
                print!("{}\t ", value);
            }
            j += 1;
        }
        println!();
        i += 1;
    }
}

/// Main function implementing the command-line interface for the spreadsheet.
///
/// The CLI supports the following commands:
/// - `w`: Move up (display previous 10 rows)
/// - `a`: Move left (display previous 10 columns)
/// - `s`: Move down (display next 10 rows)
/// - `d`: Move right (display next 10 columns)
/// - `scroll_to <cell>`: Jump to the specified cell location
/// - `disable_output`: Stop displaying the spreadsheet after each command
/// - `enable_output`: Resume displaying the spreadsheet after each command
/// - `q`: Quit the application
/// - `<cell>=<formula>`: Set a formula for the specified cell
///
/// # Command-line Arguments
/// * First argument: Number of rows in the spreadsheet
/// * Second argument: Number of columns in the spreadsheet
pub fn run_help(args: Vec<String>) {
    if args.len() != 3 {
        eprintln!("Usage: {} <rows> <columns>", args[0]);
        std::process::exit(1);
    }
    let int1: i32 = args[1].parse().expect("Invalid integer for rows");
    let int2: i32 = args[2].parse().expect("Invalid integer for columns");
    let int1 = int1 + 1;
    let int2 = int2 + 1;

    // Initialize spreadsheet and UI state
    let mut test_sheet = Sheet::new(int1 as usize, int2 as usize);
    let mut rowi = 1;
    let mut coli = 1;
    let mut input = String::new();
    let mut display_button = true;
    let mut massage = "ok";
    let mut time = 0.0;

    // Main input loop
    while {
        if display_button {
            display_sheet(
                &test_sheet,
                int1 as usize,
                int2 as usize,
                rowi as usize,
                coli as usize,
            );
        }
        print!("[{}] ({}) > ", (time / 1000.0), massage);
        massage = "ok";
        io::stdout().flush().unwrap();

        input.clear(); // Clear previous input
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        let input_trimmed = input.trim();
        input_trimmed != "q" // Continue until 'q' is entered
    } {
        let trimmed = input.trim();
        if DEBUG {
            println!("{}", trimmed);
        }

        // Handle cell assignment (e.g., "A1=42" or "B2=A1+C3")
        if trimmed.contains("=") {
            if DEBUG {
                println!("This is an assignment: {}", trimmed);
            }
            // Split the assignment into left-hand side (lhs) and right-hand side (rhs)
            let parts: Vec<&str> = trimmed.split('=').collect();
            if parts.len() == 2 {
                let lhs = parts[0].trim(); // e.g., A1
                let rhs = parts[1].trim(); // e.g., A2+A3
                if DEBUG {
                    println!("Left: {}, Right: {}", lhs, rhs);
                }
                // Convert the cell reference to indices
                let (cell_index_row, cell_index_col) = convert_to_index(lhs.to_string());
                // The core engine now only auto-detects a formula when the
                // content starts with '=' (Excel-style); since the CLI's own
                // "CELL=CONTENT" syntax already consumes that '=' as its
                // assignment separator, re-add it here so "A1=A2+A3" keeps
                // being treated as a formula rather than literal text.
                let rhs_content = format!("={}", rhs);
                let result = test_sheet.update_cell_data(
                    cell_index_row,
                    cell_index_col,
                    rhs_content,
                );
                time = result.time;
                match result.error {
                    Error::InvalidInput => massage = "invalid input",
                    Error::None => massage = "ok",
                    Error::CycleDetected => massage = "cycle detected",
                    Error::DivByZero => massage = "ok",
                }
            } else {
                massage = "invalid input";
            }
        }
        // Handle navigation commands
        else if trimmed == "w" {
            rowi = cmp::max(1, rowi - 10);
        } else if trimmed == "s" {
            rowi = cmp::min(int1 - 10, rowi + 10);
        } else if trimmed == "a" {
            coli = cmp::max(1, coli - 10);
        } else if trimmed == "d" {
            coli = cmp::min(int2 - 10, coli + 10);
        }
        // Handle display toggle commands
        else if trimmed == "disable_output" {
            display_button = false
        } else if trimmed == "enable_output" {
            display_button = true
        }
        // Handle scroll_to command
        else if trimmed.len() > 9 && &trimmed[0..9] == "scroll_to" {
            let parts: Vec<&str> = trimmed.split(' ').collect();
            if parts.len() == 2 {
                let (scroll_row, scroll_col) = convert_to_index(parts[1].to_string());
                if ((scroll_row as i32) < int1)
                    && ((scroll_col as i32) < int2)
                    && (scroll_row as i32 >= 1)
                    && (scroll_col as i32 >= 1)
                {
                    rowi = scroll_row as i32;
                    coli = scroll_col as i32;
                    rowi = cmp::min(rowi, int1);
                    coli = cmp::min(coli, int2);
                } else {
                    massage = "invalid input";
                }
            } else {
                massage = "invalid input";
            }
        } else {
            massage = "invalid input";
        }
    }
}

pub fn run() {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    run_help(args);
}

#[cfg(test)]
mod tests {
    use super::*;
    // use std::io::Write;

    // Test utility methods like column_to_letter
    #[test]
    fn test_column_to_letter() {
        assert_eq!(column_to_letter(1), "A");
        assert_eq!(column_to_letter(26), "Z");
        assert_eq!(column_to_letter(27), "AA");
        assert_eq!(column_to_letter(28), "AB");
        assert_eq!(column_to_letter(52), "AZ");
        assert_eq!(column_to_letter(53), "BA");
    }

    #[test]
    fn test_convert_to_index() {
        let (row, col) = convert_to_index("A1".to_string());
        assert_eq!(row, 1);
        assert_eq!(col, 1);
    }
    #[test]
    fn test_display_sheet() {
        let mut test_sheet = Sheet::new(20, 20);
        test_sheet.grid[1][1].value = 42;
        test_sheet.grid[1][2].value = 50;
        test_sheet.grid[1][3].value = 100;
        test_sheet.grid[1][4].formula.flag.set_is_div_by_zero(1);
        let rowi = 1;
        let coli = 1;
        display_sheet(&test_sheet, 20, 20, rowi as usize, coli as usize);
    }
    #[test]
    fn test_invalid_input() {
        let (row, col) = convert_to_index("Z1000".to_string());
        assert_eq!(row, 1000);
        assert_eq!(col, 26);
    }

    // // Test the main function with mocked input

    // // Helper to run tests with simulated stdin/stdout
    // // Replace the failing test with this improved version
    // #[test]
    // fn test_run_help_with_mocked_io() {
    //     use std::path::Path;
    //     use std::process::{Command, Stdio};
    //     use std::thread;

    //     // Helper to find the binary
    //     fn find_binary() -> Option<String> {
    //         // Try possible binary locations
    //         let possible_paths = vec![
    //             "target/debug/cli",
    //             "../target/debug/cli",
    //             "target/release/cli",
    //             "../target/release/cli",
    //         ];

    //         for path in &possible_paths {
    //             if Path::new(path).exists() {
    //                 return Some(path.to_string());
    //             }
    //         }

    //         // Try building the binary
    //         let build_status = Command::new("cargo")
    //             .args(["build", "--bin", "cli"])
    //             .status();

    //         if let Ok(status) = build_status {
    //             if status.success() {
    //                 // Try paths again after building
    //                 for path in &possible_paths {
    //                     if Path::new(path).exists() {
    //                         return Some(path.to_string());
    //                     }
    //                 }
    //             }
    //         }

    //         None
    //     }

    //     // Get binary path or skip test
    //     let bin_path = match find_binary() {
    //         Some(path) => path,
    //         None => {
    //             println!("WARNING: Couldn't find or build CLI binary, skipping test");
    //             return; // Skip the test
    //         }
    //     };

    //     // Create a test with input
    //     let handle = thread::spawn(move || {
    //         let mut child = Command::new(&bin_path)
    //             .args(["10", "10"])
    //             .stdin(Stdio::piped())
    //             .stdout(Stdio::piped())
    //             .spawn()
    //             .unwrap_or_else(|_| panic!("Failed to start command: {}", bin_path));

    //         let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    //         stdin
    //             .write_all(b"A1=42\nB1=84\nq\n")
    //             .expect("Failed to write to stdin");

    //         // Get output
    //         let output = child.wait_with_output().expect("Failed to wait on child");
    //         assert!(output.status.success());

    //         // Check output
    //         let stdout = String::from_utf8_lossy(&output.stdout);
    //         assert!(stdout.contains("42"));
    //         assert!(stdout.contains("84"));
    //     });

    //     handle.join().unwrap();
    // }

    // #[test]
    // fn test_run_help_navigation_commands() {
    //     use std::path::Path;
    //     use std::process::{Command, Stdio};
    //     use std::thread;

    //     // Reuse the find_binary function from test_run_help_with_mocked_io
    //     fn find_binary() -> Option<String> {
    //         let possible_paths = vec![
    //             "target/debug/cli",
    //             "../target/debug/cli",
    //             "target/release/cli",
    //             "../target/release/cli",
    //         ];

    //         for path in &possible_paths {
    //             if Path::new(path).exists() {
    //                 return Some(path.to_string());
    //             }
    //         }

    //         let build_status = Command::new("cargo")
    //             .args(["build", "--bin", "cli"])
    //             .status();

    //         if let Ok(status) = build_status {
    //             if status.success() {
    //                 for path in &possible_paths {
    //                     if Path::new(path).exists() {
    //                         return Some(path.to_string());
    //                     }
    //                 }
    //             }
    //         }

    //         None
    //     }

    //     let bin_path = match find_binary() {
    //         Some(path) => path,
    //         None => {
    //             println!("WARNING: Couldn't find or build CLI binary, skipping test");
    //             return;
    //         }
    //     };

    //     let handle = thread::spawn(move || {
    //         // Use a large sheet so we can test navigation
    //         let mut child = Command::new(&bin_path)
    //             .args(["20", "20"])
    //             .stdin(Stdio::piped())
    //             .stdout(Stdio::piped())
    //             .spawn()
    //             .unwrap_or_else(|_| panic!("Failed to start command: {}", bin_path));

    //         let stdin = child.stdin.as_mut().expect("Failed to open stdin");

    //         // Fill some cells in different areas of the sheet
    //         stdin.write_all(b"A1=11\n").expect("Failed to write");
    //         stdin.write_all(b"A15=15\n").expect("Failed to write"); // Out of initial view
    //         stdin.write_all(b"O1=101\n").expect("Failed to write"); // Also out of initial view
    //         stdin.write_all(b"O15=1515\n").expect("Failed to write"); // Far corner

    //         // Test navigation commands
    //         stdin.write_all(b"s\n").expect("Failed to write"); // Move down to see row A15
    //         stdin.write_all(b"d\n").expect("Failed to write"); // Move right to see column O
    //         stdin.write_all(b"w\n").expect("Failed to write"); // Move back up
    //         stdin.write_all(b"a\n").expect("Failed to write"); // Move back left
    //         stdin.write_all(b"q\n").expect("Failed to write"); // Quit

    //         let output = child.wait_with_output().expect("Failed to wait on child");
    //         assert!(output.status.success());

    //         let stdout = String::from_utf8_lossy(&output.stdout);
    //         assert!(stdout.contains("11")); // Initial view

    //         // These checks are a bit fragile since we don't know the exact output format,
    //         // but we can verify the command sequence ran successfully
    //         assert!(output.status.success());
    //     });

    //     handle.join().unwrap();
    // }

    // #[test]
    // fn test_run_help_display_toggle() {
    //     use std::path::Path;
    //     use std::process::{Command, Stdio};
    //     use std::thread;

    //     fn find_binary() -> Option<String> {
    //         // Same function as above
    //         let possible_paths = vec![
    //             "target/debug/cli",
    //             "../target/debug/cli",
    //             "target/release/cli",
    //             "../target/release/cli",
    //         ];

    //         for path in &possible_paths {
    //             if Path::new(path).exists() {
    //                 return Some(path.to_string());
    //             }
    //         }

    //         let build_status = Command::new("cargo")
    //             .args(["build", "--bin", "cli"])
    //             .status();

    //         if let Ok(status) = build_status {
    //             if status.success() {
    //                 for path in &possible_paths {
    //                     if Path::new(path).exists() {
    //                         return Some(path.to_string());
    //                     }
    //                 }
    //             }
    //         }

    //         None
    //     }

    //     let bin_path = match find_binary() {
    //         Some(path) => path,
    //         None => {
    //             println!("WARNING: Couldn't find or build CLI binary, skipping test");
    //             return;
    //         }
    //     };

    //     let handle = thread::spawn(move || {
    //         let mut child = Command::new(&bin_path)
    //             .args(["10", "10"])
    //             .stdin(Stdio::piped())
    //             .stdout(Stdio::piped())
    //             .spawn()
    //             .unwrap_or_else(|_| panic!("Failed to start command: {}", bin_path));

    //         let stdin = child.stdin.as_mut().expect("Failed to open stdin");

    //         // Initial value
    //         stdin.write_all(b"A1=42\n").expect("Failed to write");
    //         // Disable output
    //         stdin
    //             .write_all(b"disable_output\n")
    //             .expect("Failed to write");
    //         // Add more values (grid shouldn't display)
    //         stdin.write_all(b"B1=84\n").expect("Failed to write");
    //         stdin.write_all(b"C1=126\n").expect("Failed to write");
    //         // Enable output
    //         stdin
    //             .write_all(b"enable_output\n")
    //             .expect("Failed to write");
    //         // Add one more value (grid should display)
    //         stdin.write_all(b"D1=168\n").expect("Failed to write");
    //         stdin.write_all(b"q\n").expect("Failed to write");

    //         let output = child.wait_with_output().expect("Failed to wait on child");
    //         assert!(output.status.success());

    //         let stdout = String::from_utf8_lossy(&output.stdout);

    //         // Count grid appearances (by counting column headers)
    //         let grid_displays = stdout.matches(" \t A").count();

    //         // Should display grid at least 3 times:
    //         // - Initial display
    //         // - After A1=42
    //         // - After enable_output+D1=168
    //         assert!(
    //             grid_displays >= 3,
    //             "Grid should be displayed at least 3 times, got {}",
    //             grid_displays
    //         );
    //     });

    //     handle.join().unwrap();
    // }

    // #[test]
    // fn test_run_help_scroll_to() {
    //     use std::path::Path;
    //     use std::process::{Command, Stdio};
    //     use std::thread;

    //     // Reuse function
    //     fn find_binary() -> Option<String> {
    //         let possible_paths = vec![
    //             "target/debug/cli",
    //             "../target/debug/cli",
    //             "target/release/cli",
    //             "../target/release/cli",
    //         ];

    //         for path in &possible_paths {
    //             if Path::new(path).exists() {
    //                 return Some(path.to_string());
    //             }
    //         }

    //         let build_status = Command::new("cargo")
    //             .args(["build", "--bin", "cli"])
    //             .status();

    //         if let Ok(status) = build_status {
    //             if status.success() {
    //                 for path in &possible_paths {
    //                     if Path::new(path).exists() {
    //                         return Some(path.to_string());
    //                     }
    //                 }
    //             }
    //         }

    //         None
    //     }

    //     let bin_path = match find_binary() {
    //         Some(path) => path,
    //         None => {
    //             println!("WARNING: Couldn't find or build CLI binary, skipping test");
    //             return;
    //         }
    //     };

    //     let handle = thread::spawn(move || {
    //         let mut child = Command::new(&bin_path)
    //             .args(["30", "30"])
    //             .stdin(Stdio::piped())
    //             .stdout(Stdio::piped())
    //             .spawn()
    //             .unwrap_or_else(|_| panic!("Failed to start command: {}", bin_path));

    //         let stdin = child.stdin.as_mut().expect("Failed to open stdin");

    //         // Add value in visible range
    //         stdin.write_all(b"A1=1\n").expect("Failed to write");
    //         // Add value outside visible range
    //         stdin.write_all(b"Z25=9999\n").expect("Failed to write");
    //         // Scroll to the far cell
    //         stdin
    //             .write_all(b"scroll_to Z25\n")
    //             .expect("Failed to write");
    //         // Add one more value
    //         stdin.write_all(b"Y25=8888\n").expect("Failed to write");
    //         // Test invalid scroll_to command
    //         stdin.write_all(b"scroll_to\n").expect("Failed to write");
    //         // Test invalid cell reference
    //         stdin
    //             .write_all(b"scroll_to ZZZ999\n")
    //             .expect("Failed to write");
    //         stdin.write_all(b"q\n").expect("Failed to write");

    //         let output = child.wait_with_output().expect("Failed to wait on child");
    //         assert!(output.status.success());
    //     });

    //     handle.join().unwrap();
    // }

    // #[test]
    // #[ignore]
    // fn test_run_help_error_handling() {
    //     use std::path::Path;
    //     use std::process::{Command, Stdio};
    //     use std::thread;

    //     fn find_binary() -> Option<String> {
    //         let possible_paths = vec![
    //             "target/debug/cli",
    //             "../target/debug/cli",
    //             "target/release/cli",
    //             "../target/release/cli",
    //         ];

    //         for path in &possible_paths {
    //             if Path::new(path).exists() {
    //                 return Some(path.to_string());
    //             }
    //         }

    //         let build_status = Command::new("cargo")
    //             .args(["build", "--bin", "cli"])
    //             .status();

    //         if let Ok(status) = build_status {
    //             if status.success() {
    //                 for path in &possible_paths {
    //                     if Path::new(path).exists() {
    //                         return Some(path.to_string());
    //                     }
    //                 }
    //             }
    //         }

    //         None
    //     }

    //     let bin_path = match find_binary() {
    //         Some(path) => path,
    //         None => {
    //             println!("WARNING: Couldn't find or build CLI binary, skipping test");
    //             return;
    //         }
    //     };

    //     let handle = thread::spawn(move || {
    //         let mut child = Command::new(&bin_path)
    //             .args(["10", "10"])
    //             .stdin(Stdio::piped())
    //             .stdout(Stdio::piped())
    //             .spawn()
    //             .unwrap_or_else(|_| panic!("Failed to start command: {}", bin_path));

    //         let stdin = child.stdin.as_mut().expect("Failed to open stdin");

    //         // Add normal value
    //         stdin.write_all(b"A1=42\n").expect("Failed to write");

    //         // Test invalid formula
    //         stdin
    //             .write_all(b"A2=invalid_formula\n")
    //             .expect("Failed to write");

    //         // Test reference before assignment
    //         stdin.write_all(b"B1=C1\n").expect("Failed to write");

    //         // Test cycle detection
    //         stdin.write_all(b"C1=10\n").expect("Failed to write");
    //         stdin.write_all(b"D1=C1+E1\n").expect("Failed to write");
    //         stdin.write_all(b"E1=D1\n").expect("Failed to write");

    //         // Test division by zero
    //         stdin.write_all(b"F1=1/0\n").expect("Failed to write");

    //         // Test completely invalid command
    //         stdin
    //             .write_all(b"this_is_not_a_valid_command\n")
    //             .expect("Failed to write");

    //         stdin.write_all(b"q\n").expect("Failed to write");

    //         let output = child.wait_with_output().expect("Failed to wait on child");
    //         assert!(output.status.success());

    //         let stdout = String::from_utf8_lossy(&output.stdout);

    //         // Check for error messages
    //         assert!(
    //             stdout.contains("invalid input"),
    //             "Should show 'invalid input' message"
    //         );
    //         assert!(
    //             stdout.contains("cycle detected"),
    //             "Should show 'cycle detected' message"
    //         );
    //         assert!(
    //             stdout.contains("ERR"),
    //             "Should show 'ERR' for division by zero"
    //         );
    //     });

    //     handle.join().unwrap();
    // }

    // #[test]
    // #[ignore]
    // fn test_run_help_formula_evaluation() {
    //     use std::path::Path;
    //     use std::process::{Command, Stdio};
    //     use std::thread;

    //     fn find_binary() -> Option<String> {
    //         let possible_paths = vec![
    //             "target/debug/cli",
    //             "../target/debug/cli",
    //             "target/release/cli",
    //             "../target/release/cli",
    //         ];

    //         for path in &possible_paths {
    //             if Path::new(path).exists() {
    //                 return Some(path.to_string());
    //             }
    //         }

    //         let build_status = Command::new("cargo")
    //             .args(["build", "--bin", "cli"])
    //             .status();

    //         if let Ok(status) = build_status {
    //             if status.success() {
    //                 for path in &possible_paths {
    //                     if Path::new(path).exists() {
    //                         return Some(path.to_string());
    //                     }
    //                 }
    //             }
    //         }

    //         None
    //     }

    //     let bin_path = match find_binary() {
    //         Some(path) => path,
    //         None => {
    //             println!("WARNING: Couldn't find or build CLI binary, skipping test");
    //             return;
    //         }
    //     };

    //     let handle = thread::spawn(move || {
    //         let mut child = Command::new(&bin_path)
    //             .args(["10", "10"])
    //             .stdin(Stdio::piped())
    //             .stdout(Stdio::piped())
    //             .spawn()
    //             .unwrap_or_else(|_| panic!("Failed to start command: {}", bin_path));

    //         let stdin = child.stdin.as_mut().expect("Failed to open stdin");

    //         // Add values
    //         stdin.write_all(b"A1=10\n").expect("Failed to write");
    //         stdin.write_all(b"A2=20\n").expect("Failed to write");

    //         // Test simple addition
    //         stdin.write_all(b"B1=A1+A2\n").expect("Failed to write");

    //         // Test more complex formula with order of operations
    //         stdin.write_all(b"B2=(A1+A2)*2\n").expect("Failed to write");

    //         // Test formula with multiple references
    //         stdin
    //             .write_all(b"C1=A1+A2+B1+B2\n")
    //             .expect("Failed to write");

    //         stdin.write_all(b"q\n").expect("Failed to write");

    //         let output = child.wait_with_output().expect("Failed to wait on child");
    //         assert!(output.status.success());
    //     });

    //     handle.join().unwrap();
    // }
}
