//! Spreadsheet CSV Export Module
//!
//! This module provides functionality to export spreadsheet data to CSV files.
//! Unlike the .ss format which stores formulas and dependencies, CSV export
//! only includes the calculated values of cells, making it suitable for
//! interoperability with other spreadsheet applications.

use crate::sheet::Sheet;

/// Converts a column index to alphabetic column header (A, B, C, ..., Z, AA, AB, etc.)
///
/// This function transforms a 1-based column index into the standard spreadsheet
/// column notation, supporting multi-letter columns (e.g., AA, AB) for indices
/// beyond 26.
///
/// # Arguments
///
/// * `col` - The column index (1-based) to convert
///
/// # Returns
///
/// A string representing the column in letter notation (A, B, C, ..., Z, AA, AB, etc.)
fn column_to_letter(col: usize) -> String {
    let mut result = String::new();
    let mut c = col;
    while c > 0 {
        c -= 1;
        let letter = (c % 26) as u8 + b'A';
        result.insert(0, letter as char);
        c /= 26;
    }
    result
}

impl Sheet {
    /// Exports the spreadsheet data to a CSV file
    ///
    /// This method writes the current state of the spreadsheet to a CSV file. The file
    /// includes column headers (A, B, C, etc.) and row numbers, followed by the calculated
    /// values of each cell. Note that this export only includes cell values, not formulas
    /// or dependencies.
    ///
    /// # Arguments
    ///
    /// * `filename` - The path where the CSV file should be created
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the file was successfully written
    /// * `Err(std::io::Error)` - If an I/O error occurred during file creation or writing
    ///
    /// # CSV Format
    ///
    /// The generated CSV file will have the following format:
    /// ```text
    /// ,A,B,C,...
    /// 1,42,0,0,...
    /// 2,0,0,84,...
    /// ...
    /// ```
    /// where the first row contains column headers and the first column contains row numbers.
    pub fn write_csv_file(&self, filename: &str) -> Result<(), std::io::Error> {
        let file = std::fs::File::create(filename)?;
        let mut writer = csv::Writer::from_writer(file);
        // Write the header row with column letters
        let mut header: Vec<String> = Vec::new();
        header.push("".to_string()); // Empty cell for the first column
        for i in 0..self.col {
            let col_letter = column_to_letter(i + 1);
            header.push(col_letter);
        }
        writer.write_record(&header)?;
        for i in 1..self.grid.len() {
            // Use the better approach mentioned in the comment
            let mut row: Vec<String> = self.grid[i]
                .iter()
                .map(|cell| {
                    if cell.formula.flag.type_() == 3 {
                        cell.text.clone()
                    } else {
                        cell.value.to_string()
                    }
                })
                .collect();
            row[0] = (i).to_string();
            writer.write_record(&row)?;
        }
        writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::sheet::Sheet;

    #[test]
    fn test_write_csv_file() {
        let mut test_sheet = Sheet::new(6, 6);
        test_sheet.update_cell_data(1, 1, String::from("10"));
        test_sheet
            .write_csv_file("../temp/test_output.csv")
            .unwrap();
    }
}
