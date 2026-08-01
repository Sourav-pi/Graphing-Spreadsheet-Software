//! Header component for the spreadsheet application.
//!
//! This module provides the header component that displays the application
//! title, current filename, and contains the toolbar and formula bar.

use super::formula_bar::*;
use super::toolbar::*;
use dioxus::prelude::*;

/// Icon for the spreadsheet application
const SPREADSHEET_ICON: Asset = asset!("assets/spreadsheet.png");

/// Properties for the Header component
#[derive(Props, PartialEq, Clone)]
pub struct HeaderProps {
    /// The current filename being edited
    pub filename: String,
    /// Number of rows in the spreadsheet
    pub num_rows: usize,
    /// Number of columns in the spreadsheet
    pub num_cols: usize,
}

/// The Header component for the spreadsheet application.
///
/// Displays the application title, current filename, toolbar, and formula bar.
///
/// # Properties
/// * `filename` - The name of the current file being edited
/// * `num_rows` - Total number of rows in the spreadsheet
/// * `num_cols` - Total number of columns in the spreadsheet
#[component]
pub fn Header(props: HeaderProps) -> Element {
    rsx! {
      div { class: "app-header",
        div { class: "app-header-bar",
          div { class: "app-header-left",
              img {
                  src: "{SPREADSHEET_ICON}",
                  alt: "Logo",
                  class: "app-logo"
              }
              h1 { class: "app-title", "{props.filename}" }
          }
          Toolbar {
              num_rows: props.num_rows,
              num_cols: props.num_cols,
          }
        }
        FormulaBar {}
      }
    }
}
