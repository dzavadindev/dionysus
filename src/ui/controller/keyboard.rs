use crate::core;
use crate::ui::controller::activation::activate_selected;
use crate::ui::controller::selection::{focus_list_for_navigation, select_next, select_prev};

pub fn handle_global_key(
    key: gtk4::gdk::Key,
    selection_model: &gtk4::SingleSelection,
    entries_list: &gtk4::ListView,
    state: &core::SharedState,
    main_window: &gtk4::ApplicationWindow,
) -> gtk4::glib::Propagation {
    match key {
        gtk4::gdk::Key::Up => {
            select_prev(selection_model);
            focus_list_for_navigation(selection_model, entries_list);
            gtk4::glib::Propagation::Stop
        }
        gtk4::gdk::Key::Down => {
            select_next(selection_model);
            focus_list_for_navigation(selection_model, entries_list);
            gtk4::glib::Propagation::Stop
        }
        gtk4::gdk::Key::Return | gtk4::gdk::Key::KP_Enter => {
            activate_selected(selection_model, state, main_window);
            gtk4::glib::Propagation::Stop
        }
        _ => gtk4::glib::Propagation::Proceed,
    }
}
