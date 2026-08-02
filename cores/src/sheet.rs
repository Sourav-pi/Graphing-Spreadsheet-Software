use crate::parse::*;
use fxhash::FxHashSet;
use std::{thread, time};

const DEBUG: bool = false;

/// Constant used for encoding cell indices
pub const ENCODE_SHIFT: usize = 10000;

/// A cell in the spreadsheet.
///
/// Each cell holds a value, a formula, and a list of cells that depend on it.
#[derive(Clone)]
pub struct Cell {
    /// The current calculated value of the cell
    pub value: i32,
    /// The formula assigned to the cell
    pub formula: CommandCall,
    /// List of cells that depend on this cell's value
    pub depend: Vec<usize>,
    /// Literal text content, populated only when `formula.flag.type_() == 3`
    /// (a cell holding plain text rather than a number or formula).
    pub text: String,
}

/// Error types that can occur during spreadsheet operations.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Division by zero error
    DivByZero,
    /// Invalid input error when parsing formula
    InvalidInput,
    /// Cyclic dependency detected
    CycleDetected,
    /// No error
    None,
}

/// Result of a cell update operation.
#[allow(dead_code)]
pub struct CallResult {
    /// Time taken to execute the operation (in milliseconds)
    pub time: f64,
    /// Error that occurred during the operation, if any
    pub error: Error,
}

/// A structure representing a spreadsheet with cells that can contain values and formulas.
///
/// The spreadsheet consists of a grid of cells, each capable of storing a value,
/// a formula, and dependencies on other cells. Operations performed on the spreadsheet
/// ensure that all dependencies are properly maintained and cycles are detected.
pub struct Sheet {
    /// The grid of cells in the spreadsheet, represented as a 2D vector.
    pub grid: Vec<Vec<Cell>>,
    /// Number of rows in the spreadsheet.
    pub row: usize,
    /// Number of columns in the spreadsheet.
    pub col: usize,
}

impl Sheet {
    /// Creates a new spreadsheet with the specified dimensions.
    ///
    /// # Parameters
    /// * `row` - Number of rows in the spreadsheet
    /// * `col` - Number of columns in the spreadsheet
    ///
    /// # Returns
    /// A new `Sheet` instance with all cells initialized to zero.
    pub fn new(row: usize, col: usize) -> Self {
        let grid: Vec<Vec<Cell>> = vec![
            vec![
                Cell {
                    value: 0,
                    formula: CommandCall {
                        flag: CommandFlag::new(),
                        param1: 0,
                        param2: 0,
                    },
                    depend: Vec::new(),
                    text: String::new(),
                };
                col + 1
            ];
            row + 1
        ];
        Self { grid, row, col }
    }

    /// Returns the formula string for a specific cell.
    ///
    /// # Parameters
    /// * `row` - Row index of the cell
    /// * `col` - Column index of the cell
    ///
    /// # Returns
    /// A string representation of the cell's formula.
    #[allow(dead_code)]
    pub fn get_formula(&self, row: usize, col: usize) -> String {
        unparse(self.grid[row][col].clone())
    }

