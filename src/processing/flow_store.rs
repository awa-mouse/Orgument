use super::flow;

use intmap::IntMap;

#[derive(Clone, Debug)]
pub struct FlowStore {
   flows: IntMap<flow::Flow>,
   next_id: u64,
}
impl FlowStore {
   pub(super) fn new() -> FlowStore {
      Self{flows: IntMap::new(), next_id: 0}
   }

   pub fn get(&self, id: FlowId) -> Option<&flow::Flow> { self.flows.get(id.0) }
   pub(super) fn get_mut(&mut self, id: FlowId) -> Option<&mut flow::Flow> { self.flows.get_mut(id.0) }

   pub(super) fn add(&mut self, flow: flow::Flow) -> FlowId {
      let next_id = self.gen_next_id();
      self.flows.insert(next_id, flow);
      FlowId(next_id)
   }

   pub(super) fn alter<T, F: FnOnce(&Self, &mut flow::Flow) -> T>(&mut self, id: FlowId, f: F) -> T {
      let mut flow = self.flows.remove(id.0).unwrap();
      let result = f(&self, &mut flow);
      self.flows.insert(id.0, flow);
      result
   }

   pub(super) fn remove(&mut self, id: FlowId) -> Option<flow::Flow> { self.flows.remove(id.0) }

   fn gen_next_id(&mut self) -> u64 {
      assert!(self.flows.len() <= std::u64::MAX as usize);

      while self.flows.contains_key(self.next_id) {
         self.next_id = self.next_id.wrapping_add(1);
      }
      let id = self.next_id;
      self.next_id = self.next_id.wrapping_add(1);
      id
   }
}

impl std::ops::Index<FlowId> for FlowStore {
   type Output = flow::Flow;
   fn index(&self, id: FlowId) -> &flow::Flow { self.get(id).unwrap() }
}

impl std::ops::IndexMut<FlowId> for FlowStore {
   fn index_mut(&mut self, id: FlowId) -> &mut flow::Flow { self.get_mut(id).unwrap() }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct FlowId(pub(super) u64);