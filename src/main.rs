#[macro_use] extern crate matches2;
#[macro_use] extern crate linear_map;

mod parser_org;
mod ui;
mod audio;
mod processing;

use crate::ui::SynthToUiMessage;
use processing::processor::{Buffer, GenericSampledBuffer};
use processing::element::Element;
use processing::prim_element::PrimElement;
use processing::Value;

use std::sync::{mpsc, Arc, Mutex};

pub enum UiToSynthMessage {
   ChangeFreq(u32, f64),
}

const F_NYQ: u64 = audio::SAMPLE_HZ / 2;

fn main() {
   let (tx, rx) = mpsc::channel();

   let mut store = processing::Store::new();
   let global_flow = store.add_flow();

   let output_type = processing::Type::Sampled{ty: processing::PrimType::F32, f_nyq: F_NYQ};

   let (out_l, out_l_ix) = store.add_output(global_flow, output_type);
   let (out_r, out_r_ix) = store.add_output(global_flow, output_type);


   let freq = store.add_element(global_flow, Element::Prim(PrimElement::Constant{value: Value::F32(440.0.into()), f_nyq: F_NYQ}));
   let a_m = store.add_element(global_flow, Element::Prim(PrimElement::Constant{value: Value::F32(10.0.into()), f_nyq: F_NYQ}));
   let osc = store.add_element(global_flow, Element::Prim(PrimElement::SineOscF32{f_nyq: F_NYQ}));
   let osc2 = store.add_element(global_flow, Element::Prim(PrimElement::SineOscF32{f_nyq: F_NYQ}));
   let amp = store.add_element(global_flow, Element::Prim(PrimElement::Constant{value: Value::F32(440.0.into()), f_nyq: F_NYQ}));

   store.add_edge(global_flow, freq, 0.into(), osc, 0.into()).unwrap();
   store.add_edge(global_flow, osc, 0.into(), out_l_ix, 0.into()).unwrap();
   store.add_edge(global_flow, osc, 0.into(), out_r_ix, 0.into()).unwrap();


   let out_buffer = Arc::new(Mutex::new( (Buffer::new(output_type), Buffer::new(output_type)) ));

   let (request_tx, request_rx) = mpsc::channel();
   let (response_tx, response_rx) = mpsc::channel();

   let out_buffer_for_processing_thread = out_buffer.clone();
   let _processing_thread = std::thread::spawn(move || {
      while let Ok(buffer_sz) = request_rx.recv() {
         let mut out_buffer = out_buffer_for_processing_thread.lock().unwrap();
         let (ref mut out_buffer_l, ref mut out_buffer_r) = *out_buffer;
         let mut output = std::iter::once((out_l, out_buffer_l)).chain(std::iter::once((out_r, out_buffer_r))).collect();
         let input = std::iter::empty().collect();

         store.compute_outplace(global_flow, &mut output, &input, buffer_sz);

         response_tx.send(()).unwrap();
      }
   });

   audio::run_with(|| ui::run(tx), move |buffer, _| {
      request_tx.send(buffer.len()).unwrap();
      response_rx.recv().unwrap();
      
      let out_buffer = out_buffer.lock().unwrap();
      let (ref out_buffer_l, ref out_buffer_r) = *out_buffer;
      buffer.iter_mut().zip(
         unwrap_match!((out_buffer_l, out_buffer_r),
            (Buffer::Sampled(GenericSampledBuffer::F32(l)), Buffer::Sampled(GenericSampledBuffer::F32(r)))
               => l.samples.iter().zip(&r.samples))
      ).for_each(|(dst, (src_l, src_r))| *dst = [*src_l, *src_r]);
   }).unwrap();
}