//! Provides functionality for generating various chart types from spreadsheet data.
//!
//! This module extends the Sheet struct with methods to create:
//! - Line graphs
//! - Bar graphs
//! - Pie charts
//! - Scatter plots
//!
//! Each graph is generated as a JSON string compatible with visualization libraries.

use crate::parse::convert_to_index;
use crate::sheet::Sheet;
use charming::{
    Chart,
    component::{Axis, Legend, Title},
    element::{AxisType, Emphasis, ItemStyle, Orient, Symbol, Tooltip, Trigger},
    series::{Bar, Line, pie},
};
use std::{cmp::max, vec};

/// Represents a cell range with start and end coordinates.
///
/// Format is ((start_row, start_col), (end_row, end_col))
type Range = ((usize, usize), (usize, usize));

/// Parses a range expression (e.g., "A1:B5") into start and end cell coordinates.
///
/// The range must be either a single row or a single column.
///
/// # Parameters
/// * `range` - A string slice containing the range expression (e.g., "A1:B5")
///
/// # Returns
/// * `Ok(Range)` - Successfully parsed range with start and end coordinates
/// * `Err(String)` - Error message if the range is invalid
pub fn parse_range(range: &str, row: usize, col: usize) -> Result<Range, String> {
    let parts: Vec<&str> = range.split(':').collect();
    if parts.len() != 2 {
        return Err("Invalid range format".to_string());
    }
    let start = parts[0].trim();
    let end = parts[1].trim();
    if start.is_empty() || end.is_empty() {
        return Err("Range cannot be empty".to_string());
    }
    let start_tuple = convert_to_index(start.to_string());
    let end_tuple = convert_to_index(end.to_string());
    if start_tuple == (0, 0) || end_tuple == (0, 0) {
        return Err("Invalid cell reference".to_string());
    }

    if start_tuple.0 > row || start_tuple.1 > col || end_tuple.0 > row || end_tuple.1 > col {
        return Err("Cell reference out of bounds".to_string());
    }

    if start_tuple.0 == end_tuple.0 && start_tuple.1 <= end_tuple.1 {
        Ok((start_tuple, end_tuple))
    } else if start_tuple.0 <= end_tuple.0 && start_tuple.1 == end_tuple.1 {
        return Ok((start_tuple, end_tuple));
    } else {
        return Err("Invalid range".to_string());
    }
}

/// Parses a comma-separated list of labels into a vector of strings.
///
/// # Parameters
/// * `labels` - A string slice containing comma-separated labels
///
/// # Returns
/// A vector of trimmed label strings
fn parse_lables(labels: &str) -> Vec<String> {
    if labels.is_empty() {
        return Vec::new();
    }
    labels.split(',').map(|s| s.trim().to_string()).collect()
}

impl Sheet {
    /// Creates a line graph from the specified range of cells.
    ///
    /// # Parameters
    /// * `range` - The cell range to use as data points (e.g., "A1:A10")
    /// * `x_labels` - Comma-separated labels for the X-axis (empty for automatic numbering)
    /// * `y_lable` - Label for the Y-axis
    /// * `title` - Title of the graph
    ///
    /// # Returns
    /// * `Ok(String)` - JSON string representation of the chart
    /// * `Err(String)` - Error message if the range is invalid
    pub fn line_graph(
        &self,
        range: &str,
        x_labels: &str,
        y_lable: &str,
        title: &str,
    ) -> Result<String, String> {
        let (start, end) = parse_range(range, self.row, self.col)?;
        let label_x: Vec<String> = if x_labels.is_empty() {
            let temp_range = max(end.0 - start.0 + 1, end.1 - start.1 + 1);
            let mut temp_vec: Vec<String> = Vec::new();
            for i in 1..(temp_range + 1) {
                temp_vec.push(format!("{}", i));
            }
            temp_vec
        } else {
            parse_lables(x_labels)
        };
        let mut values = Vec::new();
        for i in start.0..=end.0 {
            for j in start.1..=end.1 {
                values.push(self.grid[i][j].value);
            }
        }
        Ok(Chart::new()
            .x_axis(Axis::new().type_(AxisType::Category).data(label_x))
            .title(Title::new().text(title))
            .y_axis(Axis::new().name(y_lable).type_(AxisType::Value))
            .series(Line::new().data(values))
            .to_string())
    }

