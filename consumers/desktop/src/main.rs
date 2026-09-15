use material_desktop_consumer::Studio;

fn main() -> iced::Result {
    iced::application(Studio::default, Studio::update, Studio::view)
        .title("Northstar Studio")
        .theme(Studio::theme)
        .default_font(iced_m3::fonts::REGULAR)
        .window_size((1000.0, 800.0))
        .run()
}
