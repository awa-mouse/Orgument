use crate::audio::UiToSynthMessage;

use std::sync::mpsc;

pub struct Id(u32);

pub enum Effectful {
   ChangePitch(f64),
}

impl Effectful {
   pub fn execute(&self, synth_tx: &mpsc::Sender<UiToSynthMessage>) {
      match self {
         Effectful::ChangePitch(x) => {
            synth_tx.send( UiToSynthMessage::ChangeFreq(440.0 * 2.0_f64.powf(x / 12.0)) ).unwrap();
         },
      }
   }
}