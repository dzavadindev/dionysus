pub mod app_entry_object;
mod list_item;
mod list_item_factory;
mod window;

use crate::core;
use app_entry_object as aep;
use gtk4::gio;
use gtk4::prelude::*;
use list_item_factory as lif;

thread_local! {
    pub static UI_HANDLE: std::cell::RefCell<Option<UiHandle>> = std::cell::RefCell::new(None);
}

// -------- STRUCTS

#[derive(Clone)]
pub struct UiHandle {
    pub main_window: gtk4::ApplicationWindow,
    pub scrolled_window: gtk4::ScrolledWindow,
    pub entries_store: gio::ListStore,
    pub selection_model: gtk4::SingleSelection,
    pub entries_list: gtk4::ListView,
}

// -------- FUNCS

pub fn build_ui(app: &gtk4::Application, state: core::SharedState) -> UiHandle {
    let window = window::build_main_window(app);

    // Create the model (store)
    let store = gio::ListStore::new::<aep::AppEntryObject>();

    {
        // Fill the model with GObject converted AppEntries
        let s = state.lock();
        for entry in &s.apps {
            store.append(&aep::AppEntryObject::new(&entry));
        }
    }

    // Selection model
    let selection_model = gtk4::SingleSelection::new(Some(store.clone()));

    // Create the List Item factory
    let factory = lif::build_factory();

    // Create the view, pass the model to it
    let list_view = gtk4::ListView::new(Some(selection_model.clone()), Some(factory.clone()));
    list_view.set_vexpand(true);

    // Build the scrolling view here
    let scrolled_list = gtk4::ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .build();

    // Set children appropriately
    scrolled_list.set_child(Some(&list_view));
    window.set_child(Some(&scrolled_list));

    UiHandle {
        main_window: window,
        scrolled_window: scrolled_list,
        entries_store: store,
        selection_model,
        entries_list: list_view,
    }
}

pub fn update_entries(ui: &UiHandle, apps: &[core::AppEntry]) {
    ui.entries_store.remove_all();
    for entry in apps {
        ui.entries_store.append(&aep::AppEntryObject::new(entry));
    }
}
