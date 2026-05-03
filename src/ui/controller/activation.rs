use crate::core;
use crate::ui::model::app_entry_object as aep;
use crate::ui::controller::selection::selected_or_first;
use gtk4::prelude::*;

pub fn activate_position(
    selection: &gtk4::SingleSelection,
    state: &core::SharedState,
    window: &gtk4::ApplicationWindow,
    position: u32,
) {
    let obj = selection
        .model()
        .and_then(|m| m.item(position))
        .and_downcast::<aep::AppEntryObject>();

    let launch_data = obj.and_then(|obj| {
        let s = state.lock();
        s.apps
            .iter()
            .find(|app| app.id == obj.id())
            .map(|app| (app.id.clone(), app.exec.clone()))
    });

    if let Some((id, exec)) = launch_data {
        if core::desktop::launch_exec(&exec).is_ok() {
            let freq_to_save = {
                let mut s = state.lock();
                s.record_launch(&id);
                s.freq.clone()
            };

            if let Err(err) = core::freq_store::save_freq(&freq_to_save) {
                eprintln!("Failed to persist launch frequency: {err}");
            }

            window.hide();
        }
    }
}

pub fn activate_selected(
    selection: &gtk4::SingleSelection,
    state: &core::SharedState,
    window: &gtk4::ApplicationWindow,
) {
    if let Some(position) = selected_or_first(selection) {
        activate_position(selection, state, window, position);
    }
}
