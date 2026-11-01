//! .ss File Import Module
//!
//! This module provides functionality to import spreadsheet data from .ss files.
//! It allows loading a complete spreadsheet state, including cell values, formulas,
//! and dependencies from a .ss file.

use crate::parse::CommandFlag;
use crate::{parse::CommandCall, sheet::*};
use serde::{self, Deserialize};

/// Temporary structure for deserializing .ss records.
///
/// This structure maps directly to the .ss file columns and is used as an
/// intermediate step before converting to the Cell structure.
#[derive(Debug, Deserialize)]
struct TempRecord {
    /// Row index of the cell
    row: i32,
    /// Column index of the cell
    col: i32,
    /// Calculated value of the cell
    value: i32,
    /// String representation of the CommandFlag bitfield
    /// Format: "type:X,cmd:Y,type1:Z,..."
    flag: String,
    /// First parameter of the cell's formula
    param1: i32,
    /// Second parameter of the cell's formula
    param2: i32,
    /// Comma-separated list of cell dependencies
    depend: String,
    /// Literal text content (only meaningful for type_ == 3 cells).
    /// Defaults to empty so .ss files written before text-cell support was
    /// added still load correctly.
    #[serde(default)]
    text: String,
}

impl Sheet {
    /// Reads spreadsheet data from a .ss file and populates the sheet.
    ///
    /// This method loads a spreadsheet state from a .ss file, including cell values,
    /// formulas, and dependencies. It first resets the current sheet state, then
    /// populates it with the data from the .ss file.
    ///
    /// # Parameters
    /// * `file_path` - Path to the .ss file to read
    ///
    /// # Returns
    /// * `Ok(())` - If the file was successfully read and parsed
    /// * `Err(Box<dyn std::error::Error>)` - If any error occurred during reading or parsing
    ///
    /// # .ss Format
    /// The .ss file should have the following columns:
    /// - row: Row index (0-based)
    /// - col: Column index (0-based)
    /// - value: The calculated cell value
    /// - flag: String encoding of the CommandFlag bitfield (comma-separated key-value pairs)
    /// - param1: First parameter of the cell formula
    /// - param2: Second parameter of the cell formula
    /// - depend: Comma-separated list of cell indices that depend on this cell
    pub fn read_file(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut rdr = csv::Reader::from_path(file_path)?;

        // Reset the current sheet state
        for i in 0..self.row {
            for j in 0..self.col {
                self.grid[i][j] = Cell {
                    value: 0,
                    formula: CommandCall {
                        flag: CommandFlag::new(),
                        param1: 0,
                        param2: 0,
                    },
                    depend: Vec::new(),
                    text: String::new(),
                };
            }
        }

        // Read and process each record from the .ss file
        for result in rdr.deserialize() {
            let record: TempRecord = result?;
            let mut new_cell = Cell {
                value: record.value,
                formula: CommandCall {
                    flag: CommandFlag::new(),
                    param1: record.param1,
                    param2: record.param2,
                },
                depend: Vec::new(),
                text: record.text.clone(),
            };
            new_cell.value = record.value;
            new_cell.formula.param1 = record.param1;
            new_cell.formula.param2 = record.param2;

            // Parse the flag string and set the appropriate flags
            let flag_parts: Vec<&str> = record.flag.split(",").collect();
            for i in flag_parts {
                if let [flag_type, value_of_flag] = i.split(":").collect::<Vec<&str>>().as_slice() {
                    match *flag_type {
                        "type" => new_cell
                            .formula
                            .flag
                            .set_type_(value_of_flag.parse::<u8>().unwrap()),
                        "cmd" => new_cell
                            .formula
                            .flag
                            .set_cmd(value_of_flag.parse::<u8>().unwrap()),
                        "type1" => new_cell
                            .formula
                            .flag
                            .set_type1(value_of_flag.parse::<u8>().unwrap()),
                        "type2" => new_cell
                            .formula
                            .flag
                            .set_type2(value_of_flag.parse::<u8>().unwrap()),
                        "error" => new_cell
                            .formula
                            .flag
                            .set_error(value_of_flag.parse::<u8>().unwrap()),
                        "div_by_zero" => new_cell
                            .formula
                            .flag
                            .set_is_div_by_zero(value_of_flag.parse::<u8>().unwrap()),
                        _ => {}
                    }
                }
            }

            // Parse and set cell dependencies
            if record.depend.is_empty() {
                new_cell.depend = Vec::new();
            } else {
                let depend_parts: Vec<&str> = record.depend.split(",").collect();
                for i in depend_parts {
                    if let Ok(index) = i.parse::<usize>() {
                        new_cell.depend.push(index);
                    }
                }
            }

            // Update the cell in the grid
            self.grid[record.row as usize][record.col as usize] = new_cell;
        }
        Ok(())
    }
}

#[test]
fn test_read_ss() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("input.ss");
    let filename = path.to_str().unwrap();
    let mut test_sheet = Sheet::new(6, 6);
    test_sheet.update_cell_data(1, 1, String::from("=AVG(A2:D5)"));
    test_sheet.update_cell_data(2, 2, String::from("500"));
    test_sheet.update_cell_data(2, 1, String::from("=B1+B5"));
    test_sheet.update_cell_data(3, 3, String::from("hello, world"));
    test_sheet.write_file(filename).unwrap();
    let mut new_sheet = Sheet::new(6, 6);
    new_sheet.read_file(filename).unwrap();
    assert_eq!(new_sheet.get_value(2, 2), 500);
    assert_eq!(new_sheet.get_value(1, 1), 31);
    assert_eq!(new_sheet.grid[3][3].text, "hello, world");
    assert_eq!(new_sheet.grid[3][3].formula.flag.type_(), 3);
}
