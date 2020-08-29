use super::{
   flow::{Flow, Node, EdgeIx},
   Type, PrimType, Value,
   flow_store::{FlowStore, FlowId},
   prim_element::{PrimElement, PrimElementProcessor, mk_prim_element_processor},
   OutputNo, InputNo
};

use intmap::IntMap;
use std::cell::{RefCell, RefMut};
use linear_map::LinearMap;
use std::ops::{Deref, DerefMut};
use std::collections::HashMap;

pub struct ProcessorStore {
   processors: IntMap<Processor>,
   prim_element_processors: HashMap<PrimElement, (Box<RefCell<dyn PrimElementProcessor + Send>>, u32)>,
}
impl ProcessorStore {
   pub(super) fn new() -> Self{ Self{processors: IntMap::new(), prim_element_processors: HashMap::new()} }

   pub(super) fn compute_outplace<BufferRef, BufferRefMut>(
      &self, flow_id: FlowId, output: &mut LinearMap<OutputNo, BufferRefMut>, input: &LinearMap<InputNo, BufferRef>, buffer_sz: usize,
      flow_store: &FlowStore,
   ) where BufferRef: Deref<Target=Buffer>, BufferRefMut: DerefMut<Target=Buffer>
   {
      let flow = &flow_store[flow_id];
      let processor = self.processor(flow_id);
      processor.compute_outplace(output, input, buffer_sz, flow, flow_store, self)
   }

   pub(super) fn compute_outplace_prim(
      &self, prim_element_id: PrimElement,
      output: &mut LinearMap<OutputNo, RefMut<Buffer>>, input: &LinearMap<InputNo, RefMut<Buffer>>,
      buffer_sz: usize, flow_store: &FlowStore,
   ) {
      self.prim_element_processors[&prim_element_id].0.borrow_mut().compute_outplace(output, input, buffer_sz, flow_store);
   }

   pub fn processor(&self, flow_id: FlowId) -> &Processor {
      self.processors.get(flow_id.0).unwrap()
   }

   pub(super) fn processor_mut(&mut self, flow_id: FlowId) -> &mut Processor {
      self.processors.get_mut(flow_id.0).unwrap()
   }

   pub(super) fn add(&mut self, flow_id: FlowId) {
      let result = self.processors.insert(flow_id.0, Processor::new());
      assert!(result);
   }

   pub(super) fn remove(&mut self, flow_id: FlowId) -> Processor {
      self.processors.remove(flow_id.0).unwrap()
   }

   pub(super) fn use_prim_element_processor(&mut self, prim_element_id: PrimElement) {
      if let Some(pep) = self.prim_element_processors.get_mut(&prim_element_id) {
         pep.1 += 1;
      }
      else {
         self.prim_element_processors.insert(prim_element_id, (mk_prim_element_processor(prim_element_id), 0));
      }
   }

   pub(super) fn free_prim_element_processor(&mut self, prim_element_id: PrimElement) {
      let rc = &mut self.prim_element_processors.get_mut(&prim_element_id).unwrap().1;
      if *rc == 0 {
         self.prim_element_processors.remove(&prim_element_id);
      }
      else {
         *rc -= 1;
      }
   }
}

pub struct Processor {
   buffers: IntMap<RefCell<Buffer>>,
}
impl Processor {
   pub(super) fn new() -> Self {
      Self{buffers: IntMap::new()}
   }

