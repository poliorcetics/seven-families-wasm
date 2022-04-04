//! Sentences for the game.
use std::collections::HashSet;

use enum_iterator::IntoEnumIterator;
use rand::seq::SliceRandom;

use crate::family::Family;

/// Sentences for a game.
///
/// Shuffled once on creation and never again.
#[derive(Debug)]
pub struct Sentences(Vec<Sentence>);

impl Sentences {
    /// Build a new set of sentences from the selected families.
    ///
    /// Initially, all the possible sentences are available in random order
    /// and they are popped by [`Self::draw_one()`].
    pub fn new(families: &HashSet<Family>) -> Self {
        // There are 6 elements per family
        let mut sentences = Vec::with_capacity(families.len() * 6);
        for family in families {
            match family {
                Family::ChiefKit => {
                    sentences.extend(ChiefKit::into_enum_iter().map(Sentence::ChiefKit))
                }
                Family::Fruits => sentences.extend(Fruits::into_enum_iter().map(Sentence::Fruits)),
                Family::Hygiene => {
                    sentences.extend(Hygiene::into_enum_iter().map(Sentence::Hygiene))
                }
                Family::ProfessionalGestures => sentences.extend(
                    ProfessionalGestures::into_enum_iter().map(Sentence::ProfessionalGestures),
                ),
                Family::RedFruits => {
                    sentences.extend(RedFruits::into_enum_iter().map(Sentence::RedFruits))
                }
                Family::SmallUstensils => {
                    sentences.extend(SmallUstensils::into_enum_iter().map(Sentence::SmallUstensils))
                }
                Family::Trimmings => {
                    sentences.extend(Trimmings::into_enum_iter().map(Sentence::Trimmings))
                }
            }
        }

        sentences.shuffle(&mut rand::rngs::OsRng);
        Self(sentences)
    }

    /// Draw one sentence from the list.
    pub fn draw_one(&mut self) -> Option<Sentence> {
        self.0.pop()
    }

    /// `true` if there are no more sentences.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// All the possible sentences.
#[derive(Debug, Clone, Copy)]
pub enum Sentence {
    /// Ustensils used by a chef when cooking.
    ChiefKit(ChiefKit),
    /// Other fruits, like oranges.
    Fruits(Fruits),
    /// Hygiene is important for a cook.
    Hygiene(Hygiene),
    /// Flipping pancakes is a professional gesture.
    ProfessionalGestures(ProfessionalGestures),
    /// Red fruits, like strawberries.
    RedFruits(RedFruits),
    /// Small tools used in cooking that are not part
    /// of a chief's kit.
    SmallUstensils(SmallUstensils),
    /// Trimmings, like cutting cucumbers.
    Trimmings(Trimmings),
}

impl Sentence {
    /// Sound file for the whole family.
    pub fn family_sound_file(&self) -> &'static str {
        match self {
            Sentence::ChiefKit(st) => st.family_sound_file(),
            Sentence::Fruits(st) => st.family_sound_file(),
            Sentence::Hygiene(st) => st.family_sound_file(),
            Sentence::ProfessionalGestures(st) => st.family_sound_file(),
            Sentence::RedFruits(st) => st.family_sound_file(),
            Sentence::SmallUstensils(st) => st.family_sound_file(),
            Sentence::Trimmings(st) => st.family_sound_file(),
        }
    }

    /// Sound file for the specific element.
    pub fn element_sound_file(&self) -> &'static str {
        match self {
            Sentence::ChiefKit(st) => st.element_sound_file(),
            Sentence::Fruits(st) => st.element_sound_file(),
            Sentence::Hygiene(st) => st.element_sound_file(),
            Sentence::ProfessionalGestures(st) => st.element_sound_file(),
            Sentence::RedFruits(st) => st.element_sound_file(),
            Sentence::SmallUstensils(st) => st.element_sound_file(),
            Sentence::Trimmings(st) => st.element_sound_file(),
        }
    }
}

