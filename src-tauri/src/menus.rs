
use tauri::{CustomMenuItem, Menu, WindowMenuEvent};
use tauri::api::dialog::FileDialogBuilder;
// ? Could use tauri::api::dialog::blocking::FileDialogBuilder;


pub fn get_menus() -> Menu {
    Menu::new()
        .add_item(CustomMenuItem::new("open", "Open"))
        .add_item(CustomMenuItem::new("save", "Save"))
}

pub fn handle_menu_event(event: WindowMenuEvent) {
    let dialog = FileDialogBuilder::new();
    let dialog = dialog.add_filter("Markdown", &["md"]);
    match event.menu_item_id() {
        "open" => dialog.pick_file(|file_path| {println!("Open {:?}", file_path)}),
        "save" => dialog.save_file(|file_path| {println!("Save {:?}", file_path)}),
        _ => {}
    };
}
