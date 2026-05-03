use gtk4::glib::{self, subclass::prelude::*};
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;

use crate::ui::model::app_entry_object as aep;

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    pub struct AppListItemWidget {
        pub name_label: RefCell<Option<gtk4::Label>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AppListItemWidget {
        const NAME: &'static str = "DionysusAppListItemWidget";
        type Type = super::AppListItemWidget;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for AppListItemWidget {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.set_orientation(gtk4::Orientation::Horizontal);
            obj.set_spacing(12);
            obj.set_margin_top(8);
            obj.set_margin_bottom(8);
            obj.set_margin_start(12);
            obj.set_margin_end(12);

            let label = gtk4::Label::new(None);
            label.set_xalign(0.0);
            obj.append(&label);
            self.name_label.replace(Some(label));
        }
    }

    impl WidgetImpl for AppListItemWidget {}
    impl BoxImpl for AppListItemWidget {}
}

glib::wrapper! {
    pub struct AppListItemWidget(ObjectSubclass<imp::AppListItemWidget>)
        @extends gtk4::Widget, gtk4::Box,
        @implements gtk4::Orientable, gtk4::Buildable, gtk4::Accessible, gtk4::ConstraintTarget;
}

impl AppListItemWidget {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn bind(&self, item: &aep::AppEntryObject) {
        if let Some(label) = self.imp().name_label.borrow().as_ref() {
            label.set_text(&item.name());
        }
    }

    pub fn clear(&self) {
        if let Some(label) = self.imp().name_label.borrow().as_ref() {
            label.set_text("");
        }
    }
}