/// Generate a `sound_file` method on `$name`.
///
/// The file for `$name::$variant` is `assets / $folder / $file .mp3`.
macro_rules! assets {
    ($(#[$meta:meta])* $name:ident: $folder:literal; $($(#[$variant_meta:meta])* $variant:ident: $file:literal),+ $(,)?) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, IntoEnumIterator)]
        pub enum $name {
            $(
                $(#[$variant_meta])*
                $variant,
            )+
        }

        impl $name {
            /// Path to the sound file for the family, absolute from the root
            /// of the website.
            const fn family_sound_file(&self) -> &'static str {
                // Ensure the file exists.
                const _: &[u8] = include_bytes!(concat!("../assets/", $folder, "/0-famille.mp3")).as_slice();
                // Relative to the root of the website.
                // Adapted to github pages.
                if crate::IS_FOR_GH_PAGES {
                    concat!("/seven-families-wasm/assets/", $folder, "/0-famille.mp3")
                } else {
                    concat!("/assets/", $folder, "/0-famille.mp3")
                }
            }

            /// Path to the sound file for the sentence, absolute from the root of
            /// the website.
            const fn element_sound_file(&self) -> &'static str {
                // Ensure the file exists.
                $( const _: &[u8] = include_bytes!(concat!("../assets/", $folder, "/", $file, ".mp3")).as_slice(); )+
                match self {
                    // Relative to the root of the website.
                    // Adapted to github pages.
                    $(
                        Self::$variant => if crate::IS_FOR_GH_PAGES {
                            concat!("/seven-families-wasm/assets/", $folder, "/", $file, ".mp3")
                        } else {
                            concat!("/assets/", $folder, "/", $file, ".mp3")
                        },
                    )+
                }
            }
        }
    };
}

assets! {
    /// Malette
    ChiefKit: "mallette";
    /// Canneleur
    Coring: "canneleur",
    /// Filet de sole
    FilletKnife: "filet-de-sole",
    /// Couteau d'office
    ParingKnife: "couteau-d-office",
    /// Économe
    Peeler: "econome",
    /// Éminceur
    Slicer: "eminceur",
    /// Zesteur,
    Zester: "zesteur",
}

assets! {
    /// Fruits
    Fruits: "fruits";
    /// Pomme
    Apple: "pomme",
    /// Abricot
    Apricot: "abricot",
    /// Raisin
    Grapes: "raisin",
    /// Orange
    Orange: "orange",
    /// Pêche
    Peach: "peche",
    /// Prune
    Plum: "prune",
}

assets! {
    /// Hygiène
    Hygiene: "hygiene";
    /// Bactérie
    Bacterium: "bacterie",
    /// Nettoyage
    Cleaning: "nettoyage",
    /// Désinfectant
    Disinfectant: "desinfectant",
    /// EPI
    Epi: "epi",
    /// Microbe
    Microbe: "microbe",
    /// Moisissure
    Mould: "moisissure",
}

assets! {
    /// Gestes professionnels
    ProfessionalGestures: "gestes-professionnels";
    /// Escalopper
    Cutletting: "escalopper",
    /// Abaisser
    Lower: "abaisser",
    /// Émincer
    Slice: "emincer",
    /// Suer
    Sweat: "suer",
    /// Tourner
    Turn: "tourner",
    /// Vanner
    Winnow: "vanner",
}

assets! {
    /// Fruits rouges
    RedFruits: "fruits-rouges";
    /// Mûre
    Blackberry: "mure",
    /// Cassis
    Blackcurrant: "cassis",
    /// Cerise
    Cherry: "cerise",
    /// Framboise
    Raspberry: "framboise",
    /// Groseille
    Redcurrant: "groseille",
    /// Fraise
    Strawberry: "fraise",
}

assets! {
    /// Petit matériel
    SmallUstensils: "petit-materiel";
    /// Bahut
    Chests: "bahut",
    /// Cul de poule
    ChickenButt: "cul-de-poule",
    /// Chinois étamine
    ChineseCheesecloth: "chinois-etamine",
    /// Plaque à débarasser
    CleaningPlate: "plaque-a-debarasser",
    /// Rondeau
    Roundel: "rondeau",
    /// Écumoire
    Skimmer: "ecumoire",
}

// Funny how many words are directly taken from French here#[derive(Debug, Clone, Copy, IntoEnumIterator)]
assets! {
    /// Taillages
    Trimmings: "taillages";
    /// Brunoise
    Brunoise: "brunoise",
    /// Jardinière
    Jardiniere: "jardiniere",
    /// Julienne
    JulienneStrip: "julienne",
    /// Macédoine
    Macedonia: "macedoine",
    /// Mirepoix
    Mirepoix: "mirepoix",
    /// Paysanne
    PaysanneCut: "paysanne",
}
