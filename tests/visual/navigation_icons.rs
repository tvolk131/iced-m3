use super::harness::Harness;
use crate::{Element, NavigationItem, Theme, navigation_rail};
use iced::{Size, widget};

fn view(selected: u8, modal: bool) -> Element<'static, u8> {
    let rail = navigation_rail(
        [
            NavigationItem::new(0, "Home", widget::text("home outline"))
                .selected_icon(widget::text("home filled")),
            NavigationItem::new(1, "Inbox", widget::text("inbox outline"))
                .selected_icon(widget::text("inbox filled")),
        ],
        Some(selected),
    )
    .on_select(|value| value);
    if modal {
        crate::modal_navigation_rail(widget::space(), rail, true, 2)
    } else {
        rail.into()
    }
}

#[test]
fn navigation_icon_overrides_follow_controlled_selection_after_rebuild() {
    for modal in [false, true] {
        let mut h = Harness::new(view(0, modal), Size::new(600.0, 400.0), Theme::light());
        h.at(1000);
        h.find("home filled");
        h.find("inbox outline");
        h.click("Inbox");
        assert_eq!(h.messages, [1]);
        // A click reports the value; only rebuilding with app-owned selection
        // changes the icon. Reuse the runtime's persistent widget tree.
        h.find("home filled");
        h.find("inbox outline");
        h.rebuild(view(1, modal));
        h.find("home outline");
        h.find("inbox filled");
        h.click("Home");
        assert_eq!(h.messages, [1, 0]);
        h.rebuild(view(0, modal));
        h.find("home filled");
        h.find("inbox outline");
    }
}
