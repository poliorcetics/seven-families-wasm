//! See [`Audio`].
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::HtmlAudioElement;
use yew::prelude::Event;

/// Audio element with a mandatory `onended` callback.
///
/// Wrapper around an [`HtmlAudioElement`] that takes care of keeping
/// the callback alive.
pub struct Audio {
    /// Inner element, created once only.
    inner: HtmlAudioElement,
    /// Handle to keep the `onended` closure alive for later use.
    _onended_listener: Closure<dyn Fn(Event)>,
}

impl Audio {
    /// Creates a new `Audio` with the given callback for the `onended` event.
    pub fn new(onended: impl Fn(Event) + 'static) -> Self {
        let inner = HtmlAudioElement::new().unwrap();
        let onended_listener = Closure::<dyn Fn(Event)>::wrap(Box::new(onended));
        inner.set_onended(Some(onended_listener.as_ref().unchecked_ref()));
        Self {
            inner,
            _onended_listener: onended_listener,
        }
    }

    /// Play the audio. A source must have been set with [`Self::set_src()`] before.
    pub fn play(&self) {
        self.inner.play().ok();
    }

    /// Pause the audio.
    pub fn pause(&self) {
        self.inner.pause().ok();
    }

    /// Set the source to play.
    pub fn set_src(&self, src: &str) {
        self.inner.set_src(src)
    }
}
