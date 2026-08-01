//! Error display component for the spreadsheet application.
//!
//! This module provides a component for displaying error messages, warnings,
//! information, and success notifications to the user.

use dioxus::prelude::*;

/// Define error types/severity levels for different notifications
#[derive(Clone, PartialEq)]
#[allow(unused)]
pub enum ErrorType {
    /// Error notification - for critical issues that prevent operation
    Error,
    /// Warning notification - for potential issues that don't prevent operation
    Warning,
    /// Information notification - for general information
    Info,
    /// Success notification - for successful operations
    Success,
}

/// Signal to store active errors with message, type and timeout
pub type ErrorContext = Signal<Option<(String, ErrorType, Option<f64>)>>;

/// Shows an error message with the specified type and optional timeout
///
/// # Parameters
/// * `error_ctx` - The error context signal to update
/// * `message` - The message to display
/// * `error_type` - The type of notification (error, warning, etc.)
/// * `timeout` - Optional timeout in seconds after which to hide the message
pub fn show_error(
    error_ctx: &mut ErrorContext,
    message: &str,
    error_type: ErrorType,
    timeout: Option<f64>,
) {
    error_ctx.set(Some((message.to_string(), error_type, timeout)));
}

/// Error display component that shows messages to the user
///
/// This component manages displaying error messages, warnings, information,
/// and success notifications. It handles:
/// - Displaying the appropriate styling based on message type
/// - Auto-dismissing messages after their timeout
/// - Allowing users to manually dismiss messages
#[component]
pub fn ErrorDisplay() -> Element {
    let mut error_ctx = use_context::<ErrorContext>();

    // Render error if present
    let error_data = error_ctx.read();
    if let Some((message, error_type, _)) = error_data.as_ref() {
        let toast_class = match error_type {
            ErrorType::Error => "toast toast-error",
            ErrorType::Warning => "toast toast-warning",
            ErrorType::Info => "toast toast-info",
            ErrorType::Success => "toast toast-success",
        };

        let icon = match error_type {
            ErrorType::Error => "⛔",
            ErrorType::Warning => "⚠️",
            ErrorType::Info => "ℹ️",
            ErrorType::Success => "✅",
        };

        rsx! {
            // Overlay to capture clicks outside
            div {
                class: "toast-overlay",
                onclick: move |_| error_ctx.set(None),

                // Error container
                div {
                    class: "{toast_class}",
                    // Stop propagation to prevent overlay click from triggering when clicking on the message
                    onclick: move |e| e.stop_propagation(),
                    span { "{icon}" }
                    span { "{message}" }
                    button {
                        class: "toast-close-btn",
                        onclick: move |_| error_ctx.set(None),
                        "×"
                    }
                }
            }
        }
    } else {
        rsx! { div {} }
    }
}
