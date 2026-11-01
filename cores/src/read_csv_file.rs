//! Spreadsheet CSV Import Module
//!
//! This module provides functionality to import spreadsheet data from CSV files.
//! Unlike the .ss format which can restore formulas and dependencies, CSV import
//! only loads cell values, treating all cells as constants without formulas.
//! This is useful for importing data from other spreadsheet applications.

use crate::parse::CommandFlag;
use crate::{parse::CommandCall, sheet::*};

impl Sheet {
    /// Imports spreadsheet data from a CSV file
    ///
    /// This method reads a CSV file and populates the current spreadsheet with the values
    /// from the file. It first resets all cells in the current sheet, then imports values
    /// from the CSV. The method expects a specific CSV format with column headers in the
    /// first row and row numbers in the first column.
    ///
    /// # Arguments
    ///
    /// * `filename` - The path to the CSV file to be imported
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the file was successfully read and parsed
    /// * `Err(std::io::Error)` - If an I/O error occurred during file reading or parsing
    ///
    /// # CSV Format
    ///
    /// The expected CSV file format is:
    /// ```text
    /// ,A,B,C,...
    /// 1,42,15,0,...
    /// 2,0,7,84,...
    /// ...
    /// ```
    ///
    /// # Notes
    ///
    /// - The first row is treated as column headers and skipped
    /// - The first column in each row is treated as the row number and skipped
    /// - Empty cells in the CSV file remain as 0 in the sheet
    /// - All imported cells are treated as constants (no formulas)
    pub fn read_csv_file(&mut self, filename: &str) -> Result<(), std::io::Error> {
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
        let file = std::fs::File::open(filename)?;
        let mut reader = csv::Reader::from_reader(file);
        let mut row = 1;
        // Read the header row and skip it
        let _ = reader.headers()?;
        for result in reader.deserialize() {
            let cell_data: Vec<String> = result?;
            // Skip the first row if it contains headers
            for (i, item) in cell_data.iter().enumerate().skip(1) {
                if item.is_empty() {
                    continue;
                }
                let col = i;
                if col >= self.col {
                    break;
                }
                if let Ok(value) = item.parse::<i32>() {
                    self.grid[row][col].value = value;
                } else {
                    self.grid[row][col].formula.flag.set_type_(3);
                    self.grid[row][col].formula.flag.set_is_any(1);
                    self.grid[row][col].text = item.clone();
                }
            }
            row += 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::sheet::Sheet;

    #[test]
    fn test_read_csv_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("input.csv");
        std::fs::write(&path, ",A,B\n1,10,\"hello, world\"\n").unwrap();

        let mut test_sheet = Sheet::new(6, 6);
        test_sheet.read_csv_file(path.to_str().unwrap()).unwrap();
        assert_eq!(test_sheet.grid[1][1].value, 10);
        assert_eq!(test_sheet.grid[1][2].text, "hello, world");
        assert_eq!(test_sheet.grid[1][2].formula.flag.type_(), 3);
    }
}
