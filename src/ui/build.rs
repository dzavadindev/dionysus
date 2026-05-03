use crate::core;
use crate::ui::UiHandle;
use crate::ui::controller::selection::selected_or_first;
use crate::ui::model::entries_store;
use crate::ui::view;
use crate::ui::view::list_item_factory as lif;
use gtk4::prelude::*;

pub fn build_ui(app: &gtk4::Application, state: core::SharedState) -> UiHandle {
    let window = view::window::build_main_window(app);

    let store = {
        let s = state.lock();
        entries_store::build_entries_store(&s.apps)
    };

    let selection_model = gtk4::SingleSelection::new(Some(store.clone()));
    let factory = lif::build_factory();
    let list_view = gtk4::ListView::new(Some(selection_model.clone()), Some(factory.clone()));
    list_view.set_vexpand(true);
    list_view.set_focusable(true);

    let scrolled_list = gtk4::ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .build();

    let prompt = gtk4::Entry::builder().hexpand(true).build();
    prompt.grab_focus();

    let root = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .hexpand(true)
        .build();

    root.append(&prompt);
    root.append(&scrolled_list);

    scrolled_list.set_child(Some(&list_view));
    window.set_child(Some(&root));

    UiHandle {
        main_window: window,
        scrolled_window: scrolled_list,
        entries_store: store,
        selection_model,
        entries_list: list_view,
        prompt,
    }
}

pub fn update_entries(ui: &UiHandle, apps: &[core::AppEntry]) {
    entries_store::update_entries_store(&ui.entries_store, apps);

    if ui.selection_model.n_items() > 0 {
        ui.selection_model
            .set_selected(selected_or_first(&ui.selection_model).unwrap_or(0));
    }
}
