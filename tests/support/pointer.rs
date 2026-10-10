use iced::{Event, mouse, window};
use std::time::Instant;

pub fn press() -> Event {
    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
}
pub fn release() -> Event {
    Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
}
pub fn redraw(now: Instant) -> Event {
    Event::Window(window::Event::RedrawRequested(now))
}