    /// Creates a bar graph from the specified range of cells.
    ///
    /// # Parameters
    /// * `range` - The cell range to use as data points (e.g., "A1:A10")
    /// * `x_labels` - Comma-separated labels for the X-axis (empty for automatic numbering)
    /// * `y_lable` - Label for the Y-axis
    /// * `title` - Title of the graph
    ///
    /// # Returns
    /// * `Ok(String)` - JSON string representation of the chart
    /// * `Err(String)` - Error message if the range is invalid
    pub fn bar_graph(
        &self,
        range: &str,
        x_labels: &str,
        y_lable: &str,
        title: &str,
    ) -> Result<String, String> {
        let (start, end) = parse_range(range, self.row, self.col)?;
        let label_x = if x_labels.is_empty() {
            let temp_range = max(end.0 - start.0 + 1, end.1 - start.1 + 1);
            let mut temp_vec: Vec<String> = Vec::new();
            for i in 1..(temp_range + 1) {
                temp_vec.push(format!("{}", i));
            }
            temp_vec
        } else {
            parse_lables(x_labels)
        };
        let mut values = Vec::new();
        for i in start.0..=end.0 {
            for j in start.1..=end.1 {
                values.push(self.grid[i][j].value);
            }
        }
        Ok(Chart::new()
            .x_axis(Axis::new().type_(AxisType::Category).data(label_x))
            .title(Title::new().text(title))
            .y_axis(Axis::new().name(y_lable).type_(AxisType::Value))
            .series(Bar::new().data(values))
            .to_string())
    }

    /// Creates a pie chart from the specified range of cells.
    ///
    /// # Parameters
    /// * `range` - The cell range to use as data points (e.g., "A1:A10")
    /// * `x_labels` - Comma-separated labels for the pie slices (empty for automatic numbering)
    /// * `title` - Title of the chart
    ///
    /// # Returns
    /// * `Ok(String)` - JSON string representation of the chart
    /// * `Err(String)` - Error message if the range is invalid
    pub fn pie_graph(&self, range: &str, x_labels: &str, title: &str) -> Result<String, String> {
        let (start, end) = parse_range(range, self.row, self.col)?;
        let mut x_labels = parse_lables(x_labels);
        let mut values: Vec<i32> = Vec::new();
        let mut cnt = 0;

        if start.0 == end.0 {
            for i in start.1..=end.1 {
                values.push(self.grid[start.0][i].value);
                if x_labels.len() <= cnt {
                    x_labels.push(format!("{}", cnt + 1));
                }
                cnt += 1;
            }
        } else if start.1 == end.1 {
            for i in start.0..=end.0 {
                values.push(self.grid[i][start.1].value);
                if x_labels.len() <= cnt {
                    x_labels.push(format!("{}", cnt + 1));
                }
                cnt += 1;
            }
        } else {
            return Err("Invalid range".to_string());
        }
        let data: Vec<(i32, String)> = values.into_iter().zip(x_labels).collect();
        Ok(Chart::new()
            .tooltip(Tooltip::new().trigger(Trigger::Item))
            .legend(Legend::new().orient(Orient::Vertical).right("right"))
            .series(
                pie::Pie::new()
                    .radius(100)
                    .center(vec!["50%", "50%"])
                    .data(data)
                    .emphasis(
                        Emphasis::new().item_style(
                            ItemStyle::new()
                                .shadow_color("rgba(0, 0, 0, 0.5)")
                                .shadow_offset_x(0)
                                .shadow_offset_y(5)
                                .shadow_blur(10),
                        ),
                    ),
            )
            .title(Title::new().text(title))
            .to_string())
    }

