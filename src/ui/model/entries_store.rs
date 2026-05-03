use crate::core;
use crate::ui::model::app_entry_object as aep;
use gtk4::gio;

pub fn build_entries_store(apps: &[core::AppEntry]) -> gio::ListStore {
    let store = gio::ListStore::new::<aep::AppEntryObject>();
    update_entries_store(&store, apps);
    store
}

pub fn update_entries_store(store: &gio::ListStore, apps: &[core::AppEntry]) {
    store.remove_all();
    for entry in apps {
        store.append(&aep::AppEntryObject::new(entry));
    }
}
