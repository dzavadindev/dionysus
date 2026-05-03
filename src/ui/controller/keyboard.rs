use gtk4::prelude::*;

pub fn route_text_key_to_prompt(
    prompt: &gtk4::Entry,
    key: gtk4::gdk::Key,
    state: gtk4::gdk::ModifierType,
) -> bool {
    if state.intersects(
        gtk4::gdk::ModifierType::CONTROL_MASK
            | gtk4::gdk::ModifierType::ALT_MASK
            | gtk4::gdk::ModifierType::META_MASK,
    ) {
        return false;
    }

    let ch = match key.to_unicode() {
        Some(ch) => ch,
        None => return false,
    };

    if ch.is_control() {
        return false;
    }

    prompt.grab_focus();
    let mut pos = prompt.position();
    let text = ch.to_string();
    prompt.insert_text(&text, &mut pos);
    prompt.set_position(pos);
    true
}

pub fn route_backspace_to_prompt(prompt: &gtk4::Entry, key: gtk4::gdk::Key) -> bool {
    if key != gtk4::gdk::Key::BackSpace {
        return false;
    }

    prompt.grab_focus();
    let pos = prompt.position();
    if pos > 0 {
        prompt.delete_text(pos - 1, pos);
        prompt.set_position(pos - 1);
    }
    true
}

pub fn route_delete_to_prompt(prompt: &gtk4::Entry, key: gtk4::gdk::Key) -> bool {
    if key != gtk4::gdk::Key::Delete {
        return false;
    }

    prompt.grab_focus();
    let pos = prompt.position();
    prompt.delete_text(pos, pos + 1);
    prompt.set_position(pos);
    true
}
