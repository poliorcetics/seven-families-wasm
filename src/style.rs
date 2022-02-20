use stylist::{css, StyleSource};

/// Color to use for the "(un)select all families" buttons.
pub fn button_select_all(background_color: &'static str) -> StyleSource<'static> {
    css!(
        background-color: ${background_color};
    )
}

/// Class to use for the family selection buttons, depends on the selection state.
pub fn button_select_family(selected: bool) -> &'static str {
    if selected {
        "family_selected"
    } else {
        "family_not_selected"
    }
}
