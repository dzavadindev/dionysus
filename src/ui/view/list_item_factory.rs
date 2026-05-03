use crate::ui::model::app_entry_object as aep;
use crate::ui::view::list_item::AppListItemWidget;
use gtk4;
use gtk4::prelude::*;

pub fn build_factory() -> gtk4::SignalListItemFactory {
    let factory = gtk4::SignalListItemFactory::new();

    factory.connect_setup(|_, list_item| {
        let row = AppListItemWidget::new();
        list_item.set_child(Some(&row));
    });

    factory.connect_bind(|_, list_item| {
        let item = list_item
            .item()
            .and_downcast::<aep::AppEntryObject>()
            .expect("The item should be an AppEntryObject");

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
