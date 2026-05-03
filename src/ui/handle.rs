use gtk4::gio;

#[derive(Clone)]
pub struct UiHandle {
    pub main_window: gtk4::ApplicationWindow,
    pub scrolled_window: gtk4::ScrolledWindow,
    pub entries_store: gio::ListStore,
    pub selection_model: gtk4::SingleSelection,
    pub entries_list: gtk4::ListView,
    pub prompt: gtk4::Entry,
}
