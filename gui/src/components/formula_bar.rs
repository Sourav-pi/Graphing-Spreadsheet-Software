//! Formula bar component for the spreadsheet application.
//!
//! This module provides a formula bar component that allows viewing and editing
//! formulas for the currently selected cell.

use super::spreadsheet::*;
use dioxus::prelude::*;

/// Converts a column index to alphabetic representation (e.g. 0->A, 25->Z, 26->AA)
fn column_to_letter(column: usize) -> String {
    let mut result = String::new();
    let mut temp = column - 1;

    loop {
        let remainder = temp % 26;
        result.insert(0, (b'A' + remainder as u8) as char);
        temp /= 26;
        if temp == 0 {
            break;
        }
        temp -= 1;
    }

    result
}

/// The Formula Bar component for editing cell formulas
///
/// This component displays and allows editing of the formula in the currently
/// selected cell. It syncs with the spreadsheet state and updates when the
/// selected cell changes.
#[component]
pub fn FormulaBar() -> Element {
    // Context signals used by the formula bar
    let sheet = use_context::<SheetContext>();
    let mut sheetversion = use_context::<SheetVersionContext>();
    let selected_cell = use_context::<SelectedCellContext>();
    let mut formula = use_signal(String::new);

    // Effect to update formula when selected cell changes
    use_effect(move || {
        let _ = sheetversion.cloned();

        if let Ok(sheet_locked) = sheet.cloned().lock() {
            // Update the cell value in the Sheet object
            let formula_in_sheet = sheet_locked
                .get_formula(
                    selected_cell.cloned().0 as usize,
                    selected_cell.cloned().1 as usize,
                )
                .to_string();
            if formula_in_sheet != "0" {
                formula.set(formula_in_sheet);
            } else {
                formula.set("".to_string());
            }
        }
    });

    // Handle formula submission when Enter key is pressed
    let on_submit = move |e: Event<KeyboardData>| {
        if e.key() == Key::Enter {
            if let Ok(mut sheet_locked) = sheet.cloned().lock() {
                // Update the cell value in the Sheet object
                sheet_locked.update_cell_data(
                    selected_cell.cloned().0 as usize,
                    selected_cell.cloned().1 as usize,
                    formula.cloned(),
                );
                sheetversion.set(sheetversion.cloned() + 1);
            }
        }
    };

    // Handle formula update when input loses focus
    let on_blur = move |_| {
        if let Ok(mut sheet_locked) = sheet.cloned().lock() {
            // Update the cell value in the Sheet object
            sheet_locked.update_cell_data(
                selected_cell.cloned().0 as usize,
                selected_cell.cloned().1 as usize,
                formula.cloned(),
            );
            sheetversion.set(sheetversion.cloned() + 1);
        }
    };

    rsx! {
        div {
            class: "formula-bar",
            input {
                class: "cell-ref-chip",
                readonly: true,
                value: "{column_to_letter(selected_cell.cloned().1 as usize)}{selected_cell.cloned().0}"
            }
            input {
                class: "formula-input",
                value: "{formula}",
                oninput: move |evt| formula.set(evt.value().clone()),
                onblur: on_blur,
                onkeydown: on_submit
            }
        }
    }
}
