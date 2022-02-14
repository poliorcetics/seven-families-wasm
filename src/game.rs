//! Game page ('/game')
use std::collections::HashSet;
use std::time::Duration;

use enum_iterator::IntoEnumIterator;
use gloo_timers::callback::Interval;
use serde::{Deserialize, Serialize};
use web_sys::{HtmlAudioElement, HtmlInputElement};
use yew::html::Scope;
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
pub enum State { // Don't sort alphabetically here, we want to follow the flow of the game.
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
    Element,
    Family,
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
        let link = ctx.link();
        let location = link.location().unwrap();
        let families = location
            .query::<GameQuery>()
            .unwrap_or(GameQuery { families: 0 });
        let families = families
            .try_into()
            .unwrap_or_else(|_| HashSet::from_iter(Family::into_enum_iter()));

        let history = link
            .add_history_listener(link.callback(|_| Msg::GoHome))
            .unwrap();

        Self {
            duration: Duration::from_secs(5),
            sentences: Sentences::new(families),
            state: State::GettingSoundPermission,
            _history: history,
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        if let State::Playing { node, .. } = &self.state {
            node.cast::<HtmlAudioElement>()
                .and_then(|audio| audio.play().ok());
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let st = &self.state;
        gloo_console::debug!(format!("{st:?} --- {msg:?}"));

        match (&mut self.state, msg) {
            // State of game: timer duration was changed before game started.
            (State::GettingSoundPermission | State::WaitingPaused { .. } | State::PlayingPaused { .. }, Msg::ChangeTimer(seconds)) => {
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
                        if self.sentences.is_empty() {
                            self.state = State::Finished;
                        } else {
                            self.state = waiting_state(ctx.link(), self.duration);
                        }
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
                self.state = waiting_state(ctx.link(), *time_left);
            }
            _ => (),
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let state_view = match self.state {
            State::GettingSoundPermission => html! {
                <>
                    <button onclick={ ctx.link().callback(|_| Self::Message::SoundPermission) }>
                        { "Lancer le son" }
                    </button>
                    { timer_slider(link, self.duration) }
                </>
            },
            State::Playing {
                current, ref node, ..
            } => html! {
                <>
                    { audio_player(link, current, node.clone()) }
                    { pause_button(link) }
                </>
            },
            State::PlayingPaused { .. } => html! { resume_view(link, self.duration) },
            State::Waiting { time_left, .. } => html! {
                <>
                    <p> { format!("Phrase suivante dans ... {}s", time_left.as_secs()) } </p>
                    { pause_button(link) }
                </>
            },
            State::WaitingPaused { time_left, .. } => html! {
                <>
                    <p> { format!("Phrase suivante dans ... {}s (Pause)", time_left.as_secs()) } </p>
                    { resume_view(link, self.duration) }
                </>
            },
            State::Finished => html! {
                <>
                    <p> { "Jeu terminé !" } </p>
                    { go_home_button(link) }
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

fn waiting_state(link: &Scope<Game>, time_left: Duration) -> State {
    State::Waiting {
        time_left,
        timer: {
            let link = link.clone();
            Timer::new(time_left, move || link.send_message(Msg::NextSentence))
        },
        seconds: {
            let link = link.clone();
            Interval::new(
                1_000, /* ms */
                move || link.send_message(Msg::UpdateTime),
            )
        },
    }
}
fn audio_player(
    link: &Scope<Game>,
    (sentence, sentence_state): (Sentence, SentenceState),
    node: NodeRef,
) -> Html {
    let src = match sentence_state {
        SentenceState::Family => sentence.family_sound_file(),
        SentenceState::Element => sentence.sentence_sound_file(),
    };
    let onended = link.callback(|_| Msg::SentenceState);

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

fn timer_slider(link: &Scope<Game>, current_duration: Duration) -> Html {
    html! {
        <>
            <input
                name="ratio"
                type="range"
                min={ MIN_TIMER_DURATION_STR }
                max={ MAX_TIMER_DURATION_STR }
                step="1"
                value={ format!("{}", current_duration.as_secs()) }
                oninput={
                    link.callback(|e: InputEvent| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        Msg::ChangeTimer(input.value_as_number().round().clamp(0.0, u64::MAX as _) as u64)
                    })
                }
            />
            { format!("Compteur: {}s", current_duration.as_secs()) }
        </>
    }
}

fn pause_button(link: &Scope<Game>) -> Html {
    html! { <button onclick={ link.callback(|_| Msg::Pause) }> { "Pause" } </button> }
}
fn resume_view(link: &Scope<Game>, current_duration: Duration) -> Html {
    html! {
        <>
            <button onclick={ link.callback(|_| Msg::Resume) }> { "Reprendre" } </button>
            { timer_slider(link, current_duration) }
            { go_home_button(link) }
        </>
    }
}

fn go_home_button(link: &Scope<Game>) -> Html {
    let history = link.history().unwrap();
    let onclick = Callback::once(move |_| history.push(Route::Home));

    html! { <button {onclick}> { "Retourner à la sélection de familles" } </button> }
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
