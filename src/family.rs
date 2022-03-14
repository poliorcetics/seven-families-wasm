use enum_iterator::IntoEnumIterator;
use yew::html::Scope;
use yew::prelude::*;

use crate::game::{BeforeGameMsg, Game};
use crate::style;

/// Families without the sentences in them.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, IntoEnumIterator)]
pub enum Family {
    /// Ustensils used by a chef when cooking.
    ChiefKit,
    /// Other fruits, like oranges.
    Fruits,
    /// Hygiene is important for a cook.
    Hygiene,
    /// Flipping pancakes is a professional gesture.
    ProfessionalGestures,
    /// Red fruits, like strawberries.
    RedFruits,
    /// Small tools used in cooking that are not part
    /// of a chief's kit.
    SmallUstensils,
    /// Trimmings, like cutting cucumbers.
    Trimmings,
}

impl Family {
    /// Render the family's button, adapting to whether it is selected or not.
    pub fn render(&self, link: &Scope<Game>, selected: bool) -> Html {
        let f = *self;
        let onclick = link.callback(move |_| BeforeGameMsg::Toggle(f));

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