    /// Adds additional rows to the spreadsheet.
    ///
    /// # Parameters
    /// * `no_of_row` - Number of rows to add
    ///     Adds additional columns to the spreadsheet.
    ///
    /// # Parameters
    /// * `no_of_col` - Number of columns to add
    #[allow(dead_code)]
    pub fn copy_row(&mut self, copy_from: usize, copy_to: usize) -> Result<(), Error> {
        // Save original state of destination row in case we need to rollback
        let original_row: Vec<Cell> = self.grid[copy_to].clone();

        for i in 0..self.col {
            match self
                .update_cell_data(copy_to, i, self.get_formula(copy_from, i))
                .error
            {
                Error::None => {}
                Error::DivByZero => {
                    self.grid[copy_to][i].formula.flag.set_is_div_by_zero(1);
                }
                Error::InvalidInput => {
                    self.grid[copy_to][i].formula.flag.set_error(1);
                }
                Error::CycleDetected => {
                    // Rollback all changes to prevent corrupted state
                    self.grid[copy_to] = original_row;
                    return Err(Error::CycleDetected);
                }
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn copy_col(&mut self, copy_from: usize, copy_to: usize) -> Result<(), Error> {
        // Save original state of destination column in case we need to rollback
        let mut original_col: Vec<Cell> = Vec::with_capacity(self.row);
        for i in 0..self.row {
            original_col.push(self.grid[i][copy_to].clone());
        }

        // Perform the copy operation
        for i in 0..self.row {
            match self
                .update_cell_data(i, copy_to, self.get_formula(i, copy_from))
                .error
            {
                Error::None => {}
                Error::DivByZero => {
                    self.grid[i][copy_to].formula.flag.set_is_div_by_zero(1);
                }
                Error::InvalidInput => {
                    self.grid[i][copy_to].formula.flag.set_error(1);
                }
                Error::CycleDetected => {
                    // Rollback all changes to prevent corrupted state
                    for (i, item) in original_col.iter().enumerate().take(self.row) {
                        self.grid[i][copy_to] = item.clone();
                    }
                    return Err(Error::CycleDetected);
                }
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn copy_cell(
        &mut self,
        copy_from_row: usize,
        copy_from_col: usize,
        copy_to_row: usize,
        copy_to_col: usize,
    ) -> Result<(), Error> {
        // Save original state of destination cell
        let original_cell = self.grid[copy_to_row][copy_to_col].clone();

        // Perform the copy operation
        match self
            .update_cell_data(
                copy_to_row,
                copy_to_col,
                self.get_formula(copy_from_row, copy_from_col),
            )
            .error
        {
            Error::None => {}
            Error::DivByZero => {
                self.grid[copy_to_row][copy_to_col]
                    .formula
                    .flag
                    .set_is_div_by_zero(1);
            }
            Error::InvalidInput => {
                self.grid[copy_to_row][copy_to_col]
                    .formula
                    .flag
                    .set_error(1);
            }
            Error::CycleDetected => {
                // Rollback all changes to prevent corrupted state
                self.grid[copy_to_row][copy_to_col] = original_cell;
                return Err(Error::CycleDetected);
            }
        }

        Ok(())
    }

    pub fn clear_row(&mut self, row: usize) {
        // Clear the entire row
        for col in 0..self.col {
            self.clear_cell(row, col);
        }
    }

    pub fn clear_col(&mut self, col: usize) {
        // Clear the entire column
        for row in 0..self.row {
            self.clear_cell(row, col);
        }
    }

    pub fn clear_cell(&mut self, row: usize, col: usize) {
        // Clear the cell's value and formula
        self.update_cell_data(row, col, "0".to_string());
        // update_cell_data marks the cell as touched (is_any); a cleared
        // cell should go back to reading as genuinely empty.
        self.grid[row][col].formula.flag.set_is_any(0);
    }

    fn set_dependicies_cell(&mut self, row: usize, col: usize, command: CommandCall) {
        if command.flag.type_() == 0 {
            if command.flag.type1() == 0 {
                self.grid[row][col].value = command.param1;
            } else if command.flag.type1() == 1 {
                let (param1_row, param1_col) = convert_to_index_int(command.param1);
                if !(self.grid[param1_row][param1_col]
                    .depend
                    .contains(&(row * ENCODE_SHIFT + col)))
                {
                    self.grid[param1_row][param1_col]
                        .depend
                        .push(row * ENCODE_SHIFT + col);
                }
            }
        } else if command.flag.type_() == 1 {
            if command.flag.type1() == 0 {
                if command.flag.type2() == 0 {
                    if command.flag.cmd() == 0 {
                        self.grid[row][col].value = command.param1 + command.param2;
                    } else if command.flag.cmd() == 1 {
                        self.grid[row][col].value = command.param1 - command.param2;
                    } else if command.flag.cmd() == 2 {
                        self.grid[row][col].value = command.param1 * command.param2;
                    } else if command.param2 == 0 {
                        self.grid[row][col].formula.flag.set_is_div_by_zero(1);
                    } else {
                        self.grid[row][col].value = command.param1 / command.param2;
                    }
                } else {
                    let (param2_row, param2_col) = convert_to_index_int(command.param2);
                    if !(self.grid[param2_row][param2_col]
                        .depend
                        .contains(&(row * ENCODE_SHIFT + col)))
                    {
                        self.grid[param2_row][param2_col]
                            .depend
                            .push(row * ENCODE_SHIFT + col);
                    }
                }
            } else if command.flag.type1() == 1 {
                let (param1_row, param1_col) = convert_to_index_int(command.param1);
                if !(self.grid[param1_row][param1_col]
                    .depend
                    .contains(&(row * ENCODE_SHIFT + col)))
                {
                    self.grid[param1_row][param1_col]
                        .depend
                        .push(row * ENCODE_SHIFT + col);
                }
                if command.flag.type2() == 0 {
                } else if command.flag.type2() == 1 {
                    let (param2_row, param2_col) = convert_to_index_int(command.param2);
                    if !(self.grid[param2_row][param2_col]
                        .depend
                        .contains(&(row * ENCODE_SHIFT + col)))
                    {
                        self.grid[param2_row][param2_col]
                            .depend
                            .push(row * ENCODE_SHIFT + col);
                    }
                }
            }
        } else if command.flag.type_() == 3 {
            // Text cells hold no cell references, so there are no
            // dependencies to register.
        } else if command.flag.cmd() == 5 {
            let (param1_row, param1_col) = convert_to_index_int(command.param1);
            if !(self.grid[param1_row][param1_col]
                .depend
                .contains(&(row * ENCODE_SHIFT + col)))
            {
                self.grid[param1_row][param1_col]
                    .depend
                    .push(row * ENCODE_SHIFT + col);
            }
        } else {
            let t = row * ENCODE_SHIFT + col;
            let (param1_row, param1_col) = convert_to_index_int(command.param1);
            let (param2_row, param2_col) = convert_to_index_int(command.param2);
            for i in param1_row..(param2_row + 1) {
                for j in param1_col..(param2_col + 1) {
                    let depend_vec = &mut self.grid[i][j].depend;
                    if !depend_vec.contains(&t) {
                        depend_vec.push(t);
                    }
                }
            }
        }

        self.grid[row][col].formula = command;
    }

    fn toposort(&self, target_cell: usize) -> Vec<usize> {
        let mut visited: FxHashSet<usize> = FxHashSet::default();
        let mut stack: FxHashSet<usize> = FxHashSet::default();
        let mut result: Vec<usize> = vec![];
        let is_cycle = self.dfs(target_cell, &mut visited, &mut stack, &mut result);

        if is_cycle {
            if DEBUG {
                println!("Cycle detected in the graph");
            }
            return vec![];
        }

        result.reverse();
        result
    }
    fn dfs(
        &self,
        cell: usize,
        visited: &mut FxHashSet<usize>,
        stack: &mut FxHashSet<usize>,
        result: &mut Vec<usize>,
    ) -> bool {
        if stack.contains(&cell) {
            return true;
        }
        if visited.contains(&cell) {
            return false;
        }

        visited.insert(cell);

        let col = cell % ENCODE_SHIFT;
        let row = cell / ENCODE_SHIFT;
        let mut is_cycle = false;
        stack.insert(cell);
        for &dep in &self.grid[row][col].depend {
            is_cycle = is_cycle || self.dfs(dep, visited, stack, result);
        }
        stack.remove(&cell);
        result.push(cell);
        is_cycle
    }

    fn minimum(&self, row1: usize, row2: usize, col1: usize, col2: usize) -> i32 {
        let mut min = i32::MAX;
        for i in row1..(row2 + 1) {
            for j in col1..(col2 + 1) {
                // No need to check for negative values as `i` and `j` are of type `usize`
                if self.grid[i][j].value < min {
                    min = self.grid[i][j].value;
                }
            }
        }
        min
    }
    fn maximum(&self, row1: usize, row2: usize, col1: usize, col2: usize) -> i32 {
        let mut max = i32::MIN;
        for i in row1..(row2 + 1) {
            for j in col1..(col2 + 1) {
                // println!("{}",self.grid[i as usize][j as usize].value);
                if self.grid[i][j].value > max {
                    max = self.grid[i][j].value;
                }
            }
        }
        max
    }
    fn average(&self, row1: usize, row2: usize, col1: usize, col2: usize) -> i32 {
        let mut sum = 0;
        let mut count = 0;
        for i in row1..(row2 + 1) {
            for j in col1..(col2 + 1) {
                sum += self.grid[i][j].value;
                count += 1;
            }
        }
        if count == 0 {
            return 0;
        }
        (sum as f32 / count as f32) as i32
    }
    fn sum(&self, row1: usize, row2: usize, col1: usize, col2: usize) -> i32 {
        let mut sum = 0;
        for i in row1..(row2 + 1) {
            for j in col1..(col2 + 1) {
                sum += self.grid[i][j].value;
            }
        }
        sum
    }
    fn stddev(&self, row1: usize, row2: usize, col1: usize, col2: usize) -> i32 {
        let mut mean = 0;
        let mut sum = 0;
        let mut count = 0;
        for i in row1..(row2 + 1) {
            for j in col1..(col2 + 1) {
                let value = self.grid[i][j].value;
                sum += (value) * (value);
                mean += value;
                count += 1;
            }
        }
        if count == 0 {
            return 0;
        }
        let avg = ((mean * mean) as f64) / ((count * count) as f64);
        let mut x = (sum as f64) / (count as f64);
        x -= avg;
        ((x).sqrt()).round() as i32
    }

    fn update_cell(&mut self, list_fpr_update: Vec<usize>) {
        for i in list_fpr_update {
            let col = i % ENCODE_SHIFT;
            let row = i / ENCODE_SHIFT;
            self.grid[row][col].formula.flag.set_is_div_by_zero(0);
            if self.grid[row][col].formula.flag.type_() == 0 {
                // value

                if self.grid[row][col].formula.flag.type1() == 0 {
                    self.grid[row][col].value = self.grid[row][col].formula.param1;
                } else if self.grid[row][col].formula.flag.type1() == 1 {
                    let (param1_row, param1_col) =
                        convert_to_index_int(self.grid[row][col].formula.param1);
                    if self.grid[param1_row][param1_col]
                        .formula
                        .flag
                        .is_div_by_zero()
                        == 1
                    {
                        self.grid[row][col].formula.flag.set_is_div_by_zero(1);
                    }
                    self.grid[row][col].value = self.grid[param1_row][param1_col].value;
                }
            } else if self.grid[row][col].formula.flag.type_() == 1 {
                // arithmatic
                if self.grid[row][col].formula.flag.type1() == 0 {
                    if self.grid[row][col].formula.flag.type2() == 0 {
                        if self.grid[row][col].formula.flag.cmd() == 0 {
                            self.grid[row][col].value = self.grid[row][col].formula.param1
                                + self.grid[row][col].formula.param2;
                        } else if self.grid[row][col].formula.flag.cmd() == 1 {
                            self.grid[row][col].value = self.grid[row][col].formula.param1
                                - self.grid[row][col].formula.param2;
                        } else if self.grid[row][col].formula.flag.cmd() == 2 {
                            self.grid[row][col].value = self.grid[row][col].formula.param1
                                * self.grid[row][col].formula.param2;
                        } else if self.grid[row][col].formula.param2 == 0 {
                            self.grid[row][col].formula.flag.set_is_div_by_zero(1);
                        } else {
                            self.grid[row][col].value = self.grid[row][col].formula.param1
                                / self.grid[row][col].formula.param2;
                        }
                    } else {
                        // let param2_row=(self.grid[row][col].formula.param2%1000) as usize;
                        // let param2_col=(self.grid[row][col].formula.param2/1000) as usize;
                        let (param2_row, param2_col) =
                            convert_to_index_int(self.grid[row][col].formula.param2);
                        if self.grid[param2_row][param2_col]
                            .formula
                            .flag
                            .is_div_by_zero()
                            == 1
                        {
                            self.grid[row][col].formula.flag.set_is_div_by_zero(1);
                        }
                        if self.grid[row][col].formula.flag.cmd() == 0 {
                            self.grid[row][col].value = self.grid[row][col].formula.param1
                                + self.grid[param2_row][param2_col].value;
                        } else if self.grid[row][col].formula.flag.cmd() == 1 {
                            self.grid[row][col].value = self.grid[row][col].formula.param1
                                - self.grid[param2_row][param2_col].value;
                        } else if self.grid[row][col].formula.flag.cmd() == 2 {
                            self.grid[row][col].value = self.grid[row][col].formula.param1
                                * self.grid[param2_row][param2_col].value;
                        } else if self.grid[param2_row][param2_col].value == 0 {
                            self.grid[row][col].formula.flag.set_is_div_by_zero(1);
                        } else {
                            self.grid[row][col].value = self.grid[row][col].formula.param1
                                / self.grid[param2_row][param2_col].value;
                        }
                    }
                } else if self.grid[row][col].formula.flag.type1() == 1 {
                    let (param1_row, param1_col) =
                        convert_to_index_int(self.grid[row][col].formula.param1);
                    if self.grid[param1_row][param1_col]
                        .formula
                        .flag
                        .is_div_by_zero()
                        == 1
                    {
                        self.grid[row][col].formula.flag.set_is_div_by_zero(1);
                    }
                    if self.grid[row][col].formula.flag.type2() == 0 {
                        if self.grid[row][col].formula.flag.cmd() == 0 {
                            self.grid[row][col].value = self.grid[param1_row][param1_col].value
                                + self.grid[row][col].formula.param2;
                        } else if self.grid[row][col].formula.flag.cmd() == 1 {
                            self.grid[row][col].value = self.grid[param1_row][param1_col].value
                                - self.grid[row][col].formula.param2;
                        } else if self.grid[row][col].formula.flag.cmd() == 2 {
                            self.grid[row][col].value = self.grid[param1_row][param1_col].value
                                * self.grid[row][col].formula.param2;
                        } else if self.grid[row][col].formula.param2 == 0 {
                            self.grid[row][col].formula.flag.set_is_div_by_zero(1);
                        } else {
                            self.grid[row][col].value = self.grid[param1_row][param1_col].value
                                / self.grid[row][col].formula.param2;
                        }
                    } else if self.grid[row][col].formula.flag.type2() == 1 {
                        let (param2_row, param2_col) =
                            convert_to_index_int(self.grid[row][col].formula.param2);
                        if self.grid[param2_row][param2_col]
                            .formula
                            .flag
                            .is_div_by_zero()
                            == 1
                        {
                            self.grid[row][col].formula.flag.set_is_div_by_zero(1);
                        }
                        if self.grid[row][col].formula.flag.cmd() == 0 {
                            self.grid[row][col].value = self.grid[param1_row][param1_col].value
                                + self.grid[param2_row][param2_col].value;
                        } else if self.grid[row][col].formula.flag.cmd() == 1 {
                            self.grid[row][col].value = self.grid[param1_row][param1_col].value
                                - self.grid[param2_row][param2_col].value;
                        } else if self.grid[row][col].formula.flag.cmd() == 2 {
                            self.grid[row][col].value = self.grid[param1_row][param1_col].value
                                * self.grid[param2_row][param2_col].value;
                        } else if self.grid[param2_row][param2_col].value == 0 {
                            self.grid[row][col].formula.flag.set_is_div_by_zero(1);
                        } else {
                            self.grid[row][col].value = self.grid[param1_row][param1_col].value
                                / self.grid[param2_row][param2_col].value;
                        }
                        // self.grid[row][col].value=self.grid[param1_row][param1_col].value/self.grid[param2_row][param2_col].value;
                    }
                }
            } else if self.grid[row][col].formula.flag.type_() == 3 {
                // Text cells have a fixed value of 0; there is nothing to
                // recompute from their (unused) formula parameters.
            } else {
                let (param1_row, param1_col) =
                    convert_to_index_int(self.grid[row][col].formula.param1);
                if self.grid[param1_row][param1_col]
                    .formula
                    .flag
                    .is_div_by_zero()
                    == 1
                {
                    self.grid[row][col].formula.flag.set_is_div_by_zero(1);
                }
                let (param2_row, param2_col) =
                    convert_to_index_int(self.grid[row][col].formula.param2);
                if self.grid[param2_row][param2_col]
                    .formula
                    .flag
                    .is_div_by_zero()
                    == 1
                {
                    self.grid[row][col].formula.flag.set_is_div_by_zero(1);
                }
                if self.grid[row][col].formula.flag.cmd() == 0 {
                    self.grid[row][col].value =
                        self.minimum(param1_row, param2_row, param1_col, param2_col);
                } else if self.grid[row][col].formula.flag.cmd() == 1 {
                    self.grid[row][col].value =
                        self.maximum(param1_row, param2_row, param1_col, param2_col);
                } else if self.grid[row][col].formula.flag.cmd() == 2 {
                    self.grid[row][col].value =
                        self.sum(param1_row, param2_row, param1_col, param2_col);
                } else if self.grid[row][col].formula.flag.cmd() == 3 {
                    self.grid[row][col].value =
                        self.average(param1_row, param2_row, param1_col, param2_col);
                } else if self.grid[row][col].formula.flag.cmd() == 4 {
                    self.grid[row][col].value =
                        self.stddev(param1_row, param2_row, param1_col, param2_col);
                } else if self.grid[row][col].formula.flag.cmd() == 5 {
                    if self.grid[row][col].formula.flag.type1() == 1 {
                        let time_secs = time::Duration::from_secs(
                            self.grid[param1_row][param1_col].value as u64,
                        );
                        thread::sleep(time_secs);
                        self.grid[row][col].value = self.grid[param1_row][param1_col].value;
                    } else {
                        self.grid[row][col].value = self.grid[row][col].formula.param1;
                        let time_secs = time::Duration::from_secs(self.grid[row][col].value as u64);
                        thread::sleep(time_secs);
                    }
                }
            }
        }
    }
    fn remove_old_dependicies(&mut self, row: usize, col: usize, restore_command: CommandCall) {
        // Remove all dependencies from previous formula
        let curr_index = row * ENCODE_SHIFT + col;
        let current_command = &self.grid[row][col].formula.clone();
        // Remove dependencies based on command type
        if current_command.flag.type_() == 0 && current_command.flag.type1() == 1 {
            // Cell reference dependency

            let (param1_row, param1_col) = convert_to_index_int(current_command.param1);
            let depend_vec = &mut self.grid[param1_row][param1_col].depend;
            depend_vec.retain(|&x| x != curr_index);
        } else if current_command.flag.type_() == 1 {
            // Arithmetic operation dependencies
            if current_command.flag.type1() == 1 {
                // First parameter is a cell reference
                let (param1_row, param1_col) = convert_to_index_int(current_command.param1);
                let depend_vec = &mut self.grid[param1_row][param1_col].depend;
                depend_vec.retain(|&x| x != curr_index);
            }
            if current_command.flag.type2() == 1 {
                // Second parameter is a cell reference
                let (param2_row, param2_col) = convert_to_index_int(current_command.param2);
                let depend_vec = &mut self.grid[param2_row][param2_col].depend;
                depend_vec.retain(|&x| x != curr_index);
                // depend_vec.remove(&curr_index);
            }
        } else if current_command.flag.type_() == 2 {
            // Range function dependencies
            let (param1_row, param1_col) = convert_to_index_int(current_command.param1);
            let (param2_row, param2_col) = convert_to_index_int(current_command.param2);
            for i in param1_row..(param2_row + 1) {
                for j in param1_col..(param2_col + 1) {
                    let depend_vec = &mut self.grid[i][j].depend;
                    depend_vec.retain(|&x| x != curr_index);
                    // depend_vec.remove(&curr_index);
                }
            }
        }

        // // Set the cell's formula to the restore command
        // self.grid[row][col].formula = restore_command;
        // Restore the cell's value to the original value
        self.set_dependicies_cell(row, col, restore_command.clone());
    }
    /// Stores `text` as literal, unevaluated content in a cell: no formula,
    /// no dependencies, and a fixed value of 0 for any cell that references it.
    fn set_text_cell(
        &mut self,
        row: usize,
        col: usize,
        text: &str,
        start_total: time::Instant,
    ) -> CallResult {
        let mut text_command = CommandCall {
            flag: CommandFlag::new(),
            param1: 0,
            param2: 0,
        };
        text_command.flag.set_type_(3);
        text_command.flag.set_is_any(1);

        self.remove_old_dependicies(row, col, text_command);
        self.grid[row][col].text = text.to_string();
        self.grid[row][col].value = 0;

        // Cells that depended on the old value need to see it become 0.
        let topo_vec = self.toposort(row * ENCODE_SHIFT + col);
        if !topo_vec.is_empty() {
            self.update_cell(topo_vec);
        }

        CallResult {
            time: start_total.elapsed().as_millis() as f64,
            error: Error::None,
        }
    }

    pub fn update_cell_data(&mut self, row: usize, col: usize, new_formula: String) -> CallResult {
        // Overall timing
        let start_total = time::Instant::now();

        let trimmed = new_formula.trim();

        // Excel-style: a formula must start with '=' (stripped before parsing).
        // Without it, a bare number is still a number (so "42" just works),
        // but anything else is stored as literal text rather than being
        // auto-detected as a formula.
        let formula_input = if let Some(rest) = trimmed.strip_prefix('=') {
            rest.trim()
        } else if trimmed.parse::<i32>().is_ok() {
            trimmed
        } else {
            return self.set_text_cell(row, col, trimmed, start_total);
        };

        // Stage 1: Parse formula
        let mut command = parse_formula(formula_input);
        let (row1, col1) = convert_to_index_int(command.param1);
        let (row2, col2) = convert_to_index_int(command.param2);
        if row1 > self.row || col1 > self.col || row2 > self.row || col2 > self.col {
            command.flag.set_error(1);
        }
        if command.flag.error() == 0 {
            command.flag.set_is_any(1);
            // Stage 2: Save old command and set dependencies
            let old_command = self.grid[row][col].formula.clone();
            self.remove_old_dependicies(row, col, command.clone());
            // Stage 3: Topological sort
            let topo_vec = self.toposort(row * ENCODE_SHIFT + col);
            // Stage 4: Update cells
            if topo_vec.is_empty() {
                self.grid[row][col].formula.flag.set_error(2);
            } else {
                self.update_cell(topo_vec);
            }
            let mut ans = CallResult {
                time: start_total.elapsed().as_millis() as f64,
                error: Error::None,
            };

            if self.grid[row][col].formula.flag.is_div_by_zero() == 1 {
                ans.error = Error::DivByZero;
            } else if self.grid[row][col].formula.flag.error() == 1 {
                ans.error = Error::InvalidInput;
            } else if self.grid[row][col].formula.flag.error() == 2 {
                ans.error = Error::CycleDetected;
                self.remove_old_dependicies(row, col, old_command);
            }

            ans
        } else {
            CallResult {
                time: start_total.elapsed().as_millis() as f64,
                error: Error::InvalidInput,
            }
        }
    }

    pub fn get_value(&self, row: i32, col: i32) -> i32 {
        self.grid[row as usize][col as usize].value
    }

    /// Returns the literal text of a cell if it holds plain text rather than
    /// a number or a formula, `None` otherwise.
    pub fn get_text(&self, row: i32, col: i32) -> Option<String> {
        let cell = &self.grid[row as usize][col as usize];
        if cell.formula.flag.type_() == 3 {
            Some(cell.text.clone())
        } else {
            None
        }
    }

    /// Returns `true` if a cell has never been explicitly set (via
    /// `update_cell_data`) or has since been cleared - as opposed to a cell
    /// that was set to the number 0.
    pub fn is_empty_cell(&self, row: i32, col: i32) -> bool {
        self.grid[row as usize][col as usize]
            .formula
            .flag
            .is_any()
            == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_avg() {
        let mut test_sheet = Sheet::new(6, 6);
        test_sheet.update_cell_data(1, 1, String::from("=AVG(A2:D5)"));
        test_sheet.update_cell_data(2, 2, String::from("500"));
        println!("{}", test_sheet.get_value(1, 1));
    }
    #[test]
    fn test_sum() {
        let mut test_sheet = Sheet::new(6, 6);
        test_sheet.update_cell_data(1, 1, String::from("=SUM(A2:D5)"));
        test_sheet.update_cell_data(2, 1, String::from("=B1+B5"));
        test_sheet.update_cell_data(2, 2, String::from("=A2+A3"));
        test_sheet.update_cell_data(5, 2, String::from("10"));
        test_sheet.update_cell_data(1, 2, String::from("5"));
        test_sheet.update_cell_data(3, 1, String::from("6"));
        assert_eq!(test_sheet.get_value(1, 1), 52);
    }
    #[test]
    fn test_max() {
        let mut test_sheet = Sheet::new(6, 6);
        test_sheet.update_cell_data(1, 1, String::from("=MAX(A2:D5)"));
        test_sheet.update_cell_data(2, 1, String::from("=B1+B5"));
        test_sheet.update_cell_data(2, 2, String::from("=A2+A3"));
        test_sheet.update_cell_data(5, 2, String::from("10"));
        test_sheet.update_cell_data(1, 2, String::from("-5"));
        test_sheet.update_cell_data(3, 1, String::from("6"));
        assert_eq!(test_sheet.get_value(1, 1), 11);
    }
    #[test]
    fn test_min() {
        let mut test_sheet = Sheet::new(6, 6);
        test_sheet.update_cell_data(1, 1, String::from("=MIN(A2:D5)"));
        test_sheet.update_cell_data(2, 1, String::from("=B1+B5"));
        test_sheet.update_cell_data(2, 2, String::from("=A2+A3"));
        test_sheet.update_cell_data(5, 2, String::from("10"));
        test_sheet.update_cell_data(1, 2, String::from("-5"));
        test_sheet.update_cell_data(3, 1, String::from("6"));
        assert_eq!(test_sheet.get_value(1, 1), 0);
    }

    #[test]
    fn test_stdev() {
        let mut test_sheet = Sheet::new(6, 6);
        test_sheet.update_cell_data(1, 1, String::from("=STDEV(A2:D5)"));
        test_sheet.update_cell_data(2, 1, String::from("=B1+B5"));
        test_sheet.update_cell_data(2, 2, String::from("=A2+A3"));
        test_sheet.update_cell_data(5, 2, String::from("10"));
        test_sheet.update_cell_data(1, 2, String::from("-5"));
        test_sheet.update_cell_data(3, 1, String::from("6"));
        assert_eq!(test_sheet.get_value(1, 1), 4);
    }

    #[test]
    fn test_multiply() {
        let mut test_sheet = Sheet::new(6, 6);
        test_sheet.update_cell_data(1, 1, String::from("=A2*B2"));
        test_sheet.update_cell_data(2, 1, String::from("=B1+B5"));
        test_sheet.update_cell_data(2, 2, String::from("=A2+A3"));
        test_sheet.update_cell_data(5, 2, String::from("10"));
        test_sheet.update_cell_data(1, 2, String::from("-5"));
        test_sheet.update_cell_data(3, 1, String::from("6"));
        assert_eq!(test_sheet.get_value(1, 1), 55);
    }

    #[test]
    fn test_divide() {
        let mut test_sheet = Sheet::new(6, 6);
        test_sheet.update_cell_data(1, 1, String::from("=A2/B2"));
        test_sheet.update_cell_data(2, 1, String::from("=B1+B5"));
        test_sheet.update_cell_data(2, 2, String::from("=A2+A3"));
        test_sheet.update_cell_data(5, 2, String::from("10"));
        test_sheet.update_cell_data(1, 2, String::from("-5"));
        test_sheet.update_cell_data(3, 1, String::from("6"));
        assert_eq!(test_sheet.get_value(1, 1), 0);
    }

    #[test]
    fn test_large_cell() {
        let mut test_sheet = Sheet::new(703, 703);
        test_sheet.update_cell_data(1, 1, String::from("=ZZ29"));
        test_sheet.update_cell_data(29, 702, String::from("29"));
        println!("{}", test_sheet.get_value(1, 1));
        assert!(test_sheet.get_value(1, 1) == 29);
    }
    #[test]
    fn check_cycle() {
        let mut test_sheet = Sheet::new(6, 6);
        test_sheet.update_cell_data(1, 1, String::from("=A2"));
        test_sheet.update_cell_data(2, 1, String::from("=B1+B5"));
        test_sheet.update_cell_data(2, 2, String::from("=A2+A3"));
        test_sheet.update_cell_data(5, 2, String::from("10"));
        test_sheet.update_cell_data(1, 2, String::from("-5"));
        test_sheet.update_cell_data(3, 1, String::from("6"));
    }
    #[test]
    fn error_detected1() {
        let mut test_sheet = Sheet::new(6, 6);
        test_sheet.update_cell_data(1, 1, String::from("=A2+A3"));
        test_sheet.update_cell_data(2, 1, String::from("90"));
        test_sheet.update_cell_data(3, 1, String::from("50"));
        test_sheet.update_cell_data(3, 1, String::from("=A1+A2"));
        test_sheet.update_cell_data(3, 1, String::from("-5"));
        test_sheet.update_cell_data(3, 1, String::from("6"));
        assert!(test_sheet.get_value(1, 1) == 96);
    }
    #[test]
    fn boundry_check() {
        let mut test_sheet = Sheet::new(6, 6);
        test_sheet.update_cell_data(0, 1, String::from("45"));
        test_sheet.update_cell_data(0, 0, String::from("45"));
        test_sheet.update_cell_data(1, 0, String::from("45"));
        assert!(test_sheet.get_value(0, 0) == 45);
        // test_sheet.update_cell_data(0, 6, String::from ("45"));
    }

    #[test]
    fn test_copy_col() {
        let mut test_sheet = Sheet::new(10, 10);
        test_sheet.update_cell_data(1, 1, String::from("=A2"));
        test_sheet.update_cell_data(2, 1, String::from("=B1+B5"));
        test_sheet.update_cell_data(2, 2, String::from("=A2+A3"));
        test_sheet.update_cell_data(5, 2, String::from("10"));
        test_sheet.update_cell_data(1, 2, String::from("-5"));
        test_sheet.update_cell_data(3, 1, String::from("6"));
        let _ = test_sheet.copy_col(1, 3);

        assert!(test_sheet.get_value(1, 3) == 5);
        assert!(test_sheet.get_value(2, 3) == 5);
        assert!(test_sheet.get_value(3, 3) == 6);

        assert!(test_sheet.get_value(1, 2) == -5);
        assert!(test_sheet.get_value(2, 2) == 11);
        assert!(test_sheet.get_value(3, 2) == 0);
        assert!(test_sheet.get_value(4, 2) == 0);
        assert!(test_sheet.get_value(5, 2) == 10);

        assert!(test_sheet.get_value(1, 1) == 5);
        assert!(test_sheet.get_value(2, 1) == 5);
        assert!(test_sheet.get_value(3, 1) == 6);
    }

    #[test]
    fn test_copy_cell() {
        let mut test_sheet = Sheet::new(10, 10);
        test_sheet.update_cell_data(1, 1, String::from("=A2"));
        test_sheet.update_cell_data(2, 1, String::from("=B1+B5"));
        test_sheet.update_cell_data(2, 2, String::from("=A2+A3"));
        test_sheet.update_cell_data(5, 2, String::from("10"));
        test_sheet.update_cell_data(1, 2, String::from("-5"));
        test_sheet.update_cell_data(3, 1, String::from("6"));
        let _ = test_sheet.copy_cell(1, 1, 2, 2);
        assert!(test_sheet.get_value(2, 2) == 5);
        assert!(test_sheet.get_value(1, 1) == 5);
        assert!(test_sheet.get_value(2, 1) == 5);
        assert!(test_sheet.get_value(3, 1) == 6);
        assert!(test_sheet.get_value(5, 2) == 10);
    }

    #[test]
    fn test_values() {
        let mut test_sheet = Sheet::new(10, 10);
        test_sheet.update_cell_data(1, 1, String::from("=5+60"));
        test_sheet.update_cell_data(2, 1, String::from("=-5*70"));
        test_sheet.update_cell_data(2, 2, String::from("=35/7"));
        test_sheet.update_cell_data(5, 2, String::from("=35-90"));
        assert!(test_sheet.get_value(1, 1) == 65);
        assert!(test_sheet.get_value(2, 1) == -350);
        assert!(test_sheet.get_value(2, 2) == 5);
        assert!(test_sheet.get_value(5, 2) == -55);

        test_sheet.update_cell_data(6, 3, String::from("5"));
        test_sheet.update_cell_data(1, 1, String::from("=5+C6"));
        test_sheet.update_cell_data(2, 1, String::from("=-5*C6"));
        test_sheet.update_cell_data(2, 2, String::from("=35/C6"));
        test_sheet.update_cell_data(5, 2, String::from("=35-C6"));
        assert!(test_sheet.get_value(1, 1) == 10);
        assert!(test_sheet.get_value(2, 1) == -25);
        assert!(test_sheet.get_value(2, 2) == 7);
        assert!(test_sheet.get_value(5, 2) == 30);
    }

    #[test]
    fn test_copy_row() {
        let mut test_sheet = Sheet::new(10, 10);
        test_sheet.update_cell_data(1, 1, String::from("=A2"));
        test_sheet.update_cell_data(2, 1, String::from("=B1+B5"));
        test_sheet.update_cell_data(1, 2, String::from("-5"));
        test_sheet.update_cell_data(1, 3, String::from("=SUM(A2:B5)"));
        test_sheet.update_cell_data(1, 4, String::from("=MAX(A2:B5)"));

        // Test copy row
        let result = test_sheet.copy_row(1, 3);
        assert!(result.is_ok());

        // Verify copied values
        assert_eq!(test_sheet.get_value(3, 1), -5);
        assert_eq!(test_sheet.get_value(3, 2), -5); // From B1 in original row
        assert_eq!(test_sheet.get_value(3, 3), -15); // From C1 in original row (SUM)
        assert_eq!(test_sheet.get_value(3, 4), 0); // From D1 in original row (MAX)
    }

    #[test]
    fn test_copy_row_cycle_detection() {
        let mut test_sheet = Sheet::new(10, 10);
        test_sheet.update_cell_data(1, 1, String::from("=A3")); // A1 references A3
        test_sheet.update_cell_data(3, 1, String::from("42"));

        // This should fail because copying row 1 to row 2 would create
        // a cycle when A3 is later updated to reference A2
        let result = test_sheet.copy_row(1, 2);
        assert!(result.is_ok());

        // Now update A3 to reference A2, which would create a cycle
        let update_result = test_sheet.update_cell_data(3, 1, String::from("=A2"));
        assert_eq!(update_result.error, Error::CycleDetected);
    }

    #[test]
    fn test_clear_operations() {
        let mut test_sheet = Sheet::new(10, 10);

        // Set some values
        test_sheet.update_cell_data(1, 1, String::from("42"));
        test_sheet.update_cell_data(1, 2, String::from("84"));
        test_sheet.update_cell_data(2, 1, String::from("21"));
        test_sheet.update_cell_data(2, 2, String::from("=A1+B1"));

        // Test clear cell
        test_sheet.clear_cell(1, 1);
        assert_eq!(test_sheet.get_value(1, 1), 0);

        // Test clear row
        test_sheet.clear_row(2);
        assert_eq!(test_sheet.get_value(2, 1), 0);
        assert_eq!(test_sheet.get_value(2, 2), 0);

        // Test clear column
        test_sheet.clear_col(2);
        assert_eq!(test_sheet.get_value(1, 2), 0);
    }

    #[test]
    fn test_dependency_handling() {
        let mut test_sheet = Sheet::new(10, 10);

        // Create a chain of dependencies
        test_sheet.update_cell_data(1, 1, String::from("42"));
        test_sheet.update_cell_data(1, 2, String::from("=A1*2"));
        test_sheet.update_cell_data(1, 3, String::from("=B1*2"));

        // Verify initial values
        assert_eq!(test_sheet.get_value(1, 1), 42);
        assert_eq!(test_sheet.get_value(1, 2), 84);
        assert_eq!(test_sheet.get_value(1, 3), 168);

        // Change root value and verify propagation
        test_sheet.update_cell_data(1, 1, String::from("10"));
        assert_eq!(test_sheet.get_value(1, 1), 10);
        assert_eq!(test_sheet.get_value(1, 2), 20);
        assert_eq!(test_sheet.get_value(1, 3), 40);

        // Change formula and verify dependency updates
        test_sheet.update_cell_data(1, 2, String::from("=A1+5"));
        assert_eq!(test_sheet.get_value(1, 2), 15);
        assert_eq!(test_sheet.get_value(1, 3), 30);
    }

    #[test]
    fn test_arithmetic_with_div_by_zero() {
        let mut test_sheet = Sheet::new(10, 10);

        // Set up division by zero scenarios
        test_sheet.update_cell_data(1, 1, String::from("42"));
        test_sheet.update_cell_data(1, 2, String::from("0"));

        // Test direct division by zero
        let result = test_sheet.update_cell_data(1, 3, String::from("=A1/0"));
        assert_eq!(result.error, Error::DivByZero);

        // Test division by cell containing zero
        let result = test_sheet.update_cell_data(1, 4, String::from("=A1/B1"));
        assert_eq!(result.error, Error::DivByZero);

        // Test formula that references cell with division by zero
        test_sheet.update_cell_data(1, 5, String::from("=D1*2"));
        assert_eq!(test_sheet.grid[1][5].formula.flag.is_div_by_zero(), 1);
    }

    #[test]
    fn test_range_functions_with_negative_values() {
        let mut test_sheet = Sheet::new(6, 6);

        // Set up cells with negative values
        test_sheet.update_cell_data(2, 1, String::from("-10"));
        test_sheet.update_cell_data(2, 2, String::from("-20"));
        test_sheet.update_cell_data(3, 1, String::from("-30"));
        test_sheet.update_cell_data(3, 2, String::from("40"));

        // Test MIN with negative values
        test_sheet.update_cell_data(1, 1, String::from("=MIN(A2:B3)"));
        assert_eq!(test_sheet.get_value(1, 1), -30);

        // Test MAX with negative values
        test_sheet.update_cell_data(1, 2, String::from("=MAX(A2:B3)"));
        assert_eq!(test_sheet.get_value(1, 2), 40);

        // Test SUM with negative values
        test_sheet.update_cell_data(1, 3, String::from("=SUM(A2:B3)"));
        assert_eq!(test_sheet.get_value(1, 3), -20); // -10 + -20 + -30 + 40 = -20

        // Test AVG with negative values
        test_sheet.update_cell_data(1, 4, String::from("=AVG(A2:B3)"));
        assert_eq!(test_sheet.get_value(1, 4), -5); // (-10 + -20 + -30 + 40) / 4 = -5
    }

    #[test]
    fn test_sleep_function() {
        let mut test_sheet = Sheet::new(5, 5);

        // Use a small sleep duration to not slow down tests
        test_sheet.update_cell_data(1, 1, String::from("1"));

        // Test SLEEP with constant
        let start = time::Instant::now();
        test_sheet.update_cell_data(1, 2, String::from("=SLEEP(1)"));
        let elapsed = start.elapsed();
        assert!(elapsed.as_secs() >= 1);

        // Test SLEEP with cell reference
        let start = time::Instant::now();
        test_sheet.update_cell_data(1, 3, String::from("=SLEEP(A1)"));
        let elapsed = start.elapsed();
        assert!(elapsed.as_secs() >= 1);
    }

    #[test]
    fn test_complex_dependencies() {
        let mut test_sheet = Sheet::new(10, 10);

        // Create complex dependency chains
        test_sheet.update_cell_data(1, 1, String::from("10"));
        test_sheet.update_cell_data(1, 2, String::from("20"));
        test_sheet.update_cell_data(1, 3, String::from("=A1+B1"));
        test_sheet.update_cell_data(2, 1, String::from("=C1*2"));
        test_sheet.update_cell_data(2, 2, String::from("=A2/2"));
        test_sheet.update_cell_data(2, 3, String::from("=SUM(A1:B2)"));

        // Verify initial values
        assert_eq!(test_sheet.get_value(1, 3), 30); // A1+B1 = 10+20 = 30
        assert_eq!(test_sheet.get_value(2, 1), 60); // C1*2 = 30*2 = 60
        assert_eq!(test_sheet.get_value(2, 2), 30); // A2/2 = 60/2 = 30
        assert_eq!(test_sheet.get_value(2, 3), 120); // SUM(A1:B2) = 10+20+60+30 = 120

        // Change root value and verify propagation
        test_sheet.update_cell_data(1, 1, String::from("5"));
        assert_eq!(test_sheet.get_value(1, 3), 25); // A1+B1 = 5+20 = 25
        assert_eq!(test_sheet.get_value(2, 1), 50); // C1*2 = 25*2 = 50
        assert_eq!(test_sheet.get_value(2, 2), 25); // A2/2 = 50/2 = 25
        assert_eq!(test_sheet.get_value(2, 3), 100); // SUM(A1:B2) = 5+20+50+25 = 100
    }

    #[test]
    fn test_formula_with_all_types() {
        let mut test_sheet = Sheet::new(10, 10);

        // Set up test cells
        test_sheet.update_cell_data(1, 1, String::from("10"));
        test_sheet.update_cell_data(1, 2, String::from("20"));

        // Type 0 type1=0: constant
        test_sheet.update_cell_data(2, 1, String::from("42"));
        assert_eq!(test_sheet.get_value(2, 1), 42);

        // Type 0 type1=1: cell reference
        test_sheet.update_cell_data(2, 2, String::from("=A1"));
        assert_eq!(test_sheet.get_value(2, 2), 10);

        // Type 1 type1=0 type2=0: two constants
        test_sheet.update_cell_data(3, 1, String::from("=5+7"));
        assert_eq!(test_sheet.get_value(3, 1), 12);

        // Type 1 type1=0 type2=1: constant and cell
        test_sheet.update_cell_data(3, 2, String::from("=5+A1"));
        assert_eq!(test_sheet.get_value(3, 2), 15);

        // Type 1 type1=1 type2=0: cell and constant
        test_sheet.update_cell_data(4, 1, String::from("=A1+5"));
        assert_eq!(test_sheet.get_value(4, 1), 15);

        // Type 1 type1=1 type2=1: two cells
        test_sheet.update_cell_data(4, 2, String::from("=A1+B1"));
        assert_eq!(test_sheet.get_value(4, 2), 30);

        // Type 2: range function
        test_sheet.update_cell_data(5, 1, String::from("=SUM(A1:B1)"));
        assert_eq!(test_sheet.get_value(5, 1), 30);
    }

    #[test]
    fn test_operations_with_div_by_zero_propagation() {
        let mut test_sheet = Sheet::new(8, 8);

        // Create cells with division by zero
        test_sheet.update_cell_data(1, 1, String::from("10"));
        test_sheet.update_cell_data(1, 2, String::from("0"));
        test_sheet.update_cell_data(1, 3, String::from("=A1/B1")); // Division by zero

        // References to div by zero cells
        test_sheet.update_cell_data(2, 1, String::from("=C1+5"));
        test_sheet.update_cell_data(2, 2, String::from("=C1*2"));
        test_sheet.update_cell_data(2, 3, String::from("=5+C1"));
        test_sheet.update_cell_data(2, 4, String::from("=2*C1"));

        // Range functions using div by zero cells
        test_sheet.update_cell_data(3, 1, String::from("=SUM(A1:C1)"));
        test_sheet.update_cell_data(3, 2, String::from("=AVG(A1:C1)"));
        test_sheet.update_cell_data(3, 3, String::from("=MIN(A1:C1)"));
        test_sheet.update_cell_data(3, 4, String::from("=MAX(A1:C1)"));
        test_sheet.update_cell_data(3, 5, String::from("=STDEV(A1:C1)"));

        // Check that div by zero flag propagated
        assert_eq!(test_sheet.grid[1][3].formula.flag.is_div_by_zero(), 1);
        assert_eq!(test_sheet.grid[2][1].formula.flag.is_div_by_zero(), 1);
        assert_eq!(test_sheet.grid[2][2].formula.flag.is_div_by_zero(), 1);
        assert_eq!(test_sheet.grid[2][3].formula.flag.is_div_by_zero(), 1);
        assert_eq!(test_sheet.grid[2][4].formula.flag.is_div_by_zero(), 1);
        assert_eq!(test_sheet.grid[3][1].formula.flag.is_div_by_zero(), 1);
        assert_eq!(test_sheet.grid[3][2].formula.flag.is_div_by_zero(), 1);
        assert_eq!(test_sheet.grid[3][3].formula.flag.is_div_by_zero(), 1);
        assert_eq!(test_sheet.grid[3][4].formula.flag.is_div_by_zero(), 1);
        assert_eq!(test_sheet.grid[3][5].formula.flag.is_div_by_zero(), 1);
    }

    #[test]
    fn test_empty_range() {
        let mut test_sheet = Sheet::new(5, 5);

        // Test range functions on an "empty" range (all cells 0)
        test_sheet.update_cell_data(1, 1, String::from("=SUM(A2:B3)"));
        assert_eq!(test_sheet.get_value(1, 1), 0);

        test_sheet.update_cell_data(1, 2, String::from("=AVG(A2:B3)"));
        assert_eq!(test_sheet.get_value(1, 2), 0);

        test_sheet.update_cell_data(1, 3, String::from("=MIN(A2:B3)"));
        assert_eq!(test_sheet.get_value(1, 3), 0);

        test_sheet.update_cell_data(1, 4, String::from("=MAX(A2:B3)"));
        assert_eq!(test_sheet.get_value(1, 4), 0);

        test_sheet.update_cell_data(1, 5, String::from("=STDEV(A2:B3)"));
        assert_eq!(test_sheet.get_value(1, 5), 0);
    }

    #[test]
    fn test_update_with_invalid_input() {
        let mut test_sheet = Sheet::new(5, 5);

        // Invalid formula
        let result = test_sheet.update_cell_data(1, 1, String::from("=A1++A2"));
        assert_eq!(result.error, Error::InvalidInput);

        // Invalid range formula
        let result = test_sheet.update_cell_data(1, 2, String::from("=SUM(A1:)"));
        assert_eq!(result.error, Error::InvalidInput);

        // Invalid cell reference
        let result = test_sheet.update_cell_data(1, 3, String::from("=Z99"));
        assert_eq!(result.error, Error::InvalidInput);
    }

    #[test]
    fn test_update_formula_with_same_dependencies() {
        let mut test_sheet = Sheet::new(5, 5);

        test_sheet.update_cell_data(1, 1, String::from("10"));
        test_sheet.update_cell_data(1, 2, String::from("20"));

        // Create a formula with dependencies
        test_sheet.update_cell_data(2, 1, String::from("=A1+B1"));
        assert_eq!(test_sheet.get_value(2, 1), 30);

        // Update to a different formula with the same dependencies
        test_sheet.update_cell_data(2, 1, String::from("=A1*B1"));
        assert_eq!(test_sheet.get_value(2, 1), 200);

        // Dependencies should still work
        test_sheet.update_cell_data(1, 1, String::from("5"));
        assert_eq!(test_sheet.get_value(2, 1), 100);
    }

    #[test]
    fn test_get_range_statistics() {
        // Add the method to Sheet struct if not already present
        let mut test_sheet = Sheet::new(5, 5);

        // Set up test data
        test_sheet.update_cell_data(1, 1, String::from("10"));
        test_sheet.update_cell_data(1, 2, String::from("20"));
        test_sheet.update_cell_data(2, 1, String::from("15"));
        test_sheet.update_cell_data(2, 2, String::from("25"));
    }

    #[test]
    fn test_text_cell_basic() {
        let mut test_sheet = Sheet::new(5, 5);

        let result = test_sheet.update_cell_data(1, 1, String::from("Hello World"));
        assert_eq!(result.error, Error::None);
        assert_eq!(test_sheet.get_text(1, 1), Some("Hello World".to_string()));
        assert_eq!(test_sheet.get_value(1, 1), 0);
        assert_eq!(test_sheet.get_formula(1, 1), "Hello World");
    }

    #[test]
    fn test_bare_formula_without_equals_is_text() {
        let mut test_sheet = Sheet::new(5, 5);

        // Without a leading '=', this is no longer auto-detected as a formula.
        test_sheet.update_cell_data(1, 1, String::from("42"));
        test_sheet.update_cell_data(2, 1, String::from("A1"));
        assert_eq!(test_sheet.get_text(2, 1), Some("A1".to_string()));
        assert_eq!(test_sheet.get_value(2, 1), 0);
    }

    #[test]
    fn test_explicit_formula_with_equals() {
        let mut test_sheet = Sheet::new(5, 5);

        test_sheet.update_cell_data(1, 1, String::from("10"));
        let result = test_sheet.update_cell_data(2, 1, String::from("=A1+5"));
        assert_eq!(result.error, Error::None);
        assert_eq!(test_sheet.get_text(2, 1), None);
        assert_eq!(test_sheet.get_value(2, 1), 15);
        assert_eq!(test_sheet.get_formula(2, 1), "=A1+5");
    }

    #[test]
    fn test_text_cell_referenced_by_formula_is_zero() {
        let mut test_sheet = Sheet::new(5, 5);

        test_sheet.update_cell_data(1, 1, String::from("Revenue"));
        test_sheet.update_cell_data(2, 1, String::from("=A1+5"));
        assert_eq!(test_sheet.get_value(2, 1), 5);
    }

    #[test]
    fn test_text_cell_updates_dependents_on_change() {
        let mut test_sheet = Sheet::new(5, 5);

        test_sheet.update_cell_data(1, 1, String::from("10"));
        test_sheet.update_cell_data(2, 1, String::from("=A1*2"));
        assert_eq!(test_sheet.get_value(2, 1), 20);

        // Overwriting the numeric source with text should propagate to
        // dependents (their reference now resolves to 0).
        test_sheet.update_cell_data(1, 1, String::from("Not a number"));
        assert_eq!(test_sheet.get_value(2, 1), 0);
    }

    #[test]
    fn test_invalid_explicit_formula_still_errors() {
        let mut test_sheet = Sheet::new(5, 5);

        // An explicit '=' signals formula intent, so unparseable content
        // should still surface as an error rather than silently becoming text.
        let result = test_sheet.update_cell_data(1, 1, String::from("=NotAFormula"));
        assert_eq!(result.error, Error::InvalidInput);
        assert_eq!(test_sheet.get_text(1, 1), None);
    }

    #[test]
    fn test_text_round_trip_via_copy() {
        let mut test_sheet = Sheet::new(5, 5);

        test_sheet.update_cell_data(1, 1, String::from("Some label"));
        let _ = test_sheet.copy_cell(1, 1, 2, 2);
        assert_eq!(test_sheet.get_text(2, 2), Some("Some label".to_string()));
    }

    #[test]
    fn test_new_sheet_cells_are_empty() {
        let test_sheet = Sheet::new(5, 5);
        assert!(test_sheet.is_empty_cell(1, 1));
        assert_eq!(test_sheet.get_value(1, 1), 0);
    }

    #[test]
    fn test_cell_not_empty_after_set() {
        let mut test_sheet = Sheet::new(5, 5);
        test_sheet.update_cell_data(1, 1, String::from("42"));
        assert!(!test_sheet.is_empty_cell(1, 1));

        // Explicitly setting a cell to the number 0 is different from never
        // having touched it at all.
        test_sheet.update_cell_data(1, 2, String::from("0"));
        assert!(!test_sheet.is_empty_cell(1, 2));
    }

    #[test]
    fn test_clear_cell_makes_it_empty_again() {
        let mut test_sheet = Sheet::new(5, 5);
        test_sheet.update_cell_data(1, 1, String::from("42"));
        assert!(!test_sheet.is_empty_cell(1, 1));

        test_sheet.clear_cell(1, 1);
        assert!(test_sheet.is_empty_cell(1, 1));
        assert_eq!(test_sheet.get_value(1, 1), 0);
    }
}
