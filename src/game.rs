//! Game page ('/game')
use std::collections::HashSet;
use std::time::Duration;

use enum_iterator::IntoEnumIterator;
use serde::{Deserialize, Serialize};
use web_sys::{HtmlAudioElement, HtmlInputElement};
use yew::prelude::*;
use yew_router::prelude::*;
use yew_router::scope_ext::HistoryHandle;

use crate::family::Family;
use crate::sentences::{Sentence, Sentences};
use crate::timer::Timer;
use crate::Route;

pub struct Game {
    duration: Duration,
    sentences: Sentences,
    state: State,
    _history: HistoryHandle,
}

impl std::fmt::Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Game")
            .field("sentences", &self.sentences)
            .field("state", &self.state)
            .finish()
    }
}

#[derive(Debug)]
pub enum State {
    GettingSoundPermission,
    Playing {
        current: (Sentence, SentenceState),
        node: NodeRef,
        timer: Option<Timer>,
    },
    Paused {
        // A pause can happen during a 'waiting' state, in which case there is
        // no current sentence.
        current: Option<(Sentence, SentenceState)>,
        timer: Option<Timer>,
        // Since we don't keep the sound node here, it means the current
        // sound will restart from the start when 'Resume' is sent.
        //
        // I did not consider this a default because the sound are very short
        // (one to five words) and the game is made for people learning french:
        // making them remember half a word and hear the second part several
        // seconds later seems like a bad practice.
    },
    Waiting {
        timer: Option<Timer>,
    },
    Finished,
}

#[derive(Debug, Clone, Copy)]
pub enum SentenceState {
    Family,
    Element,
}

pub enum GameMsg {
    GoHome,
    SentenceState,
    NextSentence,
    SoundPermission,
    Pause,
    Resume,
    ChangeTimer(u64),
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

        let history = ctx
            .link()
            .add_history_listener(ctx.link().callback(|_| GameMsg::GoHome))
            .unwrap();

        Self {
            duration: Duration::from_secs(20),
            sentences: Sentences::new(families),
            state: State::GettingSoundPermission,
            _history: history,
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        if let State::Playing { ref node, .. } = self.state {
            node.cast::<HtmlAudioElement>().and_then(|x| x.play().ok());
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match (&mut self.state, msg) {
            // State of game: timer duration was changed before game started.
            (State::GettingSoundPermission, GameMsg::ChangeTimer(seconds)) => {
                self.duration = Duration::from_secs(seconds).clamp(
                    Self::MIN_TIMER_DURATION,
                    Self::MAX_TIMER_DURATION,
                );

                true
            },
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
                                Timer::new(
                                    self.duration,
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
                    current, timer, ..
                },
                GameMsg::SentenceState,
            ) => {
                match current {
                    (st, SentenceState::Family) => *current = (*st, SentenceState::Element),
                    (_, SentenceState::Element) => {
                        self.state = State::Waiting {
                            timer: timer.take(),
                        };
                    }
                }

                true
            },
            // State of game: a sound is playing
            (State::Playing { timer, node, current }, GameMsg::Pause) => {
                let sound_duration = node.cast::<HtmlAudioElement>().map_or(0.0, |x| { x.pause().ok(); x.duration() });
                if let Some(t) = timer.as_mut() {
                    t.pause();
                    t.extend(Duration::from_secs_f64(sound_duration));
                }

                self.state = State::Paused {
                    current: Some(*current),
                    timer: timer.take(),
                };

                true
            },
            // State of game: waiting for timer to launch next sentence
            (State::Waiting { timer, .. }, GameMsg::Pause) => {
                if let Some(t) = timer.as_mut() {
                    t.pause();
                }

                self.state = State::Paused {
                    current: None,
                    timer: timer.take(),
                };

                true
            },
            // State of game: resume paused game
            (State::Paused { current, timer }, GameMsg::Resume) => {
                let mut timer = timer.take();
                if let Some(t) = timer.as_mut() {
                    let link = ctx.link().clone();
                    t.resume(
                        move || link.send_message(GameMsg::NextSentence),
                    );
                }
                let  node = NodeRef::default();

                match *current {
                    None => self.state = State::Waiting { timer },
                    Some(current) => self.state = State::Playing {
                        timer,
                        node,
                        current,
                    },
                }

                true
            }
            // State of game: gome home button was touched
            (State::Paused { .. } | State::Finished, GameMsg::GoHome) => true,
            _ => false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let state_view = match self.state {
            State::GettingSoundPermission => html! {
                <>
                    <button onclick={ ctx.link().callback(|_| Self::Message::SoundPermission) }>
                        { "Lancer le son" }
                    </button>
                    <input
                        name="ratio"
                        type="range"
                        min={ Self::MIN_TIMER_DURATION_STR }
                        max={ Self::MAX_TIMER_DURATION_STR }
                        step="1"
                        value={ format!("{}", self.duration.as_secs()) }
                        oninput={
                            ctx.link().callback(|e: InputEvent| {
                                // Unchecked: we define the callback inside the element it concerns, we cannot
                                // be referencing the wrong one.
                                let input: HtmlInputElement = e.target_unchecked_into();
                                GameMsg::ChangeTimer(input.value_as_number().round().clamp(0.0, u64::MAX as _) as u64)
                            })
                        }
                    />
                    { format!("Compteur: {}s", self.duration.as_secs()) }
                </>
            },
            State::Playing {
                current, ref node, ..
            } => html! {
                <>
                    { self.audio_player(ctx, current, node.clone()) }
                    { self.pause_button(ctx) }
                </>
            },
            State::Paused { .. } => html! { self.resume_button(ctx) },
            State::Waiting { .. } => html! { "En attente de la phrase suivante ..."  },
            State::Finished => html! {
                <>
                    <p> { "Jeu terminé !" } </p>
                    { self.go_home_button(ctx) }
                </>
            },
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
    const MIN_TIMER_DURATION: Duration = Duration::from_secs(10);
    const MAX_TIMER_DURATION: Duration = Duration::from_secs(60);
    const MIN_TIMER_DURATION_STR: &'static str = "10";
    const MAX_TIMER_DURATION_STR: &'static str = "60";

    fn audio_player(
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
            <>
                <button onclick={ ctx.link().callback(|_| GameMsg::Resume) }>
                    { "Reprendre" }
                </button>
                { self.go_home_button(ctx) }
            </>
        }
    }

    fn go_home_button(&self, ctx: &Context<Self>) -> Html {
        let history = ctx.link().history().unwrap();
        let onclick = Callback::once(move |_| history.push(Route::Home));

        html! {
            <button {onclick}>
                { "Sélectionner d'autres familles" }
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
