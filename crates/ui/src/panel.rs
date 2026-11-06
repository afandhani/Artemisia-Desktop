use std::rc::Rc;

use artemisia_core::{ShellPlan, Slot, StatusItem, Widget};

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, CenterBox, Orientation, Widget as GtkWidget};

fn status_items(items: &[StatusItem]) -> Vec<crate::status::Item> {
    items
        .iter()
        .map(|item| match item {
            StatusItem::Network => crate::status::Item::Network,
            StatusItem::Volume => crate::status::Item::Volume,
            StatusItem::Battery => crate::status::Item::Battery,
        })
        .collect()
}

pub fn build(plan: &ShellPlan, on_status_click: Rc<dyn Fn()>) -> CenterBox {
    let start = GtkBox::new(Orientation::Horizontal, 8);
    let center = GtkBox::new(Orientation::Horizontal, 8);
    let end = GtkBox::new(Orientation::Horizontal, 8);

    for placement in &plan.modules {
        let widget: GtkWidget = match placement.widget {
            Widget::Launcher => crate::launcher::button().upcast(),

            Widget::Tags => crate::tags::widget(mango::watch(), mango::view_tag).upcast(),

            Widget::Clock => crate::clock::widget_with_format(plan.clock_format.clone()).upcast(),

            Widget::Status => {
                let items = status_items(&plan.status_items);
                let callback = on_status_click.clone();

                crate::status::widget(&items, move || callback()).upcast()
            }
        };

        let target = match placement.slot {
            Slot::Start => &start,
            Slot::Center => &center,
            Slot::End => &end,
        };

        target.append(&widget);
    }

    let panel = CenterBox::new();
    panel.add_css_class("bar-inner");

    panel.set_start_widget(Some(&start));
    panel.set_center_widget(Some(&center));
    panel.set_end_widget(Some(&end));

    panel
}
