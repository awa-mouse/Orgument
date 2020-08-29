mod sine_osc;
mod constant;
mod pure;

use super::{InputNo, OutputNo, Type, Value, PrimType, flow_store::FlowStore, processor::Buffer};
use linear_map::LinearMap;
use std::cell::{RefCell, RefMut};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum PrimElement {
   SineOscF32{f_nyq: u64},
   Constant{value: Value, f_nyq: u64},
}
impl PrimElement {
   pub fn input_types(&self) -> LinearMap<InputNo, Type> {
      match *self {
         PrimElement::SineOscF32{f_nyq} => linear_map!{InputNo(0) => Type::Sampled{ty: PrimType::F32, f_nyq}},
         PrimElement::Constant{..} => LinearMap::new(),
      }
   }

   pub fn output_types(&self) -> LinearMap<OutputNo, Type> {
      match *self {
         PrimElement::SineOscF32{f_nyq} => linear_map!{OutputNo(0) => Type::Sampled{ty: PrimType::F32, f_nyq}},
         PrimElement::Constant{value, f_nyq} => linear_map!{OutputNo(0) => Type::Sampled{ty: value.Type(), f_nyq}},
      }
   }
}

pub(super) fn mk_prim_element_processor(prim_element_id: PrimElement) -> Box<RefCell<dyn PrimElementProcessor + Send>> {
   match prim_element_id {
      PrimElement::SineOscF32{f_nyq} => Box::new(RefCell::new(sine_osc::SineOscF32::new(f_nyq))),
      PrimElement::Constant{value, ..} => Box::new(RefCell::new(constant::Constant::new(value))),
   }
}

pub(super) trait PrimElementProcessor {
   fn compute_outplace(
      &mut self, output: &mut LinearMap<OutputNo, RefMut<Buffer>>, input: &LinearMap<InputNo, RefMut<Buffer>>,
      buffer_sz: usize, flow_store: &FlowStore,
   );
}