//! Game page ('/game')
//!
//! See [`Game`].
use std::collections::HashSet;
use std::time::Duration;

use enum_iterator::IntoEnumIterator;
use gloo_timers::callback::Interval;
use web_sys::HtmlInputElement;
use yew::html::Scope;
use yew::prelude::*;

use crate::audio::Audio;
use crate::family::Family;
use crate::sentences::{Sentence, Sentences};
use crate::style;
use crate::timer::Timer;

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
    /// The audio element used to play the sound.
    ///
    /// Ideally it would not exist all the time, only during the [`State::Playing`] phase.
    /// This is not possible though, because mobile web browser require a user interaction
    /// to happen between the time the audio element is created and the time a sound is played
    /// from it. If we recreate the element each time, we end up in a situation where there
    /// have been user interactions ([`State::GettingSoundPermission`]) but since the audio
    /// element has been created after, it is not taken into account.
    ///
    /// This is fixed by creating the audio element once on the first load of the page and
    /// simply upadting its [`src`][Audio::set_src()] element each time we switch to the next
    /// element or sentence.
    audio: Audio,
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
    // ----
    SelectingFamilies {
        families: HashSet<Family>,
    },
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
    },
    /// Playing is paused.
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

#[derive(Debug)]
pub enum Msg {
    Before(BeforeGameMsg),
    InGame(InGameMsg),
}

/// Messages sent before a [`Game`] has begun, to select families or launch the
/// game itself.
#[derive(Debug)]
pub enum BeforeGameMsg {
    /// Toggle selection for a family.
    Toggle(Family),
    /// Select all families.
    SelectAllFamilies,
    /// Deselect all families.
    ClearAllFamilies,
    /// Launch the game with the selected families.
    LaunchGame,
}

/// Messages sent during the lifetime of a [`Game`].
#[derive(Debug)]
pub enum InGameMsg {
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

        let link = link.clone();
        let audio = Audio::new(move |_| link.send_message(InGameMsg::SentenceState));

