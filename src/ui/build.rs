use crate::core;
use crate::ui::UiHandle;
use crate::ui::controller::selection::selected_or_first;
use crate::ui::model::entries_store;
use crate::ui::view;
use crate::ui::view::list_item_factory as lif;
use gtk4::prelude::*;

const WINDOW_WIDTH: i32 = 700;
const WINDOW_MIN_HEIGHT: i32 = 140;
const WINDOW_MAX_HEIGHT: i32 = 420;
const PROMPT_HEIGHT_ESTIMATE: i32 = 44;
const CHROME_HEIGHT_ESTIMATE: i32 = 28;
const ROW_HEIGHT_ESTIMATE: i32 = 44;

pub fn build_ui(app: &gtk4::Application, state: core::SharedState) -> UiHandle {
    let window = view::window::build_main_window(app);
    window.add_css_class("launcher-window");

    let store = {
        let s = state.lock();
        entries_store::build_entries_store(&s.apps)
    };

    let selection_model = gtk4::SingleSelection::new(Some(store.clone()));
    let factory = lif::build_factory();
    let list_view = gtk4::ListView::new(Some(selection_model.clone()), Some(factory.clone()));
    list_view.add_css_class("launcher-list");
    list_view.set_vexpand(false);
    list_view.set_focusable(true);

    let scrolled_list = gtk4::ScrolledWindow::builder()
        .vexpand(false)
        .hexpand(true)
        .build();
    scrolled_list.add_css_class("launcher-scroll");

    let prompt = gtk4::Entry::builder().hexpand(true).build();
    prompt.add_css_class("launcher-prompt");
    prompt.grab_focus();

    let root = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(8)
        .hexpand(true)
        .build();
    root.add_css_class("launcher-root");

    root.append(&prompt);
    root.append(&scrolled_list);

    scrolled_list.set_child(Some(&list_view));
    window.set_child(Some(&root));

    let ui = UiHandle {
        main_window: window,
        scrolled_window: scrolled_list,
        entries_store: store,
        selection_model,
        entries_list: list_view,
        prompt,
    };

    update_window_height(&ui);
    ui
}

pub fn update_entries(ui: &UiHandle, apps: &[core::AppEntry]) {
    entries_store::update_entries_store(&ui.entries_store, apps);

    if ui.selection_model.n_items() > 0 {
        ui.selection_model
            .set_selected(selected_or_first(&ui.selection_model).unwrap_or(0));
    }

    update_window_height(ui);
}

pub fn update_window_height(ui: &UiHandle) {
    let items = ui.selection_model.n_items() as i32;
    let list_height = items.saturating_mul(ROW_HEIGHT_ESTIMATE);
    let unclamped = PROMPT_HEIGHT_ESTIMATE + CHROME_HEIGHT_ESTIMATE + list_height;
    let target_height = unclamped.clamp(WINDOW_MIN_HEIGHT, WINDOW_MAX_HEIGHT);
    let max_list_height = (WINDOW_MAX_HEIGHT - PROMPT_HEIGHT_ESTIMATE - CHROME_HEIGHT_ESTIMATE).max(0);

    ui.scrolled_window
        .set_max_content_height(max_list_height);
    ui.scrolled_window
        .set_min_content_height(list_height.min(max_list_height));
    ui.main_window
        .set_default_size(WINDOW_WIDTH, target_height);
}
