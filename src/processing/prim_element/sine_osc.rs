use super::PrimElementProcessor;

use super::{
   InputNo, OutputNo, super::flow_store::FlowStore,
   super::processor::{Buffer, GenericSampledBuffer},
};
use linear_map::LinearMap;
use std::cell::RefMut;

pub struct SineOscF32 {
   current_phase: f32,
   fs: u64,
}
impl SineOscF32 {
   pub(super) fn new(f_nyq: u64) -> Self {
      Self{current_phase: 0.0, fs: f_nyq*2}
   }
}
impl PrimElementProcessor for SineOscF32 {
   fn compute_outplace(
      &mut self, output: &mut LinearMap<OutputNo, RefMut<Buffer>>, input: &LinearMap<InputNo, RefMut<Buffer>>,
      buffer_sz: usize, _: &FlowStore,
   ) {
      if let Some(y) = output.get_mut(&OutputNo(0)) {
         let y = unwrap_match!(&mut **y, Buffer::Sampled(GenericSampledBuffer::F32(y)) => y);
         y.update_size(buffer_sz);

         if let Some(f) = input.get(&InputNo(0)) {
            let f = unwrap_match!(&**f, Buffer::Sampled(GenericSampledBuffer::F32(f)) => f);

            y.samples.iter_mut().zip(&f.samples).for_each(|(y,f)| {
               self.current_phase += f / self.fs as f32;
               *y = (self.current_phase * std::f32::consts::TAU).sin();
            });
         }
         else { y.clear(); }
      }
   }
}