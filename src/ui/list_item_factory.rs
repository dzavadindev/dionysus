use crate::ui::app_entry_object as aep;
use crate::ui::list_item::AppListItemWidget;
use gtk4;
use gtk4::prelude::*;

pub fn build_factory() -> gtk4::SignalListItemFactory {
    let factory = gtk4::SignalListItemFactory::new();

    // The structure
    factory.connect_setup(|_, list_item| {
        let row = AppListItemWidget::new();
        list_item.set_child(Some(&row));
    });

    // The content
    factory.connect_bind(|_, list_item| {
        // Get the model item and downcast it to AppEntryObject
        let item = list_item
            .item()
            .and_downcast::<aep::AppEntryObject>()
            .expect("The item should be an AppEntryObject");

        // Get the row widget we created in setup()
        let row = list_item
            .child()
            .and_downcast::<AppListItemWidget>()
            .expect("The child should be an AppListItemWidget");

        row.bind(&item);
    });

    factory.connect_unbind(|_, list_item| {
        if let Some(row) = list_item.child().and_downcast::<AppListItemWidget>() {
            row.clear();
        }
    });

    factory
}