    /// Creates a scatter plot from the specified ranges of cells.
    ///
    /// # Parameters
    /// * `rangex` - The cell range to use as X-axis data points (e.g., "A1:A10")
    /// * `rangey` - The cell range to use as Y-axis data points (e.g., "B1:B10")
    /// * `title` - Title of the chart
    /// * `x_name` - Label for the X-axis
    /// * `y_name` - Label for the Y-axis
    ///
    /// # Returns
    /// * `Ok(String)` - JSON string representation of the chart
    /// * `Err(String)` - Error message if the ranges are invalid
    pub fn scatter_graph(
        &self,
        rangex: &str,
        rangey: &str,
        title: &str,
        x_name: &str,
        y_name: &str,
    ) -> Result<String, String> {
        let (start1, end1) = parse_range(rangex, self.row, self.col)?;
        let (start2, end2) = parse_range(rangey, self.row, self.col)?;

        let diff1 = (end1.0 - start1.0) as i32;
        let diff2 = (end1.1 - start1.1) as i32;
        let diff3 = (end2.0 - start2.0) as i32;
        let diff4 = (end2.1 - start2.1) as i32;
        if diff1 != diff3 || diff2 != diff4 {
            return Err("Invalid range".to_string());
        }
        let mut values: Vec<Vec<i32>> = Vec::new();
        for i in 0..diff1 + 1 {
            for j in 0..diff2 + 1 {
                let temp_vec: Vec<i32> = vec![
                    self.grid[start1.0 + i as usize][start1.1 + j as usize].value,
                    self.grid[start2.0 + i as usize][start2.1 + j as usize].value,
                ];
                values.push(temp_vec);
            }
        }
        Ok(Chart::new()
            .title(Title::new().text(title))
            .x_axis(Axis::new().type_(AxisType::Value).name(x_name))
            .y_axis(Axis::new().type_(AxisType::Value).name(y_name))
            .series(
                Line::new()
                    .data(values)
                    .symbol(Symbol::Circle)
                    .symbol_size(10)
                    .item_style(ItemStyle::new().color("blue")),
            )
            .to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_graph() {
        let mut test_sheet = Sheet::new(6, 6);
        test_sheet.update_cell_data(1, 1, String::from("=A2+A3"));
        test_sheet.update_cell_data(2, 1, String::from("90"));
        test_sheet.update_cell_data(3, 1, String::from("50"));
        test_sheet.update_cell_data(4, 1, String::from("=A1+A2"));
        test_sheet.update_cell_data(5, 1, String::from("-5"));
        test_sheet.update_cell_data(6, 1, String::from("6"));
        let result = test_sheet.line_graph("A1:A6", "A2,A3,A4,A5,A6", "Y Axis", "Line Graph");
        assert!(result.is_ok());
    }

    #[test]
    fn test_scatter_graph() {
        let mut test_sheet = Sheet::new(6, 6);
        test_sheet.update_cell_data(1, 1, String::from("=A2+A3"));
        test_sheet.update_cell_data(2, 1, String::from("90"));
        test_sheet.update_cell_data(3, 1, String::from("50"));
        test_sheet.update_cell_data(4, 1, String::from("=A1+A2"));
        test_sheet.update_cell_data(5, 1, String::from("-5"));
        test_sheet.update_cell_data(6, 1, String::from("6"));
        let result =
            test_sheet.scatter_graph("A1:A6", "B1:B6", "Scatter Graph", "X Axis", "Y Axis");
        assert!(result.is_ok());
    }

    #[test]
    fn test_bar_graph() {
        let mut test_sheet = Sheet::new(6, 6);
        test_sheet.update_cell_data(1, 1, String::from("=A2+A3"));
        test_sheet.update_cell_data(2, 1, String::from("90"));
        test_sheet.update_cell_data(3, 1, String::from("50"));
        test_sheet.update_cell_data(4, 1, String::from("=A1+A2"));
        test_sheet.update_cell_data(5, 1, String::from("-5"));
        test_sheet.update_cell_data(6, 1, String::from("6"));
        let result = test_sheet.bar_graph("A1:A6", "A2,A3,A4,A5,A6", "Y Axis", "Bar Graph");
        assert!(result.is_ok());
    }

    #[test]
    fn test_pie_graph() {
        let mut test_sheet = Sheet::new(6, 6);
        test_sheet.update_cell_data(1, 1, String::from("=A2+A3"));
        test_sheet.update_cell_data(2, 1, String::from("90"));
        test_sheet.update_cell_data(3, 1, String::from("50"));
        test_sheet.update_cell_data(4, 1, String::from("=A1+A2"));
        test_sheet.update_cell_data(5, 1, String::from("-5"));
        test_sheet.update_cell_data(6, 1, String::from("6"));
        let result = test_sheet.pie_graph("A1:A6", "A2,A3,A4,A5,A6", "Pie Graph");
        assert!(result.is_ok());
    }
    #[test]
    fn test_parse_range_valid_row() {
        let result = parse_range("A1:A5", 100, 100);
        assert!(result.is_ok());
        let ((start_row, start_col), (end_row, end_col)) = result.unwrap();
        assert_eq!(start_row, 1);
        assert_eq!(start_col, 1);
        assert_eq!(end_row, 5);
        assert_eq!(end_col, 1);
    }

    #[test]
    fn test_parse_range_valid_column() {
        let result = parse_range("A1:C1", 100, 100);
        assert!(result.is_ok());
        let ((start_row, start_col), (end_row, end_col)) = result.unwrap();
        assert_eq!(start_row, 1);
        assert_eq!(start_col, 1);
        assert_eq!(end_row, 1);
        assert_eq!(end_col, 3);
    }

    #[test]
    fn test_parse_range_invalid_format() {
        let result = parse_range("A1B5", 100, 100);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid range format");
    }

    #[test]
    fn test_parse_range_empty_parts() {
        let result = parse_range("A1:", 100, 100);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Range cannot be empty");
    }

    #[test]
    fn test_parse_range_invalid_cell() {
        let result = parse_range("XYZ:A5", 100, 100);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid cell reference");
    }

    #[test]
    fn test_parse_range_invalid_direction() {
        let result = parse_range("C5:A1", 100, 100);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid range");
    }

    // Tests for parse_lables function
    #[test]
    fn test_parse_labels_empty() {
        let result = parse_lables("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_labels_single() {
        let result = parse_lables("Label1");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "Label1");
    }

    #[test]
    fn test_parse_labels_multiple() {
        let result = parse_lables("Label1,Label2,Label3");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "Label1");
        assert_eq!(result[1], "Label2");
        assert_eq!(result[2], "Label3");
    }

    #[test]
    fn test_parse_labels_with_spaces() {
        let result = parse_lables(" Label1 , Label2 , Label3 ");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], "Label1");
        assert_eq!(result[1], "Label2");
        assert_eq!(result[2], "Label3");
    }

    // Edge case tests for graph functions
    #[test]
    fn test_line_graph_empty_labels() {
        let mut test_sheet = Sheet::new(3, 3);
        test_sheet.update_cell_data(1, 1, String::from("10"));
        test_sheet.update_cell_data(1, 2, String::from("20"));
        test_sheet.update_cell_data(1, 3, String::from("30"));
        let result = test_sheet.line_graph("A1:C1", "", "Y Axis", "Line Graph");
        assert!(result.is_ok());
    }

    #[test]
    fn test_bar_graph_invalid_range() {
        let test_sheet = Sheet::new(3, 3);
        let result = test_sheet.bar_graph("A5:C10", "", "Y Axis", "Bar Graph");
        assert!(result.is_err());
    }

    #[test]
    fn test_pie_graph_invalid_non_linear_range() {
        let test_sheet = Sheet::new(3, 3);
        let result = test_sheet.pie_graph("A1:C3", "", "Pie Graph");
        println!("{:?}", result);
        assert!(result.is_err());
    }

    #[test]
    fn test_scatter_graph_mismatched_ranges() {
        let test_sheet = Sheet::new(3, 3);
        let result = test_sheet.scatter_graph("A1:A3", "B1:B5", "Scatter", "X", "Y");
        println!("{:?}", result);
        assert!(result.is_err());
    }
}
