//! .ss File Export Module
//!
//! This module provides functionality to export spreadsheet data to .ss files.
//! It serializes cell values, formulas, and dependencies into a structured .ss format
//! that can later be imported back into the spreadsheet.

use crate::sheet::{Cell, Sheet};
use serde::ser::{SerializeStruct, Serializer};
use serde::{self, Serialize};

/// Structure for serializing a spreadsheet cell to .ss format.
///
/// This wraps a Cell with its position information (row, column)
/// for proper serialization into the .ss format.
struct CsvStore {
    /// Row index of the cell (0-based)
    row: i32,
    /// Column index of the cell (0-based)
    col: i32,
    /// The cell data to be serialized
    data: Cell,
}

impl Serialize for CsvStore {
    /// Implements custom serialization for a cell to .ss format.
    ///
    /// This transforms the complex nested structure of a Cell into a flat
    /// structure suitable for .ss storage, including:
    /// - Converting CommandFlag bitfields to a string representation
    /// - Converting the dependency list to a comma-separated string
    ///
    /// # Parameters
    /// * `serializer` - The serializer to use
    ///
    /// # Returns
    /// * `Result<S::Ok, S::Error>` - Result of the serialization operation
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // We'll output a flat record with eight fields
        let mut state = serializer.serialize_struct("Cell", 8)?;

        // Serialize position information
        state.serialize_field("row", &self.row)?;
        state.serialize_field("col", &self.col)?;

        // Serialize the cell value
        state.serialize_field("value", &self.data.value)?;

        // Format the nested CommandFlag into a string
        let flag_str = format!(
            "type:{},cmd:{},type1:{},type2:{},error:{},div_by_zero:{}",
            self.data.formula.flag.type_(),
            self.data.formula.flag.cmd(),
            self.data.formula.flag.type1(),
            self.data.formula.flag.type2(),
            self.data.formula.flag.error(),
            self.data.formula.flag.is_div_by_zero(),
        );
        state.serialize_field("flag", &flag_str)?;

        // Serialize CommandCall's parameters
        state.serialize_field("param1", &self.data.formula.param1)?;
        state.serialize_field("param2", &self.data.formula.param2)?;

        // Convert the dependency vector into a comma-separated string
        let depend_str = self
            .data
            .depend
            .iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>()
            .join(",");
        state.serialize_field("depend", &depend_str)?;

        // Literal text content (only meaningful for type_ == 3 cells)
        state.serialize_field("text", &self.data.text)?;

        state.end()
    }
}

impl Sheet {
    /// Exports the spreadsheet data to a .ss file.
    ///
    /// This method saves the current spreadsheet state, including cell values,
    /// formulas, and dependencies to a .ss file. Only non-empty cells (cells with
    /// any flag set) are exported to keep the .ss file compact.
    ///
    /// # Parameters
    /// * `file_path` - Path to the .ss file to write
    ///
    /// # Returns
    /// * `Ok(())` - If the file was successfully written
    /// * `Err(Box<dyn std::error::Error>)` - If any error occurred during writing
    ///
    /// # .ss Format
    /// The .ss file will have the following columns:
    /// - row: Row index (0-based)
    /// - col: Column index (0-based)
    /// - value: The calculated cell value
    /// - flag: String encoding of the CommandFlag bitfield (comma-separated key-value pairs)
    /// - param1: First parameter of the cell formula
    /// - param2: Second parameter of the cell formula
    /// - depend: Comma-separated list of cell indices that depend on this cell
    pub fn write_file(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Get the dimensions
        let num_rows = self.grid.len();
        let num_cols = if num_rows > 0 { self.grid[0].len() } else { 0 };
        let mut wtr = csv::Writer::from_path(file_path)?;

        // Write only non-empty cells to the .ss file
        for row in 0..num_rows {
            for col in 0..num_cols {
                // Skip cells with no flags set (empty cells)
                if self.grid[row][col].formula.flag.is_any() == 0 {
                    continue;
                }

                // Create a CsvStore for serialization
                let cell = &self.grid[row][col];
                let csv_data = CsvStore {
                    row: row as i32,
                    col: col as i32,
                    data: cell.clone(),
                };

                // Serialize and write the cell to .ss
                wtr.serialize(csv_data)?;
            }
        }

        // Ensure data is written to disk
        wtr.flush()?;
        Ok(())
    }
}

#[test]
fn test_write_ss() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("output.ss");
    let mut test_sheet = Sheet::new(6, 6);
    test_sheet.update_cell_data(1, 1, String::from("=AVG(A2:D5)"));
    test_sheet.update_cell_data(2, 2, String::from("500"));
    test_sheet.update_cell_data(2, 1, String::from("=B1+B5"));
    test_sheet.write_file(path.to_str().unwrap()).unwrap();

    let mut reader = csv::Reader::from_path(&path).unwrap();
    assert_eq!(reader.headers().unwrap().get(7), Some("text"));
    let records = reader.records().collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(records.len(), 3);
    let constant = records
        .iter()
        .find(|record| record.get(0) == Some("2") && record.get(1) == Some("2"))
        .unwrap();
    assert_eq!(constant.get(2), Some("500"));
}
