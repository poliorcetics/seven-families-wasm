//! Game page ('/game')
use std::collections::HashSet;

use enum_iterator::IntoEnumIterator;
use gloo_timers::callback::Timeout;
use serde::{Deserialize, Serialize};
use web_sys::HtmlAudioElement;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::family::Family;
use crate::sentences::{Sentence, Sentences};

#[derive(Debug)]
pub struct Game {
    sentences: Sentences,
    state: State,
}

#[derive(Debug)]
pub enum State {
    GettingSoundPermission,
    Playing {
        current: (Sentence, SentenceState),
        node: NodeRef,
        timer: Option<Timeout>,
    },
    Paused {
        // A pause can happen during a 'waiting' state, in which case there is
        // no current sentence.
        current: Option<(Sentence, SentenceState)>,
        // Since we don't keep the sound node here, it means the current
        // sound will restart from the start when 'Resume' is sent.
        //
        // I did not consider this a default because the sound are very short
        // (one to five words) and the game is made for people learning french:
        // making them remember half a word and hear the second part several
        // seconds later seems like a bad practice.
    },
    Waiting {
        node: NodeRef,
        timer: Option<Timeout>,
    },
    Finished,
}

#[derive(Debug, Clone, Copy)]
pub enum SentenceState {
    Family,
    Element,
}

pub enum GameMsg {
    SentenceState,
    NextSentence,
    SoundPermission,
    Pause,
    Resume,
}

impl Component for Game {
    type Message = GameMsg;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let location = ctx.link().location().unwrap();
        let families = location
            .query::<GameQuery>()
            .unwrap_or(GameQuery { families: 0 });
        let families = families
            .try_into()
            .unwrap_or_else(|_| HashSet::from_iter(Family::into_enum_iter()));

        Self {
            sentences: Sentences::new(families),
            state: State::GettingSoundPermission,
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        if let State::Playing { ref node, .. } = self.state {
            node.cast::<HtmlAudioElement>().and_then(|x| x.play().ok());
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match (&mut self.state, msg) {
            // State: was waiting for permission to play sound, just got it.
            (State::GettingSoundPermission, GameMsg::SoundPermission)
            // State: launch next sentence.
            | (State::Waiting { .. }, GameMsg::NextSentence)
            | (State::Playing { .. }, GameMsg::NextSentence) => {
                match self.sentences.draw_one() {
                    None => self.state = State::Finished,
                    Some(st) => {
                        self.state = State::Playing {
                            current: (st, SentenceState::Family),
                            node: Default::default(),
                            timer: Some({
                                let link = ctx.link().clone();
                                Timeout::new(
                                    10_000, /* ms */
                                    move || link.send_message(GameMsg::NextSentence),
                                )
                            }),
                        }
                    }
                }

                true
            }
            // State of the game: a sound just finished playing.
            (
                State::Playing {
                    current, node, timer
                },
                GameMsg::SentenceState,
            ) => {
                match current {
                    (st, SentenceState::Family) => *current = (*st, SentenceState::Element),
                    (_, SentenceState::Element) => {
                        self.state = State::Waiting {
                            node: node.clone(),
                            timer: timer.take(),
                        };
                    }
                }

                true
            },
            // State of game: a sound is playing
            (State::Playing { timer, node, current }, GameMsg::Pause) => {
                timer.take().map(Timeout::cancel);
                node.cast::<HtmlAudioElement>().and_then(|x| x.pause().ok());

                self.state = State::Paused {
                    current: Some(*current),
                };

                true
            },
            // State of game: waiting for timer to launch next sentence
            (State::Waiting { timer, node }, GameMsg::Pause) => {
                timer.take().map(Timeout::cancel);
                node.cast::<HtmlAudioElement>().and_then(|x| x.pause().ok());

                self.state = State::Paused {
                    current: None,
                };

                true
            },
            // State of game: resume paused game
            (State::Paused { current }, GameMsg::Resume) => {
                let timer = Some({
                    let link = ctx.link().clone();
                    Timeout::new(
                        10_000, /* ms */
                        move || link.send_message(GameMsg::NextSentence),
                    )
                });
                let  node = NodeRef::default();

                match current {
                    None => self.state = State::Waiting { timer, node },
                    Some(current) => self.state = State::Playing {
                        timer,
                        node,
                        current: *current,
                    },
                }

                true
            }
            _ => false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let state_view = match self.state {
            State::GettingSoundPermission => html! {
                <button onclick={ ctx.link().callback(|_| Self::Message::SoundPermission) }>
                    { "Lancer le son" }
                </button>
            },
            State::Playing {
                current, ref node, ..
            } => html! {
                <>
                    { self.audios(ctx, current, node.clone()) }
                    { self.pause_button(ctx) }
                </>
            },
            State::Paused { .. } => html! { self.resume_button(ctx) },
            State::Waiting { .. } => html! { "En attente de la phrase suivante ..."  },
            State::Finished => html! { "Jeu terminé !" },
        };

        html! {
            <div>
                <pre> { format!("{:#?}", self) } </pre>
                { state_view }
            </div>
        }
    }
}

impl Game {
    fn audios(
        &self,
        ctx: &Context<Self>,
        (sentence, sentence_state): (Sentence, SentenceState),
        node: NodeRef,
    ) -> Html {
        let src = match sentence_state {
            SentenceState::Family => sentence.family_sound_file(),
            SentenceState::Element => sentence.sentence_sound_file(),
        };
        let onended = ctx.link().callback(|_| GameMsg::SentenceState);

        html! {
            <audio
                // controls=true // use for debugging sounds
                type="audio/mp3"
                id="sound-player"

                ref={ node }
                { src }
                { onended }
            >
                { "Your browser does not support the audio element" }
            </audio>
        }
    }

    fn pause_button(&self, ctx: &Context<Self>) -> Html {
        html! {
            <button onclick={ ctx.link().callback(|_| GameMsg::Pause) }>
                { "Pause" }
            </button>
        }
    }

    fn resume_button(&self, ctx: &Context<Self>) -> Html {
        html! {
            <button onclick={ ctx.link().callback(|_| GameMsg::Resume) }>
                { "Reprendre" }
            </button>
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GameQuery {
    families: u8,
}

impl From<&'_ HashSet<Family>> for GameQuery {
    fn from(h: &'_ HashSet<Family>) -> Self {
        let mut families = 0;
        for f in h {
            families |= *f as u8;
        }
        Self { families }
    }
}

impl TryFrom<GameQuery> for HashSet<Family> {
    type Error = ();

    fn try_from(value: GameQuery) -> Result<Self, Self::Error> {
        let all_families = Family::into_enum_iter().fold(0, |acc, f| acc | f as u8);
        if value.families & all_families == 0 {
            return Err(());
        }

        Ok(Self::from_iter(
            Family::into_enum_iter().filter(|f| *f as u8 & value.families != 0),
        ))
    }
}