   fn compute_outplace<BufferRef, BufferRefMut>(
      &self, output: &mut LinearMap<OutputNo, BufferRefMut>, input: &LinearMap<InputNo, BufferRef>, buffer_sz: usize,
      flow: &Flow, flow_store: &FlowStore, processor_store: &ProcessorStore
   ) where BufferRef: Deref<Target=Buffer>, BufferRefMut: DerefMut<Target=Buffer>
   {
      Self::check_buffer_types(input, flow.input_types());
      Self::check_buffer_types(output, flow.output_types());

      Self::check_buffer_sz(input, buffer_sz);
      
      output.iter_mut().for_each(|(_,x)| x.update_size(buffer_sz));

      flow.visit_order().for_each(|&node_ix| {
         match flow.node(node_ix) {
            Node::Element(e) => {
               let in_buffers =
                  e.input_nos(flow_store).into_iter().filter_map(|input_no| {
                     let mut edges = flow.input_edges(node_ix, input_no);
                     if let Some(head) = edges.next() {
                        let tail_edges = edges;
                        let mut head_buffer = self.buffer(head).borrow_mut();
                        tail_edges.for_each(|tail| head_buffer.merge(&self.buffer(tail).borrow()));
                        Some((input_no, head_buffer))
                     }
                     else { None }
                  }).collect();

               let (mut direct_out_buffers, mut rest_edges): (LinearMap<_,_>, Vec<_>) =
                  e.output_nos(flow_store).iter().filter_map(|&output_no| {
                     let mut edges = flow.output_edges(node_ix, output_no);
                     if let Some(head) = edges.next() {
                        let mut head_buffer = self.buffer(head).borrow_mut();
                        head_buffer.update_size(buffer_sz);
                        Some(((output_no, head_buffer), edges))
                     }
                     else { None }
                  }).unzip();
               
               e.compute_outplace(&mut direct_out_buffers, &in_buffers, buffer_sz, flow_store, processor_store);

               rest_edges.iter_mut().zip(&direct_out_buffers).for_each(|(rest_edges, (_,direct_out_buffer))|
                  rest_edges.for_each(|edge_ix| {
                     let mut rest_buffer = self.buffer(edge_ix).borrow_mut();
                     rest_buffer.update_size(buffer_sz);
                     rest_buffer.clone_from(direct_out_buffer);
                  })
               );
            }
            Node::Input{no, ..} => {
               let buffers = flow.output_edges(node_ix, OutputNo(0)).map(|edge_ix| self.buffer(edge_ix).borrow_mut());
               if let Some(in_buffer) = input.get(no) {
                  buffers.for_each(|mut buffer| buffer.clone_from(in_buffer));
               }
               else {
                  buffers.for_each(|mut buffer| buffer.clear());
               }
            }
            Node::Output{no, ..} =>
               if let Some(out_buffer) = output.get_mut(no) {
                  out_buffer.update_size(buffer_sz);

                  let mut edges = flow.input_edges(node_ix, InputNo(0));
                  if let Some(head) = edges.next() {
                     let tail = edges;
                     out_buffer.clone_from(&self.buffer(head).borrow());
                     tail.for_each(|edge_ix| out_buffer.merge(&self.buffer(edge_ix).borrow()));
                  }
                  else {
                     out_buffer.clear();
                  }
               }
         }
      });
   }

   fn check_buffer_types<T,U,I>(buffer: &LinearMap<T,U>, types: I)
      where T: Eq, U: Deref<Target=Buffer>, I: IntoIterator<Item = (T, Type)>
   {
      debug_assert!( types.into_iter().all(|(no, ty)| buffer.get(&no).map(|x| x.test_type(ty)).unwrap_or(true)) );
   }

   fn check_buffer_sz<T,U>(buffers: &LinearMap<T,U>, sz: usize) where T: Eq, U: Deref<Target=Buffer> {
      debug_assert!( buffers.values().filter_map(|x| option_match!(x.deref(), Buffer::Sampled(x) => x)).all(|x| x.len() == sz) );
   }

   fn buffer(&self, edge_ix: EdgeIx) -> &RefCell<Buffer> {
      self.buffers.get(edge_ix.index() as u64).unwrap()
   }

   pub(super) fn add_edge(&mut self, edge_ix: EdgeIx, ty: Type) {
      let result = self.buffers.insert(edge_ix.index() as u64, RefCell::new(Buffer::new(ty)));
      assert!(result);
   }

   pub(super) fn remove_edge(&mut self, edge_ix: EdgeIx) {
      self.buffers.remove(edge_ix.index() as u64).unwrap();
   }
}


#[derive(Clone, Debug)]
pub enum Buffer {
   Sampled(GenericSampledBuffer),
   Event(GenericEventBuffer),
}
impl Buffer {
   pub fn new(ty: Type) -> Self {
      match ty {
         Type::Sampled{ty, ..} => Self::Sampled(GenericSampledBuffer::new(ty)),
         Type::Event(ty) => Self::Event(GenericEventBuffer::new(ty)),
      }
   }

   fn test_type(&self, ty: Type) -> bool {
      match (self, ty) {
         (Self::Sampled(buf), Type::Sampled{ty, ..}) if buf.ty() == ty => true,
         (Self::Event(buf), Type::Event(ty)) if buf.ty() == ty => true,
         _ => false,
      }
   }

   fn update_size(&mut self, sz: usize) {
      if let Self::Sampled(buf) = self {
         buf.update_size(sz)
      }
   }

   fn merge(&mut self, other: &Self) {
      match (self, other) {
         (Self::Sampled(x), Self::Sampled(y)) => x.merge(y),
         (Self::Event(x), Self::Event(y)) => x.merge(y),
         _ => unreachable!(),
      }
   }

   pub(super) fn clear(&mut self) {
      match self {
         Self::Sampled(x) => x.clear(),
         Self::Event(x) => x.clear(),
      }
   }
}

