use super::{flow::Flow, store::{FlowStore, FlowId}, Type, InputNo, OutputNo, processor::{ProcessorStore, Buffer}};

use linear_map::{LinearMap, set::LinearSet};
use std::cell::{Ref, RefMut};
use std::ops::{Deref, DerefMut};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Element {
   Flow(FlowId),
}
impl Element {
   pub fn input_types(&self, store: &FlowStore) -> LinearMap<InputNo, Type> {
      match self {
         Element::Flow(flow) => store.get(*flow).unwrap().input_types().collect()
      }
   }

   pub fn output_types(&self, store: &FlowStore) -> LinearMap<OutputNo, Type> {
      match self {
         Element::Flow(flow) => store.get(*flow).unwrap().output_types().collect()
      }
   }

   pub fn input_nos(&self, store: &FlowStore) -> LinearSet<InputNo> {
      self.input_types(store).keys().copied().collect()
   }

   pub fn output_nos(&self, store: &FlowStore) -> LinearSet<OutputNo> {
      self.output_types(store).keys().copied().collect()
   }

   pub fn compute_outplace<BufferRefMut>(
      &self, output: &mut LinearMap<OutputNo, BufferRefMut>, input: &LinearMap<InputNo, BufferRefMut>, buffer_sz: usize,
      flow_store: &FlowStore, processor_store: &ProcessorStore
   ) where BufferRefMut: DerefMut<Target=Buffer>
   {
      match self {
         Element::Flow(flow_id) => processor_store.compute_outplace(*flow_id, output, input, buffer_sz, flow_store)
      }
   }
}