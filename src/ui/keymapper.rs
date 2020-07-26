use super::effect::Effectful;
use crate::audio::UiToSynthMessage;

use std::sync::mpsc;
use std::collections::BTreeMap;
use iced_native::input::keyboard::KeyCode;

pub struct KeyMapper {
   keymap: BTreeMap<KeyCode, Effectful>,
}

impl KeyMapper {
   pub fn keyboard() -> Self {
      let mut keymap = BTreeMap::new();

      [  KeyCode::A,
         KeyCode::Z,
         KeyCode::S,
         KeyCode::X,
         KeyCode::C,
         KeyCode::F,
         KeyCode::V,
         KeyCode::G,
         KeyCode::B,
         KeyCode::N,
         KeyCode::J,
         KeyCode::M,
         KeyCode::K,
         KeyCode::Comma,
         KeyCode::L,
         KeyCode::Period,
         KeyCode::Slash,
         KeyCode::Apostrophe,
         KeyCode::RShift,
      ].iter().enumerate().for_each( |(k, &code)| {
         keymap.insert(code, Effectful::ChangePitch(0, k as f64 - 1.0));
      } );

      [  KeyCode::Key1,
         KeyCode::Q,
         KeyCode::Key2,
         KeyCode::W,
         KeyCode::E,
         KeyCode::Key4,
         KeyCode::R,
         KeyCode::Key5,
         KeyCode::T,
         KeyCode::Y,
         KeyCode::Key7,
         KeyCode::U,
         KeyCode::Key8,
         KeyCode::I,
         KeyCode::Key9,
         KeyCode::O,
         KeyCode::P,
         KeyCode::Minus,
         KeyCode::LBracket,
         KeyCode::Equals,
         KeyCode::RBracket,
         KeyCode::Backslash,
      ].iter().enumerate().for_each( |(k, &code)| {
         keymap.insert(code, Effectful::ChangePitch(1, k as f64 + 11.0));
      } );

      Self{keymap}
   }

   pub fn execute(&self, keycode: KeyCode, synth_tx: &mpsc::Sender<UiToSynthMessage>) {
      if let Some(effectful) = self.keymap.get(&keycode) {
         effectful.execute(synth_tx);
      }
   }
}