use super::{
   PrimElementProcessor,
   InputNo, OutputNo, super::flow_store::FlowStore,
   super::processor::Buffer,
};
use crate::processing::Value;
use linear_map::LinearMap;
use std::cell::RefMut;

pub struct Constant {
   value: Value,
}
impl Constant {
   pub(super) fn new(value: Value) -> Self {
      Self{value}
   }
}
impl PrimElementProcessor for Constant {
   fn compute_outplace(
      &mut self, output: &mut LinearMap<OutputNo, RefMut<Buffer>>, _input: &LinearMap<InputNo, RefMut<Buffer>>,
      buffer_sz: usize, _: &FlowStore,
   ) {
      if let Some(y) = output.get_mut(&OutputNo(0)) {
         let y = unwrap_match!(&mut **y, Buffer::Sampled(y) => y);
         y.update_size(buffer_sz);
         y.fill(self.value.clone());
      }
   }
}