use super::error_display::{show_error, ErrorContext, ErrorType};
use super::spreadsheet::SheetContext;
use dioxus::prelude::*;

// Helper component for each type of graph input
#[component]
pub fn LineChartForm() -> Element {
    let mut show_graph = use_signal(|| false);
    let mut range = use_signal(String::new);
    let mut x_label = use_signal(String::new);
    let mut y_label = use_signal(String::new);
    let mut title = use_signal(String::new);
    let point_labels = use_signal(String::new);
    let sheet = use_context::<SheetContext>();
    let mut chart_json = use_signal(String::new);
    let mut error_ctx = use_context::<ErrorContext>();

    if show_graph.cloned() {
        if let Ok(sheet_locked) = sheet.cloned().lock() {
            // Update the cell value in the Sheet object
            let x = sheet_locked.line_graph(
                &range.cloned(),
                &x_label.cloned(),
                &y_label.cloned(),
                &title.cloned(),
            );
            if let Ok(json) = x {
                chart_json.set(json);
            } else {
                show_graph.set(false);
                range.set(String::new());
                show_error(
                    &mut error_ctx,
                    &format!("Error generating chart: {}", x.err().unwrap()),
                    ErrorType::Error,
                    Some(5.0),
                );
                return rsx! {};
            }
        }
        let mount_code = format!(
            r#"
                var millis = 500;
                setTimeout(function() {{
                    var chart = echarts.init(document.getElementById('chart'), null, {{renderer: 'canvas'}});
                    window.addEventListener('resize', function() {{
                        chart.resize();
                    }});
                    chart.setOption({});
                }}, millis)
            "#,
            chart_json.cloned()
        );
        return rsx! {
            document::Script { src: asset!("/assets/echarts-5.5.1.min.js") }
            div { style: "width: 100%; text-align: center; display: flex; justify-content: center; align-items: center;",
            div { id: "chart", style: "display: inline-block; height: 80%; width:80% ; background-color: white; border-radius: 10px;", onmounted: move |_| {
                document::eval(&mount_code);
            },}
        }

        };
    } else {
        rsx! {
            div { class: "form-grid",
                div {
                    div { class: "form-label", "Cell Range (e.g., A1:A10)" }
                    input {
                        class: "form-input",
                        placeholder: "Enter cell range",
                        oninput: move |evt| range.set(evt.value().clone())
                    }
                }
                div {
                    div { class: "form-label", "X-Axis Label" }
                    input {
                        class: "form-input",
                        placeholder: "X-Axis",
                        oninput: move |evt| x_label.set(evt.value().clone())
                    }
                }
                div {
                    div { class: "form-label", "Y-Axis Label" }
                    input {
                        class: "form-input",
                        placeholder: "Y-Axis",
                        oninput: move |evt| y_label.set(evt.value().clone())
                    }
                }
                div {
                    div { class: "form-label", "Chart Title (Optional)" }
                    input {
                        class: "form-input",
                        placeholder: "Enter title",
                        oninput: move |evt| title.set(evt.value().clone())
                    }
                }
                button {
                    class: "form-submit",
                    onclick: move |_| {
                        // Handle submission
                        println!("Line chart with range: {}, x: {}, y: {}, title: {}, labels: {}",
                                 range.cloned(), x_label.cloned(), y_label.cloned(), title.cloned(), point_labels.cloned());
                        show_graph.set(true);
                    },
                    "Generate Chart"
                }
            }
        }
    }
}

