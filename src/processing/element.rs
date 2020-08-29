use super::{
   flow::Flow, flow_store::{FlowStore, FlowId},
   Type, PrimType, InputNo, OutputNo, processor::{ProcessorStore, Buffer},
   prim_element::PrimElement,
};

use linear_map::{LinearMap, set::LinearSet};
use std::cell::{Ref, RefMut};
use std::ops::{Deref, DerefMut};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Element {
   Flow(FlowId),
   Prim(PrimElement),
}
impl Element {
   pub fn input_types(&self, store: &FlowStore) -> LinearMap<InputNo, Type> {
      match self {
         Element::Flow(flow) => store.get(*flow).unwrap().input_types().collect(),
         Element::Prim(prim) => prim.input_types(),
      }
   }

   pub fn output_types(&self, store: &FlowStore) -> LinearMap<OutputNo, Type> {
      match self {
         Element::Flow(flow) => store.get(*flow).unwrap().output_types().collect(),
         Element::Prim(prim) => prim.output_types(),
      }
   }

   pub fn input_nos(&self, store: &FlowStore) -> LinearSet<InputNo> {
      self.input_types(store).keys().copied().collect()
   }

   pub fn output_nos(&self, store: &FlowStore) -> LinearSet<OutputNo> {
      self.output_types(store).keys().copied().collect()
   }

   pub fn compute_outplace(
      &self, output: &mut LinearMap<OutputNo, RefMut<Buffer>>, input: &LinearMap<InputNo, RefMut<Buffer>>, buffer_sz: usize,
      flow_store: &FlowStore, processor_store: &ProcessorStore
   )
   {
      match self {
         Element::Flow(flow_id) => processor_store.compute_outplace(*flow_id, output, input, buffer_sz, flow_store),
         Element::Prim(prim_id) => processor_store.compute_outplace_prim(*prim_id, output, input, buffer_sz, flow_store),
      }
   }
}