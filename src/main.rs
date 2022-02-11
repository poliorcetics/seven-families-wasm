//! TODO: Whole game
//!
//! More precisely:
//!
//! - Start screen
//!    - Choose at least one family, at most all of them
//!    - Choose a time N between sentences (in range MIN..MAX, default ??)
//!    - Show family logos and colors
//!    - Button to select all families
//!    - Button to start game becomes visible after one family has been chosen
//! - Game itself
//!    - Pause button
//!    - Listen Again button (reset timer ??)
//!    - Sentences are given in random order, each N units (seconds)
//! - Pause screen
//!    - Quit game
//!    - Resume game
//! - Finishing the game
//!    - All sentences have been said
use yew::prelude::*;
use yew_router::prelude::*;

mod app;
mod family;
mod game;
mod sentences;

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