#[component]
pub fn BarChartForm() -> Element {
    let mut show_graph = use_signal(|| false);
    let mut range = use_signal(String::new);
    let mut y_label = use_signal(String::new);
    let mut title = use_signal(String::new);
    let mut bar_labels = use_signal(String::new);
    let sheet = use_context::<SheetContext>();
    let mut chart_json = use_signal(String::new);
    let mut error_ctx = use_context::<ErrorContext>();

    if show_graph.cloned() {
        if let Ok(sheet_locked) = sheet.cloned().lock() {
            // Generate bar chart
            let x = sheet_locked.bar_graph(
                &range.cloned(),
                &bar_labels.cloned(),
                &y_label.cloned(),
                &title.cloned(),
            );
            if let Ok(json) = x {
                chart_json.set(json);
            } else {
                show_graph.set(false);
                range.set(String::new());
                show_error(
                    &mut error_ctx,
                    &format!("Error generating bar chart: {}", x.err().unwrap()),
                    ErrorType::Error,
                    Some(5.0),
                );
                return rsx! {};
            }
        }

        let mount_code = format!(
            r#"
                var millis = 500;
                setTimeout(function() {{
                    var chart = echarts.init(document.getElementById('chart'), null, {{renderer: 'canvas'}});
                    window.addEventListener('resize', function() {{
                        chart.resize();
                    }});
                    chart.setOption({});
                }}, millis)
            "#,
            chart_json.cloned()
        );

        return rsx! {
            document::Script { src: asset!("/assets/echarts-5.5.1.min.js") }
            div { style: "width: 100%; text-align: center; display: flex; justify-content: center; align-items: center;",
                div {
                    id: "chart",
                    style: "display: inline-block; height: 80%; width:80%; background-color: white; border-radius: 10px;",
                    onmounted: move |_| {
                        document::eval(&mount_code);
                    }
                }
            }
        };
    } else {
        rsx! {
            div { class: "form-grid",
                div {
                    div { class: "form-label", "Cell Range (e.g., A1:A10)" }
                    input {
                        class: "form-input",
                        placeholder: "Enter cell range",
                        oninput: move |evt| range.set(evt.value().clone())
                    }
                }
                div {
                    div { class: "form-label", "Y-Axis Label" }
                    input {
                        class: "form-input",
                        placeholder: "Y-Axis",
                        oninput: move |evt| y_label.set(evt.value().clone())
                    }
                }
                div {
                    div { class: "form-label", "Bar Labels (Comma-separated)" }
                    input {
                        class: "form-input",
                        placeholder: "Comma-separated labels",
                        oninput: move |evt| bar_labels.set(evt.value().clone())
                    }
                }
                div {
                    div { class: "form-label", "Chart Title (Optional)" }
                    input {
                        class: "form-input",
                        placeholder: "Enter title",
                        oninput: move |evt| title.set(evt.value().clone())
                    }
                }
                button {
                    class: "form-submit",
                    onclick: move |_| {
                        println!("Bar chart with range: {}, labels: {}, y: {}, title: {}",
                                range.cloned(), bar_labels.cloned(), y_label.cloned(), title.cloned());
                        show_graph.set(true);
                    },
                    "Generate Chart"
                }
            }
        }
    }
}

#[component]
pub fn PieChartForm() -> Element {
    let mut show_graph = use_signal(|| false);
    let mut range = use_signal(String::new);
    let mut title = use_signal(String::new);
    let mut slice_labels = use_signal(String::new);
    let sheet = use_context::<SheetContext>();
    let mut chart_json = use_signal(String::new);
    let mut error_ctx = use_context::<ErrorContext>();

    if show_graph.cloned() {
        if let Ok(sheet_locked) = sheet.cloned().lock() {
            // Generate pie chart
            let x =
                sheet_locked.pie_graph(&range.cloned(), &slice_labels.cloned(), &title.cloned());
            if let Ok(json) = x {
                chart_json.set(json);
            } else {
                show_graph.set(false);
                range.set(String::new());
                show_error(
                    &mut error_ctx,
                    &format!("Error generating pie chart: {}", x.err().unwrap()),
                    ErrorType::Error,
                    Some(5.0),
                );
                return rsx! {};
            }
        }

        let mount_code = format!(
            r#"
                var millis = 500;
                setTimeout(function() {{
                    var chart = echarts.init(document.getElementById('pie-chart'), null, {{renderer: 'canvas'}});
                    window.addEventListener('resize', function() {{
                        chart.resize();
                    }});
                    chart.setOption({});
                }}, millis)
            "#,
            chart_json.cloned()
        );

        return rsx! {
            document::Script { src: asset!("/assets/echarts-5.5.1.min.js") }
            div { style: "width: 100%; text-align: center; display: flex; justify-content: center; align-items: center;",
                div {
                    id: "pie-chart",
                    style: "display: inline-block; height: 80%; width:80%; background-color: white; border-radius: 10px;",
                    onmounted: move |_| {
                        document::eval(&mount_code);
                    }
                }
            }
        };
    } else {
        rsx! {
            div { class: "form-grid",
                div {
                    div { class: "form-label", "Cell Range (e.g., A1:A10,B1:B10)" }
                    input {
                        class: "form-input",
                        placeholder: "Enter cell range",
                        oninput: move |evt| range.set(evt.value().clone())
                    }
                }
                div {
                    div { class: "form-label", "Chart Title (Optional)" }
                    input {
                        class: "form-input",
                        placeholder: "Enter title",
                        oninput: move |evt| title.set(evt.value().clone())
                    }
                }
                div {
                    div { class: "form-label", "Slice Labels (Comma-separated)" }
                    input {
                        class: "form-input",
                        placeholder: "Comma-separated labels",
                        oninput: move |evt| slice_labels.set(evt.value().clone())
                    }
                }
                // Keep the placeholder divs for consistent layout
                div { style : "visibility: hidden ",
                    div { class: "form-label", "placeholder" }
                    input {
                        class: "form-input",
                        placeholder: "placeholder"
                    }
                }
                div { style : "visibility: hidden ",
                    div { class: "form-label", "placeholder" }
                    input {
                        class: "form-input",
                        placeholder: "placeholder"
                    }
                }
                button {
                    class: "form-submit",
                    onclick: move |_| {
                        println!("Pie chart with range: {}, title: {}, labels: {}",
                                range.cloned(), title.cloned(), slice_labels.cloned());
                        show_graph.set(true);
                    },
                    "Generate Chart"
                }
            }
        }
    }
}

