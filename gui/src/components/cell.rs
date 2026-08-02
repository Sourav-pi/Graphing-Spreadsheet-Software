use super::error_display::{show_error, ErrorContext, ErrorType};
use super::spreadsheet::*;
use cores::Error;
use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct CellProps {
    pub row: i32,
    pub col: i32,
    #[props(default = false)]
    pub is_header: bool,
    #[props(default = false)]
    pub is_selected: bool,
    #[props(default = 80)]
    pub min_width: i32,
    #[props(into)]
    pub oncontextmenu: Callback<dioxus::prelude::Event<dioxus::events::MouseData>>,
}

#[component]
pub fn Cell(props: CellProps) -> Element {
    // Consume the contexts
    let mut selected_cell = use_context::<SelectedCellContext>();
    let sheet = use_context::<SheetContext>();
    let mut sheetversion = use_context::<SheetVersionContext>();
    let mut error_ctx = use_context::<ErrorContext>();

    let mut is_editing = use_signal(|| false);
    let mut formula = use_signal(String::new);
    let mut value = use_signal(String::new);

    // Handler for when user starts editing
    let on_focus = {
        let row = props.row;
        let col = props.col;
        move |_| {
            is_editing.set(true);
            selected_cell.set((row, col));
            println!("Selected cell ({}, {}) for editing", row, col);
        }
    };

    // Handler for when the cell loses focus
    let on_blur = {
        let row = props.row;
        let col = props.col;
        move |_| {
            is_editing.set(false);

            // Get the formula from context in case it was updated elsewhere
            let formula_text = formula.read().clone();
            if let Ok(mut sheet_locked) = sheet.cloned().lock() {
                if formula_text.is_empty() {
                    // Leaving the cell blank clears it rather than writing "0".
                    sheet_locked.clear_cell(row as usize, col as usize);
                    sheetversion.set(sheetversion.cloned() + 1);
                    return;
                }
                // Update the cell value in the Sheet object
                let res =
                    sheet_locked.update_cell_data(row as usize, col as usize, formula_text.clone());
                match res.error {
                    Error::None | Error::DivByZero => {
                        sheetversion.set(sheetversion.cloned() + 1);
                    }
                    Error::InvalidInput => {
                        show_error(
                            &mut error_ctx,
                            "Invalid Formula",
                            ErrorType::Error,
                            Some(3.0),
                        );
                    }
                    Error::CycleDetected => {
                        show_error(
                            &mut error_ctx,
                            "Cannot apply formula : would create circular reference",
                            ErrorType::Error,
                            Some(3.0),
                        );
                    }
                }
                // Update the displayed value
                println!("Updated cell ({}, {}) to: {}", row, col, formula_text);
            }
        }
    };

    // Handler for input changes
    let on_input = move |e: Event<FormData>| {
        let new_value = e.value().clone();
        formula.set(new_value);
    };

    // Virtualized rows/columns reuse the same Cell component instance for a
    // new (row, col) as the user scrolls, without remounting it. Since props
    // aren't reactive on their own, these effects need `use_reactive` so they
    // re-run when `props.row`/`props.col` change and not just when the sheet
    // itself changes (`sheetversion`) — otherwise the cell keeps showing
    // whatever cell it last displayed until it happens to regain focus.
    let row = props.row;
    let col = props.col;

    // Reset edit state when this reused instance is reassigned to a different cell.
    use_effect(use_reactive!(|(row, col)| {
        is_editing.set(false);
        let _ = document::eval(&format!(
            "let x = document.getElementById('row-{}-col-{}'); if (x) {{ x.blur(); }}",
            row, col
        ));
    }));

    use_effect(use_reactive!(|(row, col)| {
        let _ = sheetversion.cloned();
        if let Ok(sheet_locked) = sheet.cloned().lock() {
            // A never-set (or since-cleared) cell edits as blank rather than "0".
            let display = if sheet_locked.is_empty_cell(row, col) {
                String::new()
            } else {
                sheet_locked
                    .get_formula(row as usize, col as usize)
                    .to_string()
            };
            formula.set(display);
        }
    }));

    use_effect(use_reactive!(|(row, col)| {
        let _ = sheetversion.cloned();

        if let Ok(sheet_locked) = sheet.cloned().lock() {
            // A never-set (or since-cleared) cell displays as blank rather
            // than "0". Text cells show their literal text; everything else
            // shows its computed numeric value.
            let display = if sheet_locked.is_empty_cell(row, col) {
                String::new()
            } else {
                sheet_locked
                    .get_text(row, col)
                    .unwrap_or_else(|| sheet_locked.get_value(row, col).to_string())
            };
            value.set(display);
        }
    }));

    let width_style = format!("width: {}px;", props.min_width);

    if props.is_header {
        rsx! {
            input {
                id: "row-{props.row}-col-{props.col}",
                readonly: true,
                value: props.col.to_string(),
                style: "{width_style}",
                class: "cell-header",
            }
        }
    } else {
        // Blue border only on the cell that actually has focus, not just
        // whichever cell was last selected.
        let cell_class = if is_editing.cloned() {
            "cell-input selected"
        } else {
            "cell-input"
        };
        rsx! {
            input {
                id: "row-{props.row}-col-{props.col}",
                onfocus: on_focus,
                onblur: on_blur,
                oninput: on_input,
                oncontextmenu: props.oncontextmenu,
                // Show the formula when editing, otherwise show the result
                value: if *is_editing.read() { formula.cloned() } else { value.cloned() },
                style: "{width_style}",
                class: "{cell_class}",
            }
        }
    }
}
