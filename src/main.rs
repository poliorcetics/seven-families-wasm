//! TODO:
//!
//! - [ ] Start screen
//!    - [x] Choose at least one family, at most all of them
//!    - [ ] Show family logos and colors
//!    - [x] Button to (un)select all families
//!    - [x] Button to start game becomes visible after one family has been chosen
//! - [x] Game itself
//!    - [x] Button to get permission to play sound
//!    - [x] Pause button
//!    - [x] Sentences are given in random order, each N units (seconds)
//!    - [x] Show nice buttons
//! - [x] Pause screen
//!    - [x] Go back to selecting families
//!    - [x] Resume game
//!    - [x] Choose timer duration, either when game is paused or not started
//!    - [x] Show nice buttons
//! - [x] Finishing the game
//!    - [x] All sentences have been said
//!    - [x] Go back to selecting families
//!    - [x] Show nice buttons
mod audio;
mod family;
mod game;
mod sentences;
mod style;
mod timer;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub(crate) const IS_FOR_GH_PAGES: bool = option_env!("IS_FOR_GH_PAGES").is_some();

fn main() {
    yew::start_app::<game::Game>();
}
