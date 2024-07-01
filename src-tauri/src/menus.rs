
use std::path::PathBuf;
use std::fs::File;
use std::io::{Read, Write};

use tauri::{CustomMenuItem, Manager, Menu, Window, WindowMenuEvent};
use tauri::api::dialog::FileDialogBuilder;
// ? Could use tauri::api::dialog::blocking::FileDialogBuilder;


pub fn get_menus() -> Menu {
    Menu::new()
        .add_item(CustomMenuItem::new("open", "Open"))
        .add_item(CustomMenuItem::new("save", "Save"))
}

pub fn handle_menu_event(event: WindowMenuEvent) {
    let menu_id = event.menu_item_id();
    let dialog = FileDialogBuilder::new();
    let dialog = dialog.add_filter("Markdown", &["md"]);
    match menu_id {
        "open" => dialog.pick_file(move |file_path| open_file_on_ui(event.window(), file_path)),
        "save" => dialog.save_file(move |file_path| emit_file_save(event.window(), file_path)),
        _ => {}
    };
}

fn open_file_on_ui(window: &Window, file_path: Option<PathBuf>) {
    if let Some(file_path) = file_path {
        // Read file contents
        let mut file = File::open(file_path).expect("Unable to open file");
        let mut file_contents = String::new();
        file.read_to_string(&mut file_contents).expect("Unable to read file");
        // Send to UI
        window.emit_all("open-file", file_contents).expect("Unable to send file contents to UI");
    }
}

fn emit_file_save(window: &Window, file_path: Option<PathBuf>) {
    if let Some(file_path) = file_path {
        window.emit_all("save-to-path", file_path).expect("Unable to send file contents to UI");
    }
}

#[tauri::command]
pub fn save_file(path: String, content: String) {
    let mut file = File::create(path).expect("Unable to create file");
    file.write_all(content.as_bytes()).expect("Unable to write to file");
}
