
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

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
        "save" => dialog.save_file(save_file),
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

fn save_file(file_path: Option<PathBuf>) {
    if let Some(file_path) = file_path {
        // Read file contents
        let mut file = File::open(file_path.clone()).expect("Unable to open file");
        let mut file_contents = String::new();
        file.read_to_string(&mut file_contents).expect("Unable to read file");
        println!("Saving file contents to {:?}: {}", file_path, file_contents);
        // Send event to tell frontend to save file (by calling a tauri command)?
        // window.emit_all("open-file", file_contents).expect("Unable to send file contents to UI");
    }
}
