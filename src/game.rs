//! Game page ('/game')
use std::collections::HashSet;
use std::time::Duration;

use enum_iterator::IntoEnumIterator;
use gloo_timers::callback::Interval;
use serde::{Deserialize, Serialize};
use web_sys::{HtmlAudioElement, HtmlInputElement};
use yew::prelude::*;
use yew_router::prelude::*;
use yew_router::scope_ext::HistoryHandle;

use crate::family::Family;
use crate::sentences::{Sentence, Sentences};
use crate::timer::Timer;
use crate::Route;

const MIN_TIMER_DURATION: Duration = Duration::from_secs(3);
const MAX_TIMER_DURATION: Duration = Duration::from_secs(60);
const MIN_TIMER_DURATION_STR: &str = "3";
const MAX_TIMER_DURATION_STR: &str = "60";

pub struct Game {
    duration: Duration,
    sentences: Sentences,
    state: State,
    _history: HistoryHandle,
}

impl std::fmt::Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Game")
            .field("duration", &self.duration)
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
    },
    PlayingPaused {
        current: (Sentence, SentenceState),
        // Since we don't keep the sound node here, it means the current
        // sound will restart from the start when 'Resume' is sent.
        //
        // I did not consider this a default because the sound are very short
        // (one to five words) and the game is made for people learning french:
        // making them remember half a word and hear the second part several
        // seconds later seems like a bad practice.
    },
    Waiting {
        // Both `Interval` and `Timer` are cancelled on drop.
        seconds: Interval,
        time_left: Duration,
        timer: Timer,
    },
    WaitingPaused {
        time_left: Duration,
    },
    Finished,
}

#[derive(Debug, Clone, Copy)]
pub enum SentenceState {
    Family,
    Element,
}

#[derive(Debug)]
pub enum Msg {
    ChangeTimer(u64),
    GoHome,
    NextSentence,
    Pause,
    Resume,
    UpdateTime,
    SentenceState,
    SoundPermission,
}

impl Component for Game {
    type Message = Msg;

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
            .add_history_listener(ctx.link().callback(|_| Msg::GoHome))
            .unwrap();

