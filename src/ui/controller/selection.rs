use gtk4::prelude::*;

pub fn selected_or_first(selection: &gtk4::SingleSelection) -> Option<u32> {
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

pub fn select_prev(selection: &gtk4::SingleSelection) {
    if let Some(current) = selected_or_first(selection) {
        selection.set_selected(current.saturating_sub(1));
    }
}

pub fn select_next(selection: &gtk4::SingleSelection) {
    let count = selection.n_items();
    if let Some(current) = selected_or_first(selection) {
        selection.set_selected((current + 1).min(count - 1));
    }
}

pub fn focus_list_for_navigation(selection: &gtk4::SingleSelection, list_view: &gtk4::ListView) {
    if let Some(position) = selected_or_first(selection) {
        selection.set_selected(position);
    }
    list_view.grab_focus();
}
