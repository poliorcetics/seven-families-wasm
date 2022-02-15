//! Game page ('/game')
//!
//! See [`Game`].
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

/// Minimum time between two sentences.
const MIN_TIMER_DURATION: Duration = Duration::from_secs(3);
/// Maximum time between two sentences.
const MAX_TIMER_DURATION: Duration = Duration::from_secs(60);
/// String representation for javascript.
const MIN_TIMER_DURATION_STR: &str = "3";
/// String representation for javascript.
const MAX_TIMER_DURATION_STR: &str = "60";

/// Game component.
///
/// To be as safe as possible, states are tracked trough an enum; [`State`],
/// so that only the relevant information can be access in each state.
///
/// This is not possible for everything, especially data that must live through
/// several non-contiguous states.
pub struct Game {
    /// Time interval between each sentence.
    ///
    /// See [`MIN_TIMER_DURATION`], [`MAX_TIMER_DURATION`] and [`State::Waiting`].
    duration: Duration,
    /// The sentences selected to play the game.
    ///
    /// They are parsed from a [`GameQuery`] on construction.
    sentences: Sentences,
    /// State of the game.
    state: State,
    /// Used to access (indirectly) the history to go back to [`/`][Route::Home].
    ///
    /// Never read directly but must be present since the handle is
    /// reference counted and is not tracked by Yew directly to avoid
    /// keeping unnecessary data when possible.
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

/// States of the game.
#[derive(Debug)]
pub enum State {
    // Don't sort alphabetically here, we want to follow the flow of the game.
    /// Waiting for permission to play sound.
    ///
    /// To avoid noisy ads that autostart and such, browsers asks for at least one user interaction
    /// with a page before playing sound. Since our whole game is built around sound, we ask for
    /// that interaction (a click on a button) once at the start.
    ///
    /// This would be cumbersome if it was the only thing displayed then so this state also offers
    /// the possibility of setting the duration between sentences.
    GettingSoundPermission,
    /// Sound is currently playing.
    Playing {
        /// The sentence and at which point of if the game is.
        current: (Sentence, SentenceState),
        /// Reference to the sound node, to play/pause it.
        node: NodeRef,
    },
    /// Playing is paused.
    ///
    /// Since we don't keep the sound node here, it means the current
    /// sound will restart from the start when 'Resume' is sent.
    ///
    /// I did not consider this a default because the sound are very short
    /// (one to five words) and the game is made for people learning french:
    /// making them remember half a word and hear the second part several
    /// seconds later seems like a bad practice.
    PlayingPaused {
        /// The sentence to resume and which part of it.
        current: (Sentence, SentenceState),
    },
    /// Waiting for the next sentence.
    Waiting {
        // Both `Interval` and `Timer` are cancelled on drop.
        /// Sends a message each second to update the countdown
        /// to the [next sentence][Msg::NextSentence].
        seconds: Interval,
        /// Countdown display to the next sentence.
        time_left: Duration,
        /// Coutdown to the next sentence, will send a message
        /// once complete.
        timer: Timer,
    },
    /// Waiting for the next sentence is paused.
    WaitingPaused {
        /// What's left of the countdown to the next sentence.
        time_left: Duration,
    },
    /// Game is finished.
    Finished,
}

/// A sentence is composed of two parts (with regard to the sound files).
#[derive(Debug, Clone, Copy)]
pub enum SentenceState {
    /// Second part of the sentence, unique for each element.
    Element,
    /// First part of the sentence, the same for each element.
    Family,
}

/// Messages sent during the lifetime of a [`Game`].
#[derive(Debug)]
pub enum Msg {
    /// Update the start duration of the countdown
    /// to the next sentence.
    ChangeTimer(u64),
    /// Go back to [`/`][Route::Home].
    GoHome,
    /// Launch next sentence sound.
    NextSentence,
    /// Pause the game.
    Pause,
    /// Resume playing.
    Resume,
    /// Either the first or second sound of a whole
    /// sentence just completed.
    SentenceState,
    /// Got permission to play sound.
    SoundPermission,
    /// Update coutdown to next sentence.
    ///
    /// See also [`timer_slider()`].
    UpdateTime,
}

impl Component for Game {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link();

