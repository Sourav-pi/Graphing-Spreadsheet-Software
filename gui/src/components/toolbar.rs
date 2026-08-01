use super::error_display::{show_error, ErrorContext, ErrorType};
use super::spreadsheet::*;
use cores::convert_to_index;
use cores::Sheet;
use dioxus::prelude::*;
use std::sync::Arc;
use std::sync::Mutex;

const OPEN_ICON: Asset = asset!("assets/open.png");
const SAVE_ICON: Asset = asset!("assets/save.png");
const VISUALIZE_ICON: Asset = asset!("assets/visualize.png");
const NEWFILE_ICON: Asset = asset!("assets/newfile.png");
const LOAD_CSV_ICON: Asset = asset!("assets/load_csv.png");
const SAVE_CSV_ICON: Asset = asset!("assets/save_csv.png");

#[derive(Props, PartialEq, Clone)]
pub struct ToolbarProps {
    pub num_rows: usize,
    pub num_cols: usize,
}

#[component]
pub fn Toolbar(props: ToolbarProps) -> Element {
    let mut cur_file = use_context::<CurrentFileContext>();
    let mut is_open = use_context::<GraphPopupContext>();
    let mut start_row_ctx = use_context::<StartRowContext>();
    let mut start_col_ctx = use_context::<StartColContext>();
    let mut search_term = use_signal(String::new);
    let mut selected_cell = use_context::<SelectedCellContext>();
    let mut error_ctx = use_context::<ErrorContext>();
    let mut sheet = use_context::<SheetContext>();
    let mut sheetversion = use_context::<SheetVersionContext>();

    rsx! {
      div { class: "toolbar",
      // Search bar
      div { class: "search-group",
          input {
              class: "search-input",
              placeholder: "Cell Number",
              value: "{search_term}",
              oninput: move |evt| search_term.set(evt.value().clone()),
              onkeydown: move |evt| {
                  if evt.key() == Key::Enter {
                      let (a ,b) = convert_to_index(search_term.cloned());
                      let (a,b) = (a as i32,b as i32);
                      // println!("ans {a} {b}");
                      if !(a==0 && b==0) && a <= props.num_rows as i32 && b <= props.num_cols as i32 {
                          start_row_ctx.set(a-1);
                          start_col_ctx.set(b-1);
                          selected_cell.set((a,b));

                          let _ =document::eval(&format!("document.getElementById('row-{}-col-{}').focus()",a,b));
                      } else {
                          show_error(&mut error_ctx, "Invalid Cell !!", ErrorType::Error, Some(5.0));
                          search_term.set(String::new());
                      }
                  }
              }
          }
          button {
              class: "search-btn",
              onclick: move |_| {

                  let (a ,b) = convert_to_index(search_term.cloned());
                  let (a,b) = (a as i32,b as i32);
                  // println!("ans {a} {b}");
                  if !(a==0 && b==0) && a <= props.num_rows as i32 && b <= props.num_cols as i32 {
                      start_row_ctx.set(a-1);
                      start_col_ctx.set(b-1);
                      selected_cell.set((a,b));

                      let _ =document::eval(&format!("document.getElementById('row-{}-col-{}').focus()",a,b));
                  } else {
                      show_error(&mut error_ctx, "Invalid Cell !!", ErrorType::Error, Some(5.0));
                      search_term.set(String::new());
                  }
              },
              "GO"
          }
      }
      div { class: "toolbar-divider" }
      button {
          class: "toolbar-btn",
          onclick: move |_| {
              let new_sheet = Sheet::new(1000, 18279);
              sheet.set(Arc::new(Mutex::new(new_sheet)));
              sheetversion.set(0);
          },
          img {
              src: "{NEWFILE_ICON}",
              alt: "New File",
              class: "toolbar-icon",
          }
      },
      button { class: "toolbar-btn",
          onclick: move |_| {
              let path = std::env::current_dir().unwrap();

              let res = rfd::FileDialog::new()
                  .add_filter("spreadsheet", &["ss"])
                  .set_directory(&path)
                  .pick_file();

              if let Some(file) = &res {
                  cur_file.set(Some(file.clone()));
                  let file_path = file.as_path().to_string_lossy().to_string();
                  // Save the current sheet to the selected file
                  if let Ok(mut sheet_locked) = sheet.cloned().lock() {
                      // Update the cell value in the Sheet object
                      let write_result = sheet_locked.read_file(&file_path);
                      sheetversion.set(sheetversion.cloned() + 1);
                      if let Err(e) = write_result {
                          show_error(&mut error_ctx, &format!("Error reading from file: {}", e), ErrorType::Error, Some(5.0));
                      } else {
                          show_error(&mut error_ctx, "File loaded successfully", ErrorType::Success, Some(5.0));
                          sheetversion.set(sheetversion.cloned() + 1);
                      }
                  }
              }


          },
          img {
              src: "{OPEN_ICON}",
              alt: "Open",
              class: "toolbar-icon"
          }
      },
      button { class: "toolbar-btn",
          onclick: move |_| {
              let path = std::env::current_dir().unwrap();
              let res = rfd::FileDialog::new().add_filter("spreadsheet", &["ss"])
              .set_file_name("new_sheet.ss")
              .set_directory(&path)
              .save_file();
              // println!("The user choose: {:#?}", res);
              if let Some(file) = &res {
                  let file_path = file.as_path().to_string_lossy().to_string();
                  // Save the current sheet to the selected file
                  if let Ok(sheet_locked) = sheet.cloned().lock() {
                      // Update the cell value in the Sheet object
                      let write_result = sheet_locked.write_file(&file_path);
                      sheetversion.set(sheetversion.cloned() + 1);
                      if let Err(e) = write_result {
                          show_error(&mut error_ctx, &format!("Error writing to file: {}", e), ErrorType::Error, Some(5.0));
                      } else {
                          show_error(&mut error_ctx, "File saved successfully", ErrorType::Success, Some(5.0));
                          sheetversion.set(sheetversion.cloned() + 1);
                      }
                  }
              }



          },
          img {
              src: "{SAVE_ICON}",
              alt: "Save",
              class: "toolbar-icon"
          }
      },
      button {
          class: "toolbar-btn",
          onclick: move |_| {
              is_open.set(true);
          },
          img {
              src: "{VISUALIZE_ICON}",
              alt: "Visualize",
              class: "toolbar-icon"
          }
      },
      div { class: "toolbar-divider" }
      button { class: "toolbar-btn",
        onclick: move |_| {
        let path = std::env::current_dir().unwrap();

        let res = rfd::FileDialog::new()
            .add_filter("csv", &["csv"])
            .set_directory(&path)
            .pick_file();

        if let Some(file) = &res {
            cur_file.set(Some(file.clone()));
            let file_path = file.as_path().to_string_lossy().to_string();
            if let Ok(mut sheet_locked) = sheet.cloned().lock() {
                let write_result = sheet_locked.read_csv_file(&file_path);
                sheetversion.set(sheetversion.cloned() + 1);
                if let Err(e) = write_result {
                    show_error(&mut error_ctx, &format!("Error reading from file: {}", e), ErrorType::Error, Some(5.0));
                } else {
                    show_error(&mut error_ctx, "File loaded successfully", ErrorType::Success, Some(5.0));
                    sheetversion.set(sheetversion.cloned() + 1);
                }
            }
        }
    },

        img {
            src: "{LOAD_CSV_ICON}",
            alt: "Visualize",
            class: "toolbar-icon"
        }
    },
      button { class: "toolbar-btn",
        onclick: move |_| {
            let path = std::env::current_dir().unwrap();
            let res = rfd::FileDialog::new().add_filter("csv", &["csv"])
            .set_file_name("new_sheet.csv")
            .set_directory(&path)
            .save_file();
            // println!("The user choose: {:#?}", res);
            if let Some(file) = &res {
                let file_path = file.as_path().to_string_lossy().to_string();
                // Save the current sheet to the selected file
                if let Ok(sheet_locked) = sheet.cloned().lock() {
                    // Update the cell value in the Sheet object
                    let write_result = sheet_locked.write_csv_file(&file_path);
                    sheetversion.set(sheetversion.cloned() + 1);
                    if let Err(e) = write_result {
                        show_error(&mut error_ctx, &format!("Error writing to file: {}", e), ErrorType::Error, Some(5.0));
                    } else {
                        show_error(&mut error_ctx, "File saved successfully", ErrorType::Success, Some(5.0));
                        sheetversion.set(sheetversion.cloned() + 1);
                    }
                }
            }
        },

        img {
            src: "{SAVE_CSV_ICON}",
            alt: "Visualize",
            class: "toolbar-icon"
        }
    },
      }
    }
}
