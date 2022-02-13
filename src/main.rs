//! TODO: Whole game
//!
//! More precisely:
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
//!    - [ ] Show nice buttons
//! - [ ] Pause screen
//!    - [x] Go back to selecting families
//!    - [x] Resume game
//!    - [ ] Choose timer duration, either when game is paused or not started
//!    - [ ] Show nice buttons
//! - [x] Finishing the game
//!    - [x] All sentences have been said
//!    - [x] Go back to selecting families
//!    - [ ] Show nice buttons
use yew::prelude::*;
use yew_router::prelude::*;

mod app;
mod family;
mod game;
mod sentences;
mod timer;

fn main() {
    yew::start_app::<Main>();
}

#[function_component(Main)]
fn launcher() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={ Switch::render(Route::switch) } />
        </BrowserRouter>
    }
}

#[derive(Routable, PartialEq, Clone)]
enum Route {
    #[at("/")]
    Home,
    #[at("/game")]
    StartGame,
}

impl Route {
    fn switch(&self) -> Html {
        match self {
            Route::StartGame => html! { <game::Game /> },
            _ => html! { <app::App /> },
        }
    }
}