#[component]
pub fn ScatterPlotForm() -> Element {
    let mut show_graph = use_signal(|| false);
    let mut x_range = use_signal(String::new);
    let mut y_range = use_signal(String::new);
    let mut x_label = use_signal(String::new);
    let mut y_label = use_signal(String::new);
    let mut title = use_signal(String::new);
    let sheet = use_context::<SheetContext>();
    let mut chart_json = use_signal(String::new);
    let mut error_ctx = use_context::<ErrorContext>();

    if show_graph.cloned() {
        if let Ok(sheet_locked) = sheet.cloned().lock() {
            // Generate scatter plot
            let x = sheet_locked.scatter_graph(
                &x_range.cloned(),
                &y_range.cloned(),
                &title.cloned(),
                &x_label.cloned(),
                &y_label.cloned(),
            );
            if let Ok(json) = x {
                chart_json.set(json);
            } else {
                show_graph.set(false);
                x_range.set(String::new());
                y_range.set(String::new());
                show_error(
                    &mut error_ctx,
                    &format!("Error generating scatter plot: {}", x.err().unwrap()),
                    ErrorType::Error,
                    Some(5.0),
                );
                return rsx! {};
            }
        }

        let mount_code = format!(
            r#"
                var millis = 500;
                setTimeout(function() {{
                    var chart = echarts.init(document.getElementById('scatter-chart'), null, {{renderer: 'canvas'}});
                    window.addEventListener('resize', function() {{
                        chart.resize();
                    }});
                    chart.setOption({});
                }}, millis)
            "#,
            chart_json.cloned()
        );

        return rsx! {
            document::Script { src: asset!("/assets/echarts-5.5.1.min.js") }
            div { style: "width: 100%; text-align: center; display: flex; justify-content: center; align-items: center;",
                div {
                    id: "scatter-chart",
                    style: "display: inline-block; height: 80%; width:80%; background-color: white; border-radius: 10px;",
                    onmounted: move |_| {
                        document::eval(&mount_code);
                    }
                }
            }
        };
    } else {
        rsx! {
            div { class: "form-grid",
                div {
                    div { class: "form-label", "X-Axis Cell Range (e.g., A1:A10)" }
                    input {
                        class: "form-input",
                        placeholder: "Enter X-axis cell range",
                        oninput: move |evt| x_range.set(evt.value().clone())
                    }
                }
                div {
                    div { class: "form-label", "Y-Axis Cell Range (e.g., B1:B10)" }
                    input {
                        class: "form-input",
                        placeholder: "Enter Y-axis cell range",
                        oninput: move |evt| y_range.set(evt.value().clone())
                    }
                }
                div {
                    div { class: "form-label", "X-Axis Label" }
                    input {
                        class: "form-input",
                        placeholder: "X-Axis",
                        oninput: move |evt| x_label.set(evt.value().clone())
                    }
                }
                div {
                    div { class: "form-label", "Y-Axis Label" }
                    input {
                        class: "form-input",
                        placeholder: "Y-Axis",
                        oninput: move |evt| y_label.set(evt.value().clone())
                    }
                }
                div {
                    div { class: "form-label", "Chart Title (Optional)" }
                    input {
                        class: "form-input",
                        placeholder: "Enter title",
                        oninput: move |evt| title.set(evt.value().clone())
                    }
                }
                button {
                    class: "form-submit",
                    onclick: move |_| {
                        println!("Scatter plot with x-range: {}, y-range: {}, x-label: {}, y-label: {}, title: {}",
                                x_range.cloned(), y_range.cloned(), x_label.cloned(), y_label.cloned(), title.cloned());
                        show_graph.set(true);
                    },
                    "Generate Chart"
                }
            }
        }
    }
}
