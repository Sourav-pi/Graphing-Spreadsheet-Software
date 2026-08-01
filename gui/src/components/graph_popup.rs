use super::graph_forms::{BarChartForm, LineChartForm, PieChartForm, ScatterPlotForm};
use super::spreadsheet::{GraphPopupContext, GraphType, GraphTypeContext};
use ::dioxus::prelude::*;

#[component]
pub fn GraphPopup() -> Element {
    let mut is_open = use_context::<GraphPopupContext>();
    let mut graph_type = use_context::<GraphTypeContext>();

    let options = vec![
        ("Line Chart".to_string(), GraphType::Line),
        ("Bar Chart".to_string(), GraphType::Bar),
        ("Pie Chart".to_string(), GraphType::Pie),
        ("Scatter Plot".to_string(), GraphType::Scatter),
    ];
    if is_open.cloned() {
        rsx! {
            div { class: "popup-overlay",
                div {
                    class: "popup-panel",
                    div { class: "popup-header",
                        div { class: "popup-tabs",
                            {options.into_iter().map(|graph| {
                            let is_active = graph_type.cloned() == graph.1;
                            rsx! {button {
                                class: if is_active { "popup-tab active" } else { "popup-tab" },
                                onclick: move |_| {
                                    graph_type.set(graph.1);
                                },
                                "{graph.0}"
                            }}
                    })
                        }}
                        button {
                            class: "popup-close-btn",
                            onclick: move |_| {
                                is_open.set(false);
                            },
                            "×"
                        }
                    }
                    div {
                        class: "popup-body",
                        match graph_type.cloned() {
                            GraphType::Line => rsx! { LineChartForm {} },
                            GraphType::Bar => rsx! { BarChartForm {} },
                            GraphType::Pie => rsx! { PieChartForm {} },
                            GraphType::Scatter => rsx! { ScatterPlotForm {} },
                        }
                     }

                }

            }
        }
    } else {
        rsx! { div {} }
    }
}
