use super::row::Row;
use crate::components::spreadsheet::*;
use dioxus::html::geometry::WheelDelta;
use dioxus::{desktop::use_window, prelude::*};

/// Rendered height of a row (header or data), in pixels. Must match the
/// `.cell-header` / `.cell-input` height in main.css.
const ROW_HEIGHT: f64 = 30.0;

/// Extra rows/columns rendered beyond what exactly fits the viewport, so
/// there's no blank gap at the trailing edge while a sub-row/sub-column
/// scroll offset is in effect.
const OVERSCAN: i32 = 2;

#[derive(Props, PartialEq, Clone)]
pub struct GridProps {
    pub num_rows: i32,
    pub num_cols: i32,
}

#[component]
pub fn Grid(props: GridProps) -> Element {
    // Continuous scroll position, in pixels, from the top-left of the sheet.
    let mut scroll_top = use_signal(|| 0.0f64);
    let mut scroll_left = use_signal(|| 0.0f64);
    let mut selected_cell = use_context::<SelectedCellContext>();

    let window_width = use_window().inner_size().width;
    let window_height = use_window().inner_size().height;

    // Calculate cell width based on window size
    let min_cell_width = if window_width > 1600 {
        100
    } else if window_width > 1200 {
        90
    } else {
        81
    };
    let col_width = min_cell_width as f64;

    // Viewport available to the scrolling body, excluding the fixed header
    // row and the header/formula-bar chrome above the grid (see .grid-container
    // in main.css, which reserves the same 94px for that chrome).
    let body_viewport_height = (window_height as f64 - 94.0 - ROW_HEIGHT).max(ROW_HEIGHT);
    let data_viewport_width = (window_width as f64 - min_cell_width as f64).max(col_width);

    let max_scroll_top =
        ((props.num_rows as f64) * ROW_HEIGHT - body_viewport_height).max(0.0);
    let max_scroll_left =
        ((props.num_cols as f64) * col_width - data_viewport_width).max(0.0);

    // Clamp in case the window shrank or the sheet got smaller since the
    // last scroll.
    let clamped_top = scroll_top.cloned().clamp(0.0, max_scroll_top);
    let clamped_left = scroll_left.cloned().clamp(0.0, max_scroll_left);
    if clamped_top != scroll_top.cloned() {
        scroll_top.set(clamped_top);
    }
    if clamped_left != scroll_left.cloned() {
        scroll_left.set(clamped_left);
    }

    // First visible row/col and the sub-row/sub-col pixel remainder used to
    // shift the rendered window so scrolling feels continuous rather than
    // snapping row-by-row.
    let start_row = (clamped_top / ROW_HEIGHT).floor() as i32 + 1;
    let offset_y = clamped_top - (start_row - 1) as f64 * ROW_HEIGHT;
    let visible_rows = (body_viewport_height / ROW_HEIGHT).ceil() as i32 + OVERSCAN;
    let end_row = (start_row + visible_rows - 1).min(props.num_rows);

    let start_col = (clamped_left / col_width).floor() as i32 + 1;
    let offset_x = clamped_left - (start_col - 1) as f64 * col_width;
    let visible_cols = (data_viewport_width / col_width).ceil() as i32 + OVERSCAN;
    let end_col = (start_col + visible_cols - 1).min(props.num_cols);

    // Whether a cell is fully visible within the current scroll position
    // (not just within the overscan-padded render window).
    let is_cell_visible = move |row: i32, col: i32| -> bool {
        let row_top = (row - 1) as f64 * ROW_HEIGHT;
        let row_bottom = row_top + ROW_HEIGHT;
        let col_left = (col - 1) as f64 * col_width;
        let col_right = col_left + col_width;
        row_top >= clamped_top
            && row_bottom <= clamped_top + body_viewport_height
            && col_left >= clamped_left
            && col_right <= clamped_left + data_viewport_width
    };

    // Scroll just enough to bring a cell fully into view.
    let mut scroll_to_cell = move |row: i32, col: i32| {
        let row_top = (row - 1) as f64 * ROW_HEIGHT;
        let row_bottom = row_top + ROW_HEIGHT;
        if row_top < clamped_top {
            scroll_top.set(row_top.max(0.0));
        } else if row_bottom > clamped_top + body_viewport_height {
            scroll_top.set((row_bottom - body_viewport_height).min(max_scroll_top));
        }

        let col_left = (col - 1) as f64 * col_width;
        let col_right = col_left + col_width;
        if col_left < clamped_left {
            scroll_left.set(col_left.max(0.0));
        } else if col_right > clamped_left + data_viewport_width {
            scroll_left.set((col_right - data_viewport_width).min(max_scroll_left));
        }
    };

    // Effect to handle auto-scrolling when selected cell changes
    use_effect(move || {
        let (sel_row, sel_col) = *selected_cell.read();

        // If the currently selected cell is not visible, scroll to it
        if !is_cell_visible(sel_row, sel_col) {
            scroll_to_cell(sel_row, sel_col);
        }
    });

    // Natural, continuous scrolling: the wheel delta is added directly to
    // the pixel scroll position, same as a browser's native scroll.
    let on_wheel = move |e: Event<WheelData>| {
        e.prevent_default();

        let (raw_x, raw_y) = match e.delta() {
            WheelDelta::Pixels(v) => (v.x, v.y),
            WheelDelta::Lines(v) => (v.x * ROW_HEIGHT, v.y * ROW_HEIGHT),
            WheelDelta::Pages(v) => (
                v.x * data_viewport_width,
                v.y * body_viewport_height,
            ),
        };

        scroll_top.set((scroll_top.cloned() + raw_y).clamp(0.0, max_scroll_top));
        scroll_left.set((scroll_left.cloned() + raw_x).clamp(0.0, max_scroll_left));
    };

    // One-row/one-column nudges for the on-screen nav buttons.
    let move_up = move |_| {
        scroll_top.set((scroll_top.cloned() - ROW_HEIGHT).clamp(0.0, max_scroll_top));
    };
    let move_down = move |_| {
        scroll_top.set((scroll_top.cloned() + ROW_HEIGHT).clamp(0.0, max_scroll_top));
    };
    let move_left = move |_| {
        scroll_left.set((scroll_left.cloned() - col_width).clamp(0.0, max_scroll_left));
    };
    let move_right = move |_| {
        scroll_left.set((scroll_left.cloned() + col_width).clamp(0.0, max_scroll_left));
    };

    let on_keydown = {
        let row = selected_cell.cloned().0;
        let col = selected_cell.cloned().1;
        move |e: Event<KeyboardData>| {
            let key = e.key();
            if key == Key::Enter
                || key == Key::ArrowDown
                || key == Key::ArrowUp
                || key == Key::Tab
                || key == Key::ArrowLeft
                || key == Key::ArrowRight
            {
                e.prevent_default();
                let mut to_row = row;
                let mut to_col = col;

                if e.key() == Key::ArrowDown || e.key() == Key::Enter {
                    to_row = (row + 1).max(1).min(props.num_rows);
                } else if e.key() == Key::ArrowUp {
                    to_row = (row - 1).max(1).min(props.num_rows);
                } else if (e.key() == Key::Tab && e.modifiers().shift())
                    || e.key() == Key::ArrowLeft
                {
                    to_col = (col - 1).max(1).min(props.num_cols);
                } else if e.key() == Key::Tab || e.key() == Key::ArrowRight {
                    to_col = (col + 1).max(1).min(props.num_cols);
                }

                // Scroll synchronously so the target cell is already in the
                // DOM by the time the focus script below runs.
                scroll_to_cell(to_row, to_col);

                let script = format!(
                    r#"
                    let x = document.getElementById('row-{}-col-{}');
                    if (x) {{
                        x.focus();
                    }}
                "#,
                    to_row, to_col
                );
                selected_cell.set((to_row, to_col));
                document::eval(&script);
            }
        }
    };

    rsx! {
        div {
            class: "grid-container",
            onkeydown: on_keydown,
            onwheel: on_wheel,

            // Header row (fixed at the top, shifts horizontally with scroll_left)
            Row {
                row: 0,
                num_cols: end_col - start_col + 1,
                is_header: true,
                min_width: min_cell_width,
                start_col: start_col,
                end_col: end_col,
                x_offset: offset_x,
            }

            // Scrolling body
            div {
                class: "grid-body-clip",
                div {
                    class: "grid-body-track",
                    style: "transform: translateY(-{offset_y}px);",
                    for i in (start_row)..=(end_row) {
                        Row {
                            row: i,
                            num_cols: end_col - start_col + 1,
                            is_header: false,
                            min_width: min_cell_width,
                            start_col: start_col,
                            end_col: end_col,
                            x_offset: offset_x,
                        }
                    }
                }
            }

            // Navigation controls
            div {
                class: "nav-controls",

                // Page info display
                div {
                    class: "page-info",
                    "Rows: {start_row}-{end_row} / {props.num_rows -1 }, Cols: {start_col}-{end_col} / {props.num_cols -1}"
                }

                // Navigation buttons
                button {
                    class: "nav-btn",
                    onclick: move_up,
                    disabled: clamped_top <= 0.0,
                    id: "up-button",
                    "↑"
                }
                button {
                    class: "nav-btn",
                    onclick: move_down,
                    disabled: clamped_top >= max_scroll_top,
                    id: "down-button",
                    "↓"
                }
                button {
                    class: "nav-btn",
                    onclick: move_left,
                    disabled: clamped_left <= 0.0,
                    id: "left-button",
                    "←"
                }
                button {
                    class: "nav-btn",
                    onclick: move_right,
                    disabled: clamped_left >= max_scroll_left,
                    id: "right-button",
                    "→"
                }
            }
        }
    }
}