#[derive(Clone, Debug)]
pub enum GenericSampledBuffer {
   F32(SampledBuffer<f32>),
   C32(SampledBuffer<num::complex::Complex<f32>>),
   U32(SampledBuffer<u32>),
   I32(SampledBuffer<i32>),
}
macro_rules! impl_generic_sampled_buffer {
   ($($prim_type:ident),*) => {
      impl GenericSampledBuffer {
         fn new(ty: PrimType) -> Self {
            match ty {
               $( PrimType::$prim_type => Self::$prim_type(SampledBuffer::new()), )*
            }
         }

         fn resize(&mut self, sz: usize) {
            match self {
               $( Self::$prim_type(buf) => buf.resize(sz), )*
            }
         }

         fn ty(&self) -> PrimType {
            match self {
               $( Self::$prim_type(..) => PrimType::$prim_type, )*
            }
         }

         fn len(&self) -> usize {
            match self {
               $( Self::$prim_type(buf) => buf.len(), )*
            }
         }

         pub(super) fn update_size(&mut self, sz: usize) {
            match self {
               $( Self::$prim_type(buffer) => buffer.update_size(sz), )*
            }
         }

         fn merge(&mut self, other: &Self) {
            match (self, other) {
               $( (Self::$prim_type(x), Self::$prim_type(y)) => x.merge(y), )*
               _ => unreachable!(),
            }
         }

         pub(super) fn clear(&mut self) {
            match self {
               $( Self::$prim_type(buf) => buf.clear(), )*
            }
         }

         pub(super) fn fill(&mut self, value: Value) {
            match (self, value) {
               (Self::F32(x), Value::F32(y)) => x.fill(y.into()),
               (Self::C32(x), Value::C32(y)) => x.fill(num::complex::Complex::new(y.re.into(), y.im.into())),
               (Self::U32(x), Value::U32(y)) => x.fill(y),
               (Self::I32(x), Value::I32(y)) => x.fill(y),
               _ => unreachable!(),
            }
         }
      }
   }
}
enumerate_prim_types!{impl_generic_sampled_buffer}

#[derive(Clone, Debug)]
pub struct SampledBuffer<T> {
   pub samples: Vec<T>,
}
impl<T> SampledBuffer<T> {
   fn new() -> Self { Self{samples: Vec::new()} }

   fn resize(&mut self, sz: usize) where T: Default + Clone {
      self.samples.resize(sz, Default::default())
   }

   fn len(&self) -> usize { self.samples.len() }

   pub(super) fn update_size(&mut self, sz: usize) where T: Default + Clone {
      if self.len() != sz {
         self.resize(sz)
      }
   }

   fn merge<'a>(&mut self, other: &'a Self) where T: std::ops::AddAssign<&'a T> {
      self.samples.iter_mut().zip(&other.samples).for_each(|(x,y)| *x += y);
   }

   pub(super) fn clear(&mut self) where T: Default {
      self.samples.iter_mut().for_each(|x| *x = Default::default());
   }

   fn fill(&mut self, value: T) where T: Copy {
      self.samples.iter_mut().for_each(|x| *x = value);
   }
}


#[derive(Clone, Debug)]
pub enum GenericEventBuffer {
   F32(EventBuffer<f32>),
   C32(EventBuffer<num::complex::Complex<f32>>),
   U32(EventBuffer<u32>),
   I32(EventBuffer<i32>),
}
macro_rules! impl_generic_event_buffer {
   ($($prim_type:ident),*) => {
      impl GenericEventBuffer {
         fn new(ty: PrimType) -> Self {
            match ty {
               $( PrimType::$prim_type => Self::$prim_type(EventBuffer::new()), )*
            }
         }

         fn ty(&self) -> PrimType {
            match self {
               $( Self::$prim_type(..) => PrimType::$prim_type, )*
            }
         }

         fn merge(&mut self, other: &Self) {
            match (self, other) {
               $( (Self::$prim_type(x), Self::$prim_type(y)) => x.merge(y), )*
               _ => unreachable!(),
            }
         }

         pub(super) fn clear(&mut self) {
            match self {
               $( Self::$prim_type(buf) => buf.clear(), )*
            }
         }
      }
   }
}
enumerate_prim_types!{impl_generic_event_buffer}

#[derive(Clone, Debug)]
pub struct EventBuffer<T> {
   events: Vec<Event<T>>,
}
impl<T> EventBuffer<T> {
   fn new() -> Self { Self{events: Vec::new()} }

   fn merge(&mut self, other: &Self) where T: Clone {
      self.events.extend(other.events.iter().cloned())
   }

   pub(super) fn clear(&mut self) {
      self.events.clear();
   }
}

#[derive(Clone, Debug)]
struct Event<T> {
   time: u64,
   value: T,
}