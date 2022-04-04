use std::borrow::Cow;

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
                <img
                    src={ self.logo_file() }
                    alt={ format!("Logo de la famille {}", self) }
                    class={ "family_not_selected" }
                    style={ self.logo_style(selected).to_string() }
                />
                { self.to_string() }
            </button>
        }
    }

    /// Color of the family's button.
    pub fn button_style(&self, selected: bool) -> String {
        let color = self.color();

        if selected {
            format!("background-color:{color};border-color:{color};border-style:solid;")
        } else {
            format!("border-color:{color};border-style:solid;")
        }
    }

    /// Style for logos.
    pub fn logo_style(&self, selected: bool) -> Cow<'static, str> {
        const STYLE: &str = "align:center;max-width:50px;max-height:50px;margin-right:1%;";
        if selected {
            let color = self.color();
            Cow::Owned(format!("{STYLE}background-color:{color};"))
        } else {
            Cow::Borrowed(STYLE)
        }
    }

    /// Color associated with the family.
    pub fn color(&self) -> &'static str {
        match self {
            Self::ChiefKit => "purple",
            Self::Fruits => "orange",
            Self::Hygiene => "blue",
            Self::ProfessionalGestures => "black",
            Self::RedFruits => "red",
            Self::SmallUstensils => "gray",
            Self::Trimmings => "darkgreen",
        }
    }

    /// Path to logo file.
    pub fn logo_file(&self) -> &'static str {
        macro_rules! logo_image_file {
            ($folder:literal) => {{
                // Check for file existence at compile-time
                const _: &[u8] =
                    include_bytes!(concat!("../assets/", $folder, "/0-logo.png")).as_slice();
                // Adapt file path after checking if we're running on github pages or no
                if crate::IS_FOR_GH_PAGES {
                    concat!("/seven-families-wasm/assets/", $folder, "/0-logo.png")
                } else {
                    concat!("/assets/", $folder, "/0-logo.png")
                }
            }};
        }

        match self {
            Self::ChiefKit => logo_image_file!("mallette"),
            Self::Fruits => logo_image_file!("fruits"),
            Self::Hygiene => logo_image_file!("hygiene"),
            Self::ProfessionalGestures => logo_image_file!("gestes-professionnels"),
            Self::RedFruits => logo_image_file!("fruits-rouges"),
            Self::SmallUstensils => logo_image_file!("petit-materiel"),
            Self::Trimmings => logo_image_file!("taillages"),
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
