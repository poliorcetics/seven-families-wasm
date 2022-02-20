use stylist::{css, StyleSource};

pub fn button_select_all(background_color: &'static str) -> StyleSource<'static> {
    css!(
        background-color: ${background_color};
    )
}

pub fn button_select_family(selected: bool) -> StyleSource<'static> {
    let bc = if selected { "#008CBA" } else { "#E7E7E7" };
    let fc = if selected { "white" } else { "black" };

    css!(
        background-color: ${bc};
        color: ${fc};
    )
}
