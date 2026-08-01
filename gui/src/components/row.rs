use super::cell::Cell;
use super::context_menu::MenuType;
use super::spreadsheet::ContextMenuContext;
use dioxus::prelude::*;

// Helper function to convert column number to letter
pub fn column_to_letter(col: i32) -> String {
    if col <= 0 {
        return String::new();
    }

    let mut result = String::new();
    let mut col_num = col;

    while col_num > 0 {
        let remainder = (col_num - 1) % 26;
        result.insert(0, (b'A' + remainder as u8) as char);
        col_num = (col_num - 1) / 26;
    }

    result
}

#[derive(Props, PartialEq, Clone)]
pub struct RowProps {
    pub row: i32,
    pub num_cols: i32,
    #[props(default = false)]
    pub is_header: bool,
    #[props(default = 80)]
    pub min_width: i32,
    #[props(default = 0)]
    pub start_col: i32,
    #[props(default = 0)]
    pub end_col: i32,
    /// Sub-column pixel scroll offset, applied to everything except the
    /// pinned first (row-number/corner) cell, so horizontal scroll can move
    /// continuously instead of jumping a whole column at a time.
    #[props(default = 0.0)]
    pub x_offset: f64,
}

#[component]
pub fn Row(props: RowProps) -> Element {
    let mut context_menu = use_context::<ContextMenuContext>();

    let row_width_style = format!("width: {}px;", props.min_width);

    // Handler for row header right-click
    let on_row_contextmenu = {
        let row = props.row;
        move |e: Event<MouseData>| {
            e.stop_propagation();
            e.prevent_default();
            // Get the mouse position
            let mouse_y = e.client_coordinates().y;
            let mouse_x = e.client_coordinates().x;

            // Set the context menu data with mouse position and row number
            context_menu.set(Some((mouse_x, mouse_y, row, 0, MenuType::Row)));

            println!("Right-clicked on row {}", row);
        }
    };

    let generate_on_col_contextmenu = |col: i32| {
        let col_num = col;

        move |e: Event<MouseData>| {
            e.stop_propagation();
            e.prevent_default();
            // Get the mouse position
            let mouse_y = e.client_coordinates().y;
            let mouse_x = e.client_coordinates().x;

            // Set the context menu data with mouse position and column number
            context_menu.set(Some((mouse_x, mouse_y, 0, col_num, MenuType::Col)));

            println!("Right-clicked on column {}", col_num);
        }
    };

    let generate_on_cell_contextmenu = |row: i32, col: i32| {
        move |e: Event<MouseData>| {
            e.stop_propagation();
            e.prevent_default();
            // Get the mouse position
            let mouse_y = e.client_coordinates().y;
            let mouse_x = e.client_coordinates().x;

            // Set the context menu data with mouse position and cell coordinates
            context_menu.set(Some((mouse_x, mouse_y, row, col, MenuType::Cell)));

            println!("Right-clicked on cell ({}, {})", row, col);
        }
    };

    // Close context menu when clicking elsewhere
    let on_click = move |_| {
        context_menu.set(None);
    };

    let track_style = format!("transform: translateX(-{}px);", props.x_offset);

    rsx! {

        div {
            style: "display: flex; flex-direction: row;",
            class: "spreadsheet-row",
            onclick: on_click,
            // Pinned first cell: column-letter corner or row-number header
            if props.is_header {
                input { style: "{row_width_style}", class: "cell-header", readonly: true, "" }
            } else {
                input {
                    style: "{row_width_style}",
                    class: "cell-header",
                    readonly: true,
                    value: props.row.to_string(),
                    oncontextmenu: on_row_contextmenu,
                }
            }

            // Everything else scrolls continuously behind a clipped viewport
            div {
                class: "row-scroll-clip",
                div {
                    class: "row-scroll-track",
                    style: "{track_style}",
                    if props.is_header {
                        for col in (props.start_col)..=props.end_col {
                            {
                                let col_letter = column_to_letter(col);
                                rsx! {
                                    input {
                                        style: "width: {props.min_width}px;",
                                        class: "cell-header",
                                        readonly: true,
                                        value: "{col_letter}",
                                        oncontextmenu: generate_on_col_contextmenu(col),
                                    }
                                }
                            }
                        }
                    } else {
                        for col in (props.start_col)..=props.end_col {
                            Cell {
                                row: props.row,
                                col,
                                is_header: props.is_header,
                                min_width: props.min_width,
                                oncontextmenu: generate_on_cell_contextmenu(props.row, col),
                            }
                        }
                    }
                }
            }
        }

    }
}
