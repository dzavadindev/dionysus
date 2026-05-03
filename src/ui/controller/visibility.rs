use crate::ui::controller::selection::selected_or_first;
use crate::ui::handle::UiHandle;
use gtk4::prelude::*;

pub fn prepare_for_show(ui: &UiHandle) {
    ui.main_window.present();

    if ui.selection_model.n_items() > 0 {
        ui.selection_model
            .set_selected(selected_or_first(&ui.selection_model).unwrap_or(0));
    }

    ui.prompt.grab_focus();
}
