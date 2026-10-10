//! Explicit keyboard events: a key-down does not imply text entry or a key-up.
use super::harness::Harness;
use iced::{Event, keyboard};

pub struct KeyEvent {
    key: keyboard::Key,
    modifiers: keyboard::Modifiers,
    text: Option<String>,
}

impl<Message> Harness<'_, Message> {
    /// Dispatch a complete keystroke without advancing the test clock.
    pub fn tap_key(&mut self, key: KeyEvent) {
        self.event(key.press());
        self.event(key.release());
    }
}

/// Defaults to an unmodified, non-repeating key with no text payload.
pub fn key(key: impl Into<keyboard::Key>) -> KeyEvent {
    KeyEvent {
        key: key.into(),
        modifiers: keyboard::Modifiers::empty(),
        text: None,
    }
}

impl KeyEvent {
    pub fn modifiers(mut self, modifiers: keyboard::Modifiers) -> Self {
        self.modifiers = modifiers;
        self
    }

    /// Text is independent of the logical key (e.g. typeahead vs text entry).
    pub fn text(mut self, text: &str) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn press(&self) -> Event {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: self.key.clone(),
            modified_key: self.key.clone(),
            physical_key: keyboard::key::Physical::Unidentified(
                keyboard::key::NativeCode::Unidentified,
            ),
            location: keyboard::Location::Standard,
            modifiers: self.modifiers,
            text: self.text.as_deref().map(Into::into),
            repeat: false,
        })
    }

    pub fn release(&self) -> Event {
        Event::Keyboard(keyboard::Event::KeyReleased {
            key: self.key.clone(),
            modified_key: self.key.clone(),
            physical_key: keyboard::key::Physical::Unidentified(
                keyboard::key::NativeCode::Unidentified,
            ),
            location: keyboard::Location::Standard,
            modifiers: self.modifiers,
        })
    }
}
