use enum_iterator::IntoEnumIterator;
use serde::{Deserialize, Serialize};
use yew::{html::Scope, prelude::*};

use crate::{app::App, style};

/// Families without the sentences in them.
///
/// This is used for selection in [`/`][crate::Route::Home]
/// and with [`GameQuery`][crate::game::GameQuery].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoEnumIterator, Serialize, Deserialize)]
pub enum Family {
    /// Ustensils used by a chef when cooking.
    ChiefKit = 0b0000_0001,
    /// Other fruits, like oranges.
    Fruits = 0b0000_0010,
    /// Hygiene is important for a cook.
    Hygiene = 0b0000_0100,
    /// Flipping pancakes is a professional gesture.
    ProfessionalGestures = 0b0000_1000,
    /// Red fruits, like strawberries.
    RedFruits = 0b0001_0000,
    /// Small tools used in cooking that are not part
    /// of a chief's kit.
    SmallUstensils = 0b0010_0000,
    /// Trimmings, like cutting cucumbers.
    Trimmings = 0b0100_0000,
}

impl Family {
    /// Render the family's button, adapting to whether it is selected or not.
    pub fn render(&self, link: &Scope<App>, selected: bool) -> Html {
        let f = *self;
        let onclick = link.callback(move |_| <App as Component>::Message::Toggle(f));

        html! {
            <button {onclick} class={style::button_select_family(selected)} style={self.button_style(selected)}>
                { self.to_string() }
            </button>
        }
    }

    /// Color of the family's button.
    pub fn button_style(&self, selected: bool) -> String {
        let color = match self {
            Self::ChiefKit => "purple",
            Self::Fruits => "orange",
            Self::Hygiene => "blue",
            Self::ProfessionalGestures => "black",
            Self::RedFruits => "red",
            Self::SmallUstensils => "gray",
            Self::Trimmings => "darkgreen",
        };

        if selected {
            format!("background-color:{color};")
        } else {
            format!("border-color:{color};border-style:solid;")
        }
    }
}

impl TryFrom<u8> for Family {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        for f in Self::into_enum_iter() {
            if value == f as u8 {
                return Ok(f);
            }
        }

        Err(())
    }
}

impl std::fmt::Display for Family {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChiefKit => f.write_str("Mallette"),
            Self::Fruits => f.write_str("Fruits"),
            Self::Hygiene => f.write_str("Hygiène"),
            Self::ProfessionalGestures => f.write_str("Gestes Professionnels"),
            Self::RedFruits => f.write_str("Fruits Rouges"),
            Self::SmallUstensils => f.write_str("Petit Matériel"),
            Self::Trimmings => f.write_str("Taillages"),
        }
    }
}