#![cfg(feature = "material-symbols")]

use iced::advanced::svg::{Data, Handle};
use iced_m3::material_symbols::{Fill, IconName, OpticalSize, Style, Symbol};
use iced_m3::{Element, NavigationItem, navigation_rail};

#[test]
fn official_symbols_convert_through_the_public_api_without_font_or_file_setup() {
    let symbol = Symbol::new(IconName::Wifi)
        .style(Style::Rounded)
        .optical_size(OpticalSize::Size20);
    let handle = Handle::from(symbol);
    let Data::Bytes(bytes) = handle.data() else {
        panic!("Official artwork must be embedded");
    };
    assert!(
        std::str::from_utf8(bytes.as_ref())
            .unwrap()
            .contains("width=\"20\"")
    );
    let _: Element<'static, ()> = navigation_rail(
        [NavigationItem::new(0, "Wi-Fi", symbol).selected_icon(symbol.fill(Fill::Filled))],
        Some(0),
    )
    .into();
    assert_eq!(IconName::ALL.len(), 4157);
}