        Self {
            audio,
            duration: Duration::from_secs(20),
            /// Sentences are empty at first
            sentences: Sentences::new(&Default::default()),
            state: State::SelectingFamilies {
                families: Default::default(),
            },
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        if let State::Playing {
            current: (st, state),
        } = &self.state
        {
            // The source is reset on each render of the `Playing` state.
            //
            // This notably means that pausing and resuming will reset the progress
            // in the sound. This is voluntary: the sound are very short and makes little
            // to no sense if taken mid-step, a bad combination for a game intended for
            // people learning Frennch.
            self.audio.set_src(match state {
                SentenceState::Element => st.element_sound_file(),
                SentenceState::Family => st.family_sound_file(),
            });
            self.audio.play();
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Before(msg) => self.update_before_game(msg),
            Msg::InGame(msg) => self.update_in_game(ctx, msg),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        match self.state {
            State::SelectingFamilies { ref families } => {
                let link = ctx.link();

                html! {
                    <div>
                        <button onclick={ link.callback(|_| BeforeGameMsg::SelectAllFamilies) } class={style::button_select_all("#4CAF50")}>
                            { "Sélectionner toutes les familles" }
                        </button>
                        <button onclick={ link.callback(|_| BeforeGameMsg::ClearAllFamilies) } class={style::button_select_all("#F44336")}>
                            { "Tout déselectionner" }
                        </button>
                        <hr />
                        { family_view(link, families) }
                        { start_button(link, families) }
                    </div>
                }
            }
            // State: we do not yet have permission to play sound.
            State::GettingSoundPermission => html! {
                <>
                    <button onclick={ ctx.link().callback(|_| InGameMsg::SoundPermission) }>
                        { "Lancer la partie" }
                    </button>
                    <hr />
                    { timer_slider(link, self.duration) }
                </>
            },
            // State: sound is currently playing.
            State::Playing { .. } => html! { pause_button(link) },
            // State: sound was paused.
            State::PlayingPaused { .. } => html! { resume_view(link, self.duration) },
            // State: waiting for the coutdown to the next sentence to end.
            State::Waiting { time_left, .. } => html! {
                <>
                    { pause_button(link) }
                    { next_sentence_button(link) }
                    <p> { format!("Phrase suivante dans ... {}s", time_left.as_secs()) } </p>
                </>
            },
            // State: countdown to next sentence was paused.
            State::WaitingPaused { time_left, .. } => html! {
                <>
                    { resume_view(link, self.duration) }
                    { next_sentence_button(link) }
                    <p> { format!("Phrase suivante dans ... {}s (Pause)", time_left.as_secs()) } </p>
                </>
            },
            // State: game is finished, nothing more to do.
            State::Finished => html! {
                <>
                    { go_home_button(link) }
                    <p> { "Jeu terminé !" } </p>
                </>
            },
        }
    }
}

impl Game {
    fn update_in_game(&mut self, ctx: &Context<Self>, msg: InGameMsg) -> bool {
        match (&mut self.state, msg) {
            // State of game: timer duration was changed before game started or during a pause.
            (State::GettingSoundPermission | State::WaitingPaused { .. } | State::PlayingPaused { .. }, InGameMsg::ChangeTimer(seconds)) => {
                self.duration = Duration::from_secs(seconds).clamp(
                    MIN_TIMER_DURATION,
                    MAX_TIMER_DURATION,
                );
            },
            // State: was waiting for permission to play sound, just got it.
            (State::GettingSoundPermission, InGameMsg::SoundPermission)
            // State: launch next sentence (either because the timer just ended or because a "Phrase suivante" button was clicked).
            | (State::Waiting { .. }, InGameMsg::NextSentence)
            | (State::WaitingPaused { .. }, InGameMsg::NextSentence)
            // State: the last sentence was drawn, end the game immediately.
            | (State::Playing { .. }, InGameMsg::NextSentence) => {
                match self.sentences.draw_one() {
                    None => self.state = State::Finished,
                    Some(st) => {
                        self.state = State::Playing {
                            current: (st, SentenceState::Family),
                        }
                    }
                }
            }
            // State of the game: a sound just finished playing.
            (State::Playing { current }, InGameMsg::SentenceState) => {
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
            (State::Playing { current }, InGameMsg::Pause) => {
                self.audio.pause();

                self.state = State::PlayingPaused {
                    current: *current,
                };
            },
            // State of game: waiting for next sentence, a second just passed.
            (State::Waiting { time_left, .. }, InGameMsg::UpdateTime) => {
                *time_left = time_left.saturating_sub(Duration::from_secs(1));
            }
            // State of game: waiting for timer to launch next sentence
            //
            // This will drop the timer and the interval, cancelling them.
            (State::Waiting { timer, .. }, InGameMsg::Pause) => {
                self.state = State::WaitingPaused {
                    time_left: timer.stop(),
                };
            },
            // State of game: resume in playing mode
            (State::PlayingPaused { current }, InGameMsg::Resume) => {
                self.state = State::Playing {
                    current: *current,
                };
            }
            // State of game: resume in waiting mode
            (State::WaitingPaused { time_left }, InGameMsg::Resume) => {
                self.state = waiting_state(ctx.link(), *time_left);
            }
            // State of game: received a go home event
            (State::PlayingPaused { .. } | State::WaitingPaused { .. } | State::Finished, InGameMsg::GoHome) => {
                self.state = State::SelectingFamilies { families: Default::default() };
                self.sentences = Sentences::new(&Default::default());
            }
            _ => (),
        }

        true
    }

    fn update_before_game(&mut self, msg: BeforeGameMsg) -> bool {
        let families = match &mut self.state {
            State::SelectingFamilies { families } => families,
            _ => return false,
        };

        match msg {
            BeforeGameMsg::Toggle(f) => {
                if families.contains(&f) {
                    families.remove(&f);
                } else {
                    families.insert(f);
                }
            }
            BeforeGameMsg::SelectAllFamilies => families.extend(Family::into_enum_iter()),
            BeforeGameMsg::ClearAllFamilies => families.clear(),
            BeforeGameMsg::LaunchGame => {
                self.sentences = Sentences::new(families);
                self.state = State::GettingSoundPermission;
            }
        }

        true
    }
}

/// Make all the families available for selection/deselection.
fn family_view(link: &Scope<Game>, families: &HashSet<Family>) -> Html {
    html! {
        <div>
            { for Family::into_enum_iter().map(|f| f.render(link, families.contains(&f))) }
        </div>
    }
}

/// The start button is only shown if at least one family has been selected
/// to play.
fn start_button(link: &Scope<Game>, families: &HashSet<Family>) -> Html {
    if !families.is_empty() {
        let onclick = link.callback(|_| BeforeGameMsg::LaunchGame);

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
            Timer::new(time_left, move || {
                link.send_message(InGameMsg::NextSentence)
            })
        },
        seconds: {
            let link = link.clone();
            Interval::new(
                1_000, /* ms */
                move || link.send_message(InGameMsg::UpdateTime),
            )
        },
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
                        InGameMsg::ChangeTimer(input.value_as_number().round().clamp(0.0, u64::MAX as _) as u64)
                    })
                }
            />
            <p> { format!("Temps entre deux phrases: {}s", current_duration.as_secs()) } </p>
        </>
    }
}

/// Button to click on to [pause][Msg::Pause] the game.
fn pause_button(link: &Scope<Game>) -> Html {
    html! { <button onclick={ link.callback(|_| InGameMsg::Pause) }> { "Pause" } </button> }
}

/// Button to click on to [skip to the next sentence][Msg::NextSentence].
fn next_sentence_button(link: &Scope<Game>) -> Html {
    html! { <button onclick={ link.callback(|_| InGameMsg::NextSentence) }> { "Phrase suivante" } </button> }
}

/// View shown when the game is paused.
///
/// It displays a ["Reprendre"][Msg::Resume] button, a [slider][timer_slider()]
/// to select the duration of the next coutdown to the next sentence and a
/// [button to go home][go_home_button()].
fn resume_view(link: &Scope<Game>, current_duration: Duration) -> Html {
    html! {
        <>
            <button onclick={ link.callback(|_| InGameMsg::Resume) }> { "Reprendre" } </button>
            { timer_slider(link, current_duration) }
            <hr />
            { go_home_button(link) }
        </>
    }
}

/// Button to go back to ['/'][crate::app::App] and selecting families.
///
/// This **needs** an [`HistoryHandle`] to be present in the [`Game`] struct
/// else it will panic trying to access it.
fn go_home_button(link: &Scope<Game>) -> Html {
    let onclick = link.callback(|_| InGameMsg::GoHome);

    html! { <button {onclick}> { "Retourner à la sélection de familles" } </button> }
}

impl From<BeforeGameMsg> for Msg {
    fn from(value: BeforeGameMsg) -> Self {
        Self::Before(value)
    }
}

impl From<InGameMsg> for Msg {
    fn from(value: InGameMsg) -> Self {
        Self::InGame(value)
    }
}
