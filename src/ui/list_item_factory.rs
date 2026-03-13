use crate::ui::app_entry_object as aep;
use gtk4;
use gtk4::prelude::*;

pub fn build_factory() -> gtk4::SignalListItemFactory {
    let factory = gtk4::SignalListItemFactory::new();

    factory.connect_setup(|_, list_item| {
        let row_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        row_box.set_margin_top(8);
        row_box.set_margin_bottom(8);
        row_box.set_margin_start(12);
        row_box.set_margin_end(12);

        let label = gtk4::Label::new(None);
        label.set_xalign(0.0);

        row_box.append(&label);

        list_item.set_child(Some(&row_box));
    });

    // Bind: Connect one AppEntryObject to one row widget
    factory.connect_bind(|_, list_item| {
        // Get the model item and downcast it to AppEntryObject
        let item = list_item
            .item()
            .and_downcast::<aep::AppEntryObject>()
            .expect("The item should be an AppEntryObject");

        // Get the row widget we created in setup()
        let row_box = list_item
            .child()
            .and_downcast::<gtk4::Box>()
            .expect("The child should be a gtk4::Box");

        // Get the label inside the row box
        let label = row_box
            .first_child()
            .and_downcast::<gtk4::Label>()
            .expect("The box should contain a gtk4::Label");

        // Fill the row from the item
        label.set_text(&item.name());
    });

    factory
}
