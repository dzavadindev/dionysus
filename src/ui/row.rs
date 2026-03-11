use gtk4;
use gtk4::prelude::*;

use crate::core;

pub fn build_row(entry: core::AppEntry) -> gtk4::ListBoxRow {
    let row = gtk4::ListBoxRow::new();

    let outer = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    outer.set_margin_top(8);
    outer.set_margin_bottom(8);
    outer.set_margin_start(12);
    outer.set_margin_end(12);

    let label = gtk4::Label::new(Some(&entry.name));
    label.set_xalign(0.0);

    outer.append(&label);
    row.set_child(Some(&outer));

    row
}
