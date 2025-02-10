use permission_manager::db::{PermissionDB, PermissionEntry, PermissionState};
use gtk::prelude::*;
use gtk::{CellRendererCombo, CellRendererText, ListStore, TreeView, TreeViewColumn};
use anyhow::Result;
use gtk::traits::CellLayoutExt;

fn build_gui(app: &gtk::Application) -> Result<()> {
    let window = gtk::ApplicationWindow::new(app);
    window.set_title("Permission Manager");
    window.set_default_size(600, 400);

    let tree_view = TreeView::new();
    let model = ListStore::new(&[String::static_type(), String::static_type(), String::static_type()]);
    tree_view.set_model(Some(&model));

    let db = PermissionDB::init()?;
    load_data(&model, db.get_all()?);

    // App Name Column
    let renderer = CellRendererText::new();
    let column = TreeViewColumn::new();
    column.set_title("Application");
    CellLayoutExt::pack_start(&column, &renderer, true);
    CellLayoutExt::add_attribute(&column, &renderer, "text", 0);
    tree_view.append_column(&column);

    // Permissions Column
    let renderer = CellRendererText::new();
    let column = TreeViewColumn::new();
    column.set_title("Requested Permissions");
    CellLayoutExt::pack_start(&column, &renderer, true);
    CellLayoutExt::add_attribute(&column, &renderer, "text", 1);
    tree_view.append_column(&column);

    // Permission State Column
    let (combo, _combo_model) = create_state_combo();
    let column = TreeViewColumn::new();
    column.set_title("Permission State");
    CellLayoutExt::pack_start(&column, &combo, true);
    CellLayoutExt::add_attribute(&column, &combo, "text", 2);
    tree_view.append_column(&column);

    connect_combo_changed(&combo, model.clone());

    window.add(&tree_view);
    window.show_all();

    Ok(())
}

fn create_state_combo() -> (CellRendererCombo, ListStore) {
    let combo = CellRendererCombo::new();
    let combo_model = ListStore::new(&[String::static_type()]);
    for state in &["Allow Once", "Always", "Block"] {
        combo_model.set(&combo_model.append(), &[(0, &state.to_string())]);
    }

    combo.set_property("editable", &true);
    combo.set_property("model", &combo_model);
    combo.set_property("text-column", &0);
    (combo, combo_model)
}

fn connect_combo_changed(combo: &CellRendererCombo, model: ListStore) {
    combo.connect_edited(move |_, path, new_text| {
        if let Some(iter) = model.iter(&path) {
            if let Ok(mut db) = PermissionDB::init() {
                let app_name = model.value(&iter, 0).get::<String>().unwrap();
                let entry = PermissionEntry {
                    app_name,
                    requested_permissions: vec![],
                    permission_state: match new_text {
                        "Allow Once" => PermissionState::AllowOnce,
                        "Always" => PermissionState::Always,
                        "Block" => PermissionState::Block,
                        _ => return,
                    },
                };
                let _ = db.upsert(&entry);
            }
        }
    });
}


fn load_data(model: &ListStore, entries: Vec<PermissionEntry>) {
    model.clear();
    for entry in entries {
        model.set(
            &model.append(),
            &[
                (0, &entry.app_name),
                (1, &entry.requested_permissions.join(", ")),
                (2, &entry.permission_state.to_string()),
            ],
        );
    }
}

fn main() -> Result<()> {
    let application = gtk::Application::new(
        Some("com.example.permission-manager"),
        Default::default(),
    );

    application.connect_activate(|app| 
    {
        if let Err(e) = build_gui(app) 
        {
            eprintln!("Error building GUI: {}", e);
        }
    });

    application.run();
    Ok(())
}
