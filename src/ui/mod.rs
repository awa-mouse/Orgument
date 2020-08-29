mod keymapper;
mod effect;

use keymapper::KeyMapper;
use crate::UiToSynthMessage;

use std::sync::mpsc;
use iced::{button, Button, Column, Text, executor, Application, Command, Element, Settings, Subscription};

pub fn run(synth_tx: mpsc::Sender<UiToSynthMessage>) {
   UI::run(Settings::with_flags(Flags{synth_tx}))
}

pub struct UI {
   keymapper: KeyMapper,
   synth_tx: mpsc::Sender<UiToSynthMessage>,
}

#[derive(Debug, Clone)]
pub enum Message {
   NativeEvent(iced_native::Event),
}

pub struct SynthToUiMessage {
}

#[derive(Debug, Clone)]
pub struct Flags {
   synth_tx: mpsc::Sender<UiToSynthMessage>,
}

impl Application for UI {
   type Executor = executor::Default;
   type Message = Message;
   type Flags = Flags;

   fn new(flags: Flags) -> (UI, Command<Self::Message>) {
      ( UI {
         keymapper: KeyMapper::keyboard(),
         synth_tx: flags.synth_tx,
      }, Command::none() )
   }

   fn title(&self) -> String {
      String::from("A cool application")
   }

   fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
      match message {
         Message::NativeEvent(iced_native::Event::Keyboard(iced_native::input::keyboard::Event::Input{state: iced_native::input::ButtonState::Pressed, key_code, ..})) => {
            self.keymapper.execute(key_code, &self.synth_tx);
         },
         _ => {},
      }

      Command::none()
   }

   fn view(&mut self) -> Element<Self::Message> {
      Text::new("Hello, world!").into()
   }

   fn subscription(&self) -> Subscription<Message> {
      iced_native::subscription::events().map(Message::NativeEvent)
   }
}