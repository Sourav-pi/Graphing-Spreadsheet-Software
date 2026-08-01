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

    // Check if this cell is selected based on context
    let is_this_cell_selected = {
        let (sel_row, sel_col) = *selected_cell.read();
        props.row == sel_row && props.col == sel_col
    };

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
            let mut formula_text = formula.read().clone();
            // Evaluate the formula and update the displayed value
            if formula_text.is_empty() {
                formula_text = "0".to_string();
            }
            if let Ok(mut sheet_locked) = sheet.cloned().lock() {
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

    use_effect(use_reactive!(|(row, col)| {
        let _ = sheetversion.cloned();
        if let Ok(sheet_locked) = sheet.cloned().lock() {
            // Update the cell value in the Sheet object
            formula.set(
                sheet_locked
                    .get_formula(row as usize, col as usize)
                    .to_string(),
            );
        }
    }));

    use_effect(use_reactive!(|(row, col)| {
        let _ = sheetversion.cloned();

        if let Ok(sheet_locked) = sheet.cloned().lock() {
            // Update the cell value in the Sheet object
            value.set(sheet_locked.get_value(row, col).to_string());
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
        let cell_class = if is_editing.cloned() || is_this_cell_selected {
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
                value: if *is_editing.read() {
                    formula.cloned()
                } else {
                    value.cloned()
                },
                style: "{width_style}",
                class: "{cell_class}",
            }
        }
    }
}
