//! Sentences
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
    pub fn new(families: HashSet<Family>) -> Self {
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

    pub fn draw_one(&mut self) -> Option<Sentence> {
        self.0.pop()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

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

    pub fn sentence_sound_file(&self) -> &'static str {
        match self {
            Sentence::ChiefKit(st) => st.sentence_sound_file(),
            Sentence::Fruits(st) => st.sentence_sound_file(),
            Sentence::Hygiene(st) => st.sentence_sound_file(),
            Sentence::ProfessionalGestures(st) => st.sentence_sound_file(),
            Sentence::RedFruits(st) => st.sentence_sound_file(),
            Sentence::SmallUstensils(st) => st.sentence_sound_file(),
            Sentence::Trimmings(st) => st.sentence_sound_file(),
        }
    }
}

/// Generate a `sound_file` method on `$name`.
///
/// The file for `$name::$variant` is `assets / $folder / $file .mp3`.
macro_rules! assets {
    ($name:ty: $folder:expr; $($variant:ident: $file:expr),+ $(,)?) => {
        impl $name {
            /// Path to the sound file for the family, absolute from the root
            /// of the website.
            fn family_sound_file(&self) -> &'static str {
                concat!("/assets/", $folder, "/", "0-famille.mp3")
            }

            /// Path to the sound file for the sentence, absolute from the root of
            /// the website.
            fn sentence_sound_file(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant => concat!("/assets/", $folder, "/", $file, ".mp3"),
                    )+
                }
            }
        }
    };
}

/// Malette
#[derive(Debug, Clone, Copy, IntoEnumIterator)]
pub enum ChiefKit {
    /// Canneleur
    Coring,
    /// Filet de sole
    FilletKnife,
    /// Couteau d'office
    ParingKnife,
    /// Économe
    Peeler,
    /// Éminceur
    Slicer,
    /// Zesteur,
    Zester,
}

assets! {
    ChiefKit: "mallette";
    Coring: "canneleur",
    FilletKnife: "filet-de-sole",
    ParingKnife: "couteau-d-office",
    Peeler: "econome",
    Slicer: "eminceur",
    Zester: "zesteur",
}

/// Fruits
#[derive(Debug, Clone, Copy, IntoEnumIterator)]
pub enum Fruits {
    /// Pomme
    Apple,
    /// Abricot
    Apricot,
    /// Raisin
    Grapes,
    /// Orange
    Orange,
    /// Pêche
    Peach,
    /// Prune
    Plum,
}

assets! {
    Fruits: "fruits";
    Apple: "pomme",
    Apricot: "abricot",
    Grapes: "raisin",
    Orange: "orange",
    Peach: "peche",
    Plum: "prune",
}

/// Hygiène
#[derive(Debug, Clone, Copy, IntoEnumIterator)]
pub enum Hygiene {
    /// Bactérie
    Bacterium,
    /// Nettoyage
    Cleaning,
    /// Désinfectant
    Disinfectant,
    /// EPI
    Epi,
    /// Microbe
    Microbe,
    /// Moisissure
    Mould,
}

assets! {
    Hygiene: "hygiene";
    Bacterium: "bacterie",
    Cleaning: "nettoyage",
    Disinfectant: "desinfectant",
    Epi: "epi",
    Microbe: "microbe",
    Mould: "moisissure",
}

/// Gestes professionnels
#[derive(Debug, Clone, Copy, IntoEnumIterator)]
pub enum ProfessionalGestures {
    /// Escalopper
    Cutletting,
    /// Abaisser
    Lower,
    /// Émincer
    Slice,
    /// Suer
    Sweat,
    /// Tourner
    Turn,
    /// Vanner
    Winnow,
}

assets! {
    ProfessionalGestures: "gestes-professionnels";
    Cutletting: "escalopper",
    Lower: "abaisser",
    Slice: "emincer",
    Sweat: "suer",
    Turn: "tourner",
    Winnow: "vanner",
}

/// Fruits rouges
#[derive(Debug, Clone, Copy, IntoEnumIterator)]
pub enum RedFruits {
    /// Mûre
    Blackberry,
    /// Cassis
    Blackcurrant,
    /// Cerise
    Cherry,
    /// Framboise
    Raspberry,
    /// Groseille
    Redcurrant,
    /// Fraise
    Strawberry,
}

assets! {
    RedFruits: "fruits-rouges";
    Blackberry: "mure",
    Blackcurrant: "cassis",
    Cherry: "cerise",
    Raspberry: "framboise",
    Redcurrant: "groseille",
    Strawberry: "fraise",
}

/// Petit matériel
#[derive(Debug, Clone, Copy, IntoEnumIterator)]
pub enum SmallUstensils {
    /// Bahut
    Chests,
    /// Cul de poule
    ChickenButt,
    /// Chinois étamine
    ChineseCheesecloth,
    /// Plaque à débarasser
    CleaningPlate,
    /// Rondeau
    Roundel,
    /// Écumoire
    Skimmer,
}

assets! {
    SmallUstensils: "petit-materiel";
    Chests: "bahut",
    ChickenButt: "cul-de-poule",
    ChineseCheesecloth: "chinois-etamine",
    CleaningPlate: "plaque-a-debarasser",
    Roundel: "rondeau",
    Skimmer: "ecumoire",
}

// Funny how many words are directly taken from French here
/// Taillages
#[derive(Debug, Clone, Copy, IntoEnumIterator)]
pub enum Trimmings {
    /// Brunoise
    Brunoise,
    /// Jardinière
    Jardiniere,
    /// Julienne
    JulienneStrip,
    /// Macédoine
    Macedonia,
    /// Mirepoix
    Mirepoix,
    /// Paysanne
    PaysanneCut,
}

assets! {
    Trimmings: "taillages";
    Brunoise: "brunoise",
    Jardiniere: "jardiniere",
    JulienneStrip: "julienne",
    Macedonia: "macedoine",
    Mirepoix: "mirepoix",
    PaysanneCut: "paysanne",
}
