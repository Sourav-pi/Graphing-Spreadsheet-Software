//! Context menu component for the spreadsheet application.
//!
//! This module provides context menus that appear when right-clicking on
//! cells, rows, or columns. It supports operations like copy, paste, and
//! other context-specific actions.

use super::error_display::{show_error, ErrorContext, ErrorType};
use super::spreadsheet::*;
use cores::Error;
use dioxus::prelude::*;

/// Defines the type of context menu to display
#[derive(Clone, Copy, PartialEq)]
pub enum MenuType {
    /// Row context menu (appears when right-clicking a row header)
    Row,
    /// Column context menu (appears when right-clicking a column header)
    Col,
    /// Cell context menu (appears when right-clicking a cell)
    Cell,
}

/// The context menu component that provides copy/paste functionality for cells, rows, and columns.
///
/// This component displays different menu items based on the menu type (row, column, or cell).
/// It supports operations like copying and pasting cells, rows, or columns.
///
/// The menu appears at the coordinates specified by the context menu signal and disappears
/// when an action is performed or the user clicks elsewhere.
#[component]
#[allow(unused_braces)]
pub fn ContextMenu() -> Element {
    let mut context_menu = use_context::<ContextMenuContext>();
    let sheet = use_context::<SheetContext>();
    let mut sheet_version = use_context::<SheetVersionContext>();
    let mut error_ctx = use_context::<ErrorContext>();

    // Get clipboard contexts
    let mut copied_cell = use_context::<CopiedCellContext>();
    let mut copied_row = use_context::<CopiedRowContext>();
    let mut copied_col = use_context::<CopiedColContext>();

    if let Some((x_cord, y_cord, row, col, menu_type)) = context_menu.cloned() {
        match menu_type {
            MenuType::Row => {
                // Check if we have a copied row
                let has_copied_row = copied_row.read().is_some();

                rsx! {
                    div {
                        class: "context-menu",
                        style: "position: fixed; left: {x_cord}px; top: {y_cord}px;",
                        div {
                            class: "context-menu-item",
                            onclick: move |_| {
                                copied_row.set(Some(row));
                                context_menu.set(None);
                            },
                            "Copy Row"
                        }
                        div {
                            class: if has_copied_row { "context-menu-item" } else { "context-menu-item disabled" },
                            onclick: move |_| {
                                if let Some(source_row) = *copied_row.read() {
                                    if let Ok(mut sheet_locked) = sheet.cloned().lock() {
                                        // Use the Sheet's copy_row method with error handling
                                        match sheet_locked.copy_row(source_row as usize, row as usize) {
                                            Ok(_) => {
                                                sheet_version.set(sheet_version.cloned() + 1);
                                            },
                                            Err(Error::CycleDetected) => {
                                                sheet_version.set(sheet_version.cloned() + 1);
                                                show_error(&mut error_ctx, "Cannot paste: would create circular reference",
                                                          ErrorType::Error, Some(3.0));

                                            },
                                            Err(_) => {
                                                sheet_version.set(sheet_version.cloned() + 1);
                                                show_error(&mut error_ctx, "Error pasting row", ErrorType::Error, Some(3.0));
                                            }
                                        }
                                    }
                                }
                                context_menu.set(None);
                            },
                            "Paste Row"
                        }
                        div {
                            class: "context-menu-item",
                            onclick: move |_| {
                                if let Ok(mut sheet_locked) = sheet.cloned().lock() {
                                    sheet_locked.clear_row(row as usize);
                                    sheet_version.set(sheet_version.cloned() + 1);
                                    }
                                context_menu.set(None);

                            },
                            "Clear Row"
                        }
                    }
                }
            }
            MenuType::Col => {
                // Check if we have a copied column
                let has_copied_col = copied_col.read().is_some();

                rsx! {
                    div {
                        class: "context-menu",
                        style: "position: fixed; left: {x_cord}px; top: {y_cord}px;",
                        div {
                            class: "context-menu-item",
                            onclick: move |_| {
                                // Store which column was copied
                                copied_col.set(Some(col));
                                context_menu.set(None);
                            },
                            "Copy Column"
                        }
                        div {
                            class: if has_copied_col { "context-menu-item" } else { "context-menu-item disabled" },
                            onclick: move |_| {
                                if let Some(source_col) = *copied_col.read() {
                                    if let Ok(mut sheet_locked) = sheet.cloned().lock() {
                                        // Use the Sheet's copy_col method with error handling
                                        match sheet_locked.copy_col(source_col as usize, col as usize) {
                                            Ok(_) => {
                                                // Update sheet version to trigger rerender
                                                sheet_version.set(sheet_version.cloned() + 1);
                                            },
                                            Err(Error::CycleDetected) => {
                                                sheet_version.set(sheet_version.cloned() + 1);
                                                show_error(&mut error_ctx, "Cannot paste: would create circular reference",
                                                          ErrorType::Error, Some(3.0));
                                            },
                                            Err(_) => {
                                                sheet_version.set(sheet_version.cloned() + 1);
                                                show_error(&mut error_ctx, "Error pasting column", ErrorType::Error, Some(3.0));
                                            }
                                        }
                                    }
                                }
                                context_menu.set(None);
                            },
                            "Paste Column"
                        }
                        div {
                            class: "context-menu-item",
                            onclick: move |_| {
                                if let Ok(mut sheet_locked) = sheet.cloned().lock() {
                                    sheet_locked.clear_col(col as usize);
                                    sheet_version.set(sheet_version.cloned() + 1);
                                    }
                                context_menu.set(None);
                            },
                            "Clear Column"
                        }
                    }
                }
            }
            MenuType::Cell => {
                // Check if we have a copied cell
                let has_copied_cell = copied_cell.read().is_some();

                rsx! {
                    div {
                        class: "context-menu",
                        style: "position: fixed; left: {x_cord}px; top: {y_cord}px;",
                        div {
                            class: "context-menu-item",
                            onclick: move |_| {
                                // Store which cell was copied
                                copied_cell.set(Some((row, col)));
                                context_menu.set(None);
                            },
                            "Copy Cell"
                        }
                        div {
                            class: if has_copied_cell { "context-menu-item" } else { "context-menu-item disabled" },
                            onclick: move |_| {
                                if let Some((source_row, source_col)) = *copied_cell.read() {
                                    if let Ok(mut sheet_locked) = sheet.cloned().lock() {
                                        // Use the Sheet's copy_cell method with error handling
                                        match sheet_locked.copy_cell(
                                            source_row as usize,
                                            source_col as usize,
                                            row as usize,
                                            col as usize
                                        ) {
                                            Ok(_) => {
                                                sheet_version.set(sheet_version.cloned() + 1);
                                            },
                                            Err(Error::CycleDetected) => {
                                                sheet_version.set(sheet_version.cloned() + 1);
                                                show_error(&mut error_ctx, "Cannot paste: would create circular reference",
                                                          ErrorType::Error, Some(3.0));
                                            },
                                            Err(_) => {
                                                sheet_version.set(sheet_version.cloned() + 1);
                                                show_error(&mut error_ctx, "Error pasting cell", ErrorType::Error, Some(3.0));
                                            }
                                        }
                                    }
                                }
                                context_menu.set(None);
                            },
                            "Paste Cell"
                        }

                        div {
                            class: "context-menu-item",
                            onclick: move |_| {
                                if let Ok(mut sheet_locked) = sheet.cloned().lock() {
                                    sheet_locked.clear_cell(row as usize, col as usize);
                                    sheet_version.set(sheet_version.cloned() + 1);
                                    }
                                context_menu.set(None);

                            },
                            "Clear Cell"
                        }
                    }
                }
            }
        }
    } else {
        rsx! { div {} }
    }
}
