//! Main page ('/')
//!
//! Allows selecting families before launching the game.
//! Prevents launching the game if no family is selected.
use std::collections::HashSet;

use enum_iterator::IntoEnumIterator;
use yew::html::Scope;
use yew::prelude::*;
use yew_router::prelude::*;
use yew_router::scope_ext::HistoryHandle;

use crate::{family::Family, game::GameQuery, Route, style};

/// See [`module level docs`][self].
pub struct App {
    /// The selected families for the upcoming game.
    families: HashSet<Family>,
    /// Needs to be kept alive to be able to push to history.
    _history: HistoryHandle,
}

pub enum Msg {
    /// Toggle selection for a family.
    Toggle(Family),
    /// Select all families.
    SelectAllFamilies,
    /// Deselect all families.
    ClearAllFamilies,
    /// Launch the game with the selected families.
    LaunchGame,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let history = ctx
            .link()
            .add_history_listener(ctx.link().callback(|_| Msg::LaunchGame))
            .unwrap();

        Self {
            families: Default::default(),
            _history: history,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Toggle(f) => {
                if self.families.contains(&f) {
                    self.families.remove(&f);
                } else {
                    self.families.insert(f);
                }

                true
            }
            Msg::SelectAllFamilies => {
                self.families.extend(Family::into_enum_iter());
                true
            }
            Msg::ClearAllFamilies => {
                self.families.clear();
                true
            }
            Msg::LaunchGame => true,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        html! {
            <div>
                <button onclick={ link.callback(|_| Msg::SelectAllFamilies) } class={style::button_select_all("#4CAF50")}>
                    { "Sélectionner toutes les familles" }
                </button>
                <button onclick={ link.callback(|_| Msg::ClearAllFamilies) } class={style::button_select_all("#F44336")}>
                    { "Tout déselectionner" }
                </button>
                <hr />
                { self.family_view(link) }
                { self.start_button(link) }
            </div>
        }
    }
}

impl App {
    /// Make all the families available for selection/deselection.
    fn family_view(&self, link: &Scope<Self>) -> Html {
        html! {
            <div>
                { for Family::into_enum_iter().map(|f| f.render(link, self.families.contains(&f))) }
            </div>
        }
    }

    /// The start button is only shown if at least one family has been selected
    /// to play.
    fn start_button(&self, link: &Scope<Self>) -> Html {
        if !self.families.is_empty() {
            let query: GameQuery = (&self.families).into();
            let history = link.history().unwrap();
            let onclick = Callback::once(move |_| {
                history.push_with_query(Route::StartGame, query).unwrap();
            });

            html! {
                <div>
                    <hr />
                    <button {onclick}>
                        { "Jouer" }
                    </button>
                </div>
            }
        } else {
            html! {}
        }
    }
}
