//! An example of a simple volume node oscillating the amplitude of a synth node.

use crate::ui::SynthToUiMessage;

use portaudio as pa;
use std::sync::mpsc;
use sample::{Sample, FromSample};

const CHANNELS: usize = 2;
const FRAMES: u32 = 64;
const SAMPLE_HZ: f64 = 44_100.0;

pub enum UiToSynthMessage {
   ChangeFreq(u32, f64),
}

pub fn run_with<F: FnOnce(mpsc::Sender<UiToSynthMessage>)>(f: F) -> Result<(), pa::Error> {
   // Construct our dsp graph.
   // let mut graph = Graph::new();

   // Construct our fancy Synth and add it to the graph!
   // let synth = graph.add_node(DspNode::Synth{phase: 0.0, synth_hz: 440.0});

   // Set the synth as the master node for the graph.
   // graph.set_master(Some(synth));

   // We'll use this to count down from three seconds and then break from the loop.
   let mut timer: f64 = 0.0;

   let (stop_tx, stop_rx) = mpsc::channel();
   let (ui_tx, ui_rx) = mpsc::channel();

   // The callback we'll use to pass to the Stream. It will request audio from our graph.
   let callback = move |pa::OutputStreamCallbackArgs { buffer, .. }| {
      // let buffer: &mut [[f32; CHANNELS]] = buffer.to_frame_slice_mut().unwrap();

      // Zero the sample buffer.
      // dsp::slice::equilibrium(buffer);

      // Request audio from the graph.
      // graph.audio_requested(buffer, SAMPLE_HZ);

      if let Ok(msg) = ui_rx.try_recv() {
         /* match msg {
            UiToSynthMessage::ChangeFreq(track, freq) => {
               let DspNode::Synth{ref mut phase, ref mut synth_hz} = graph[synth];
               // *phase = 0.0;
               *synth_hz = freq;
            },
         } */
      }

      let dt = FRAMES as f64 / SAMPLE_HZ as f64;
      timer += dt;

      if stop_rx.try_recv().is_ok() { pa::Complete }
      else { pa::Continue }
   };

   // Construct PortAudio and the stream.
   let pa = pa::PortAudio::new()?;
   let settings = pa.default_output_stream_settings::<f32>(CHANNELS as i32, SAMPLE_HZ, FRAMES)?;
   let mut stream = pa.open_non_blocking_stream(settings, callback)?;
   stream.start()?;

   f(ui_tx);
   stop_tx.send(()).unwrap();

   // Wait for our stream to finish.
   while let Ok(true) = stream.is_active() {
      std::thread::sleep(::std::time::Duration::from_millis(16));
   }

   Ok(())
}

/// Our Node to be used within the Graph.
enum DspNode {
  Synth{phase: f64, synth_hz: f64},
}

/// Implement the `Node` trait for our DspNode.
/*
impl Node<[f32; CHANNELS]> for DspNode {
   fn audio_requested(&mut self, buffer: &mut [[f32; CHANNELS]], sample_hz: f64) {
      match *self {
         DspNode::Synth{ref mut phase, synth_hz} => dsp::slice::map_in_place(buffer, |_| {
            let val = sine_wave(*phase);
            *phase += synth_hz / sample_hz;
            Frame::from_fn(|_| val)
         }),
      }
   }
}
*/

/// Return a sine wave for the given phase.
fn sine_wave<S: Sample>(phase: f64) -> S
where
   S: Sample + FromSample<f32>,
{
   use std::f64::consts::PI;
   ((phase * PI * 2.0).sin() as f32).to_sample::<S>()
}