        // Unwrap: we are in a `BrowserRouter` switch, see 'crate::switch'.
        let location = link.location().unwrap();
        let families = location
            .query::<GameQuery>()
            .unwrap_or(GameQuery { families: 0 });
        let families = families
            .try_into()
            .unwrap_or_else(|_| HashSet::from_iter(Family::into_enum_iter()));

        // Unwrap: we are in a `BrowserRouter` switch, see 'crate::switch'.
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
            // State of game: timer duration was changed before game started or during a pause.
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
            // State: we do not yet have permission to play sound.
            State::GettingSoundPermission => html! {
                <>
                    <button onclick={ ctx.link().callback(|_| Self::Message::SoundPermission) }>
                        { "Lancer le son" }
                    </button>
                    { timer_slider(link, self.duration) }
                </>
            },
            // State: sound is currently playing.
            State::Playing {
                current, ref node, ..
            } => html! {
                <>
                    { audio_player(link, current, node.clone()) }
                    { pause_button(link) }
                </>
            },
            // State: sound was paused.
            State::PlayingPaused { .. } => html! { resume_view(link, self.duration) },
            // State: waiting for the coutdown to the next sentence to end.
            State::Waiting { time_left, .. } => html! {
                <>
                    <p> { format!("Phrase suivante dans ... {}s", time_left.as_secs()) } </p>
                    { pause_button(link) }
                </>
            },
            // State: countdown to next sentence was paused.
            State::WaitingPaused { time_left, .. } => html! {
                <>
                    <p> { format!("Phrase suivante dans ... {}s (Pause)", time_left.as_secs()) } </p>
                    { resume_view(link, self.duration) }
                </>
            },
            // State: game is finished, nothing more to do.
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

/// Produce a [`State::Waiting`] instance filled correctly with the
/// time left for the [`Timer`] to the next sentence and sending the
/// [`Msg::UpdateTime`] every second for the countdown display.
///
/// Used on [`Msg::Resume`] and when the [`SentenceState::Element`] sound
/// finishes and the countdown to the next sentence must be launched.
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

/// Invisible audio player which contains the audio HTML element to play the sentences.
fn audio_player(
    link: &Scope<Game>,
    (sentence, sentence_state): (Sentence, SentenceState),
    node: NodeRef,
) -> Html {
    let src = match sentence_state {
        SentenceState::Family => sentence.family_sound_file(),
        SentenceState::Element => sentence.element_sound_file(),
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

/// Slider to select the duration of the next countdown to the next sentence.
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
                        // Unchecked: we define the callback inside the element it concerns, we cannot
                        // be referencing the wrong one.
                        let input: HtmlInputElement = e.target_unchecked_into();
                        Msg::ChangeTimer(input.value_as_number().round().clamp(0.0, u64::MAX as _) as u64)
                    })
                }
            />
            { format!("Compteur: {}s", current_duration.as_secs()) }
        </>
    }
}

/// Button to click on to [pause][Msg::Pause] the game.
fn pause_button(link: &Scope<Game>) -> Html {
    html! { <button onclick={ link.callback(|_| Msg::Pause) }> { "Pause" } </button> }
}

/// View shown when the game is paused.
///
/// It displays a ["Reprendre"][Msg::Resume] button, a [slider][timer_slider()]
/// to select the duration of the next coutdown to the next sentence and a
/// [button to go home][go_home_button()].
fn resume_view(link: &Scope<Game>, current_duration: Duration) -> Html {
    html! {
        <>
            <button onclick={ link.callback(|_| Msg::Resume) }> { "Reprendre" } </button>
            { timer_slider(link, current_duration) }
            { go_home_button(link) }
        </>
    }
}

/// Button to go back to ['/'][crate::app::App] and selecting families.
///
/// This **needs** an [`HistoryHandle`] to be present in the [`Game`] struct
/// else it will panic trying to access it.
fn go_home_button(link: &Scope<Game>) -> Html {
    // Unwrap: there is an `HistoryHandle` saved in the `Game` struct,
    // whatever the state is.
    let history = link.history().unwrap();
    // Once: since we change page, this button will disappear and cannot be clicked twice.
    let onclick = Callback::once(move |_| history.push(Route::Home));

    html! { <button {onclick}> { "Retourner à la sélection de familles" } </button> }
}

/// When going to the '/game' page, a query can be provided to select only some families.
/// If no query is present or it is invalid, all families are selected as a default.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GameQuery {
    /// The families to select, as a bitmask.
    ///
    /// See [`Family`] for more details.
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
