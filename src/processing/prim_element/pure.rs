use super::PrimElementProcessor;

use super::{
   InputNo, OutputNo, super::flow_store::FlowStore,
   super::processor::{Buffer, GenericSampledBuffer},
};
use linear_map::LinearMap;
use std::cell::RefMut;

pub struct MultiplyF32;
impl MultiplyF32 {
   pub(super) fn new() -> Self { Self }
}
impl PrimElementProcessor for MultiplyF32 {
   fn compute_outplace(
      &mut self, output: &mut LinearMap<OutputNo, RefMut<Buffer>>, input: &LinearMap<InputNo, RefMut<Buffer>>,
      buffer_sz: usize, _: &FlowStore,
   ) {
      if let Some(y) = output.get_mut(&OutputNo(0)) {
         let y = unwrap_match!(&mut **y, Buffer::Sampled(GenericSampledBuffer::F32(y)) => y);
         y.update_size(buffer_sz);

         if let (Some(x0), Some(x1)) = (input.get(&InputNo(0)), input.get(&InputNo(1))) {
            let x0 = unwrap_match!(&**x0, Buffer::Sampled(GenericSampledBuffer::F32(x0)) => x0);
            let x1 = unwrap_match!(&**x1, Buffer::Sampled(GenericSampledBuffer::F32(x1)) => x1);

            y.samples.iter_mut().zip(x0.samples.iter().zip(&x1.samples)).for_each(|(y,(x0,x1))| *y = *x0 * *x1);
         }
         else { y.clear(); }
      }
   }
}

pub struct AddF32;
impl AddF32 {
   pub(super) fn new() -> Self { Self }
}
impl PrimElementProcessor for AddF32 {
   fn compute_outplace(
      &mut self, output: &mut LinearMap<OutputNo, RefMut<Buffer>>, input: &LinearMap<InputNo, RefMut<Buffer>>,
      buffer_sz: usize, _: &FlowStore,
   ) {
      if let Some(y) = output.get_mut(&OutputNo(0)) {
         let y = unwrap_match!(&mut **y, Buffer::Sampled(GenericSampledBuffer::F32(y)) => y);
         y.update_size(buffer_sz);

         if let (Some(x0), Some(x1)) = (input.get(&InputNo(0)), input.get(&InputNo(1))) {
            let x0 = unwrap_match!(&**x0, Buffer::Sampled(GenericSampledBuffer::F32(x0)) => x0);
            let x1 = unwrap_match!(&**x1, Buffer::Sampled(GenericSampledBuffer::F32(x1)) => x1);

            y.samples.iter_mut().zip(x0.samples.iter().zip(&x1.samples)).for_each(|(y,(x0,x1))| *y = *x0 + *x1);
         }
         else { y.clear(); }
      }
   }
}