        Self {
            duration: Duration::from_secs(5),
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
        let st = &self.state;
        gloo_console::debug!(format!("{st:?} --- {msg:?}"));

        match (&mut self.state, msg) {
            // State of game: timer duration was changed before game started.
            (State::GettingSoundPermission, Msg::ChangeTimer(seconds)) => {
                self.duration = Duration::from_secs(seconds).clamp(
                    MIN_TIMER_DURATION,
                    MAX_TIMER_DURATION,
                );
            },
            // State: was waiting for permission to play sound, just got it.
            (State::GettingSoundPermission, Msg::SoundPermission)
            // State: launch next sentence.
            | (State::Waiting { .. }, Msg::NextSentence)
            | (State::Playing { .. }, Msg::NextSentence) => {
                match self.sentences.draw_one() {
                    None => self.state = State::Finished,
                    Some(st) => {
                        self.state = State::Playing {
                            current: (st, SentenceState::Family),
                            node: Default::default(),
                        }
                    }
                }
            }
            // State of the game: a sound just finished playing.
            (State::Playing { current, .. }, Msg::SentenceState | Msg::UpdateTime) => {
                match current {
                    (st, SentenceState::Family) => *current = (*st, SentenceState::Element),
                    (_, SentenceState::Element) => {
                        self.state = State::Waiting {
                            timer: {
                                let link = ctx.link().clone();
                                Timer::new(
                                    self.duration,
                                    move || link.send_message(Msg::NextSentence),
                                )
                            },
                            time_left: self.duration,
                            seconds: {
                                let link = ctx.link().clone();
                                Interval::new(
                                    1_000 /* ms */,
                                    move || link.send_message(Msg::UpdateTime),
                                )
                            },
                        };
                    }
                }
            },
            // State of game: a sound is playing
            (State::Playing { node, current }, Msg::Pause) => {
                node.cast::<HtmlAudioElement>().and_then(|x| x.pause().ok());

                self.state = State::PlayingPaused {
                    current: *current,
                };
            },
            // State of game: waiting for next sentence, a second just passed.
            (State::Waiting { time_left, .. }, Msg::UpdateTime) => {
                *time_left = time_left.saturating_sub(Duration::from_secs(1));
            }
            // State of game: waiting for timer to launch next sentence
            //
            // This will drop the timer and the interval, cancelling them.
            (State::Waiting { timer, .. }, Msg::Pause) => {
                self.state = State::WaitingPaused {
                    time_left: timer.stop(),
                };
            },
            // State of game: resume in playing mode
            (State::PlayingPaused { current }, Msg::Resume) => {
                self.state = State::Playing {
                    node: NodeRef::default(),
                    current: *current,
                };
            }
            // State of game: resume in waiting mode
            (State::WaitingPaused { time_left }, Msg::Resume) => {
                let time_left = *time_left;

                self.state = State::Waiting {
                    timer: {
                        let link = ctx.link().clone();
                        Timer::new(
                            time_left,
                            move || link.send_message(Msg::NextSentence),
                        )
                    },
                    time_left,
                    seconds: {
                        let link = ctx.link().clone();
                        Interval::new(
                            1_000 /* ms */,
                            move || link.send_message(Msg::UpdateTime),
                        )
                    },
                };
            }
            _ => (),
        }

        true
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
                        min={ MIN_TIMER_DURATION_STR }
                        max={ MAX_TIMER_DURATION_STR }
                        step="1"
                        value={ format!("{}", self.duration.as_secs()) }
                        oninput={
                            ctx.link().callback(|e: InputEvent| {
                                // Unchecked: we define the callback inside the element it concerns, we cannot
                                // be referencing the wrong one.
                                let input: HtmlInputElement = e.target_unchecked_into();
                                Msg::ChangeTimer(input.value_as_number().round().clamp(0.0, u64::MAX as _) as u64)
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
                    { audio_player(ctx, current, node.clone()) }
                    { pause_button(ctx) }
                </>
            },
            State::PlayingPaused { .. } => html! { resume_view(ctx) },
            State::Waiting { time_left, .. } => html! {
                <>
                    <p> { format!("En attente de la phrase suivante ... {}s", time_left.as_secs()) } </p>
                    { pause_button(ctx) }
                </>
            },
            State::WaitingPaused { .. } => html! {
                <>
                    <p> { "En attente de la phrase suivante ... (Pause)" } </p>
                    { resume_view(ctx) }
                </>
            },
            State::Finished => html! {
                <>
                    <p> { "Jeu terminé !" } </p>
                    { go_home_button(ctx) }
                </>
            },
        };

        html! {
            <div>
                // <pre> { format!("{:#?}", self) } </pre>
                { state_view }
            </div>
        }
    }
}

fn audio_player(
    ctx: &Context<Game>,
    (sentence, sentence_state): (Sentence, SentenceState),
    node: NodeRef,
) -> Html {
    let src = match sentence_state {
        SentenceState::Family => sentence.family_sound_file(),
        SentenceState::Element => sentence.sentence_sound_file(),
    };
    let onended = ctx.link().callback(|_| Msg::SentenceState);

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

fn pause_button(ctx: &Context<Game>) -> Html {
    html! { <button onclick={ ctx.link().callback(|_| Msg::Pause) }> { "Pause" } </button> }
}

fn resume_view(ctx: &Context<Game>) -> Html {
    html! {
        <>
            <button onclick={ ctx.link().callback(|_| Msg::Resume) }> { "Reprendre" } </button>
            { go_home_button(ctx) }
        </>
    }
}

fn go_home_button(ctx: &Context<Game>) -> Html {
    let history = ctx.link().history().unwrap();
    let onclick = Callback::once(move |_| history.push(Route::Home));

    html! { <button {onclick}> { "Sélectionner d'autres familles" } </button> }
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
