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
    pub prompt: gtk4::Entry,
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
    list_view.set_focusable(true);
    list_view.connect_activate({
        let selection = selection_model.clone();
        let window = window.clone();
        let state = state.clone();

        move |_view, position| {
            activate_position(&selection, &state, &window, position);
        }
    });

    // Build the scrolling view here
    let scrolled_list = gtk4::ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .build();

    // Create the prompt
    let prompt = gtk4::Entry::builder().hexpand(true).build();
    prompt.grab_focus();
    let prompt_keys = gtk4::EventControllerKey::new();
    prompt_keys.connect_key_pressed({
        let selection = selection_model.clone();
        let list_view = list_view.clone();
        let state = state.clone();
        let window = window.clone();

        move |_, key, _keycode, _state| match key {
            gtk4::gdk::Key::Up => {
                select_prev(&selection);
                focus_list_for_navigation(&selection, &list_view);
                gtk4::glib::Propagation::Stop
            }
            gtk4::gdk::Key::Down => {
                select_next(&selection);
                focus_list_for_navigation(&selection, &list_view);
                gtk4::glib::Propagation::Stop
            }
            gtk4::gdk::Key::Return | gtk4::gdk::Key::KP_Enter => {
                activate_selected(&selection, &state, &window);
                gtk4::glib::Propagation::Stop
            }
            _ => gtk4::glib::Propagation::Proceed,
        }
    });
    prompt.add_controller(prompt_keys);

    let list_keys = gtk4::EventControllerKey::new();
    list_keys.connect_key_pressed({
        let prompt = prompt.clone();

        move |_, key, _keycode, state| {
            if route_text_key_to_prompt(&prompt, key, state) {
                return gtk4::glib::Propagation::Stop;
            }

            if route_backspace_to_prompt(&prompt, key) {
                return gtk4::glib::Propagation::Stop;
            }

            if route_delete_to_prompt(&prompt, key) {
                return gtk4::glib::Propagation::Stop;
            }

            gtk4::glib::Propagation::Proceed
        }
    });
    list_view.add_controller(list_keys);

    // Make tree
    let root = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .hexpand(true)
        .build();

    root.append(&prompt);
    root.append(&scrolled_list);

    // Set children appropriately
    scrolled_list.set_child(Some(&list_view));
    window.set_child(Some(&root));

    UiHandle {
        main_window: window,
        scrolled_window: scrolled_list,
        entries_store: store,
        selection_model,
        entries_list: list_view,
        prompt: prompt,
    }
}

pub fn update_entries(ui: &UiHandle, apps: &[core::AppEntry]) {
    ui.entries_store.remove_all();
    for entry in apps {
        ui.entries_store.append(&aep::AppEntryObject::new(entry));
    }

    if ui.selection_model.n_items() > 0 {
        ui.selection_model
            .set_selected(selected_or_first(&ui.selection_model).unwrap_or(0));
    }
}

pub fn prepare_for_show(ui: &UiHandle) {
    ui.main_window.present();

    if ui.selection_model.n_items() > 0 {
        ui.selection_model
            .set_selected(selected_or_first(&ui.selection_model).unwrap_or(0));
    }

    ui.prompt.grab_focus();
}

// -------- HELPERS
fn activate_position(
    selection: &gtk4::SingleSelection,
    state: &core::SharedState,
    window: &gtk4::ApplicationWindow,
    position: u32,
) {
    let obj = selection
        .model()
        .and_then(|m| m.item(position))
        .and_downcast::<aep::AppEntryObject>();

    let exec = obj.and_then(|obj| {
        let s = state.lock();
        s.apps
            .iter()
            .find(|app| app.id == obj.id())
            .map(|app| app.exec.clone())
    });

    if let Some(exec) = exec {
        if core::desktop::launch_exec(&exec).is_ok() {
            window.hide();
        }
    }
}

fn selected_or_first(selection: &gtk4::SingleSelection) -> Option<u32> {
    let count = selection.n_items();
    if count == 0 {
        return None;
    }

    let selected = selection.selected();
    if selected < count {
        Some(selected)
    } else {
        Some(0)
    }
}

fn select_prev(selection: &gtk4::SingleSelection) {
    if let Some(current) = selected_or_first(selection) {
        selection.set_selected(current.saturating_sub(1));
    }
}

fn select_next(selection: &gtk4::SingleSelection) {
    let count = selection.n_items();
    if let Some(current) = selected_or_first(selection) {
        selection.set_selected((current + 1).min(count - 1));
    }
}

fn activate_selected(
    selection: &gtk4::SingleSelection,
    state: &core::SharedState,
    window: &gtk4::ApplicationWindow,
) {
    if let Some(position) = selected_or_first(selection) {
        activate_position(selection, state, window, position);
    }
}

fn focus_list_for_navigation(selection: &gtk4::SingleSelection, list_view: &gtk4::ListView) {
    if let Some(position) = selected_or_first(selection) {
        selection.set_selected(position);
    }
    list_view.grab_focus();
}

fn route_text_key_to_prompt(
    prompt: &gtk4::Entry,
    key: gtk4::gdk::Key,
    state: gtk4::gdk::ModifierType,
) -> bool {
    if state.intersects(
        gtk4::gdk::ModifierType::CONTROL_MASK
            | gtk4::gdk::ModifierType::ALT_MASK
            | gtk4::gdk::ModifierType::META_MASK,
    ) {
        return false;
    }

    let ch = match key.to_unicode() {
        Some(ch) => ch,
        None => return false,
    };

    if ch.is_control() {
        return false;
    }

    prompt.grab_focus();
    let mut pos = prompt.position();
    let text = ch.to_string();
    prompt.insert_text(&text, &mut pos);
    prompt.set_position(pos);
    true
}

fn route_backspace_to_prompt(prompt: &gtk4::Entry, key: gtk4::gdk::Key) -> bool {
    if key != gtk4::gdk::Key::BackSpace {
        return false;
    }

    prompt.grab_focus();
    let pos = prompt.position();
    if pos > 0 {
        prompt.delete_text(pos - 1, pos);
        prompt.set_position(pos - 1);
    }
    true
}

fn route_delete_to_prompt(prompt: &gtk4::Entry, key: gtk4::gdk::Key) -> bool {
    if key != gtk4::gdk::Key::Delete {
        return false;
    }

    prompt.grab_focus();
    let pos = prompt.position();
    prompt.delete_text(pos, pos + 1);
    prompt.set_position(pos);
    true
}
