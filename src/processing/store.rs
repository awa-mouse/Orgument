use intmap::IntMap;

use super::flow;

#[derive(Clone, Debug)]
pub struct FlowStore {
   flows: IntMap<flow::Flow>,
   latest_id: u64,
}
impl FlowStore {
   pub fn get(&self, id: FlowId) -> Option<&flow::Flow> { self.flows.get(id.0) }

   pub fn add(&mut self, flow: flow::Flow) -> FlowId {
      let next_id = self.gen_next_id();
      self.flows.insert(next_id, flow);
      FlowId(next_id)
   }

   pub fn remove(&mut self, id: FlowId) -> Option<flow::Flow> { self.flows.remove(id.0) }

   fn gen_next_id(&mut self) -> u64 {
      assert!(self.flows.len() <= std::u64::MAX as usize);

      while {
         self.latest_id = self.latest_id.wrapping_add(1);
         self.flows.contains_key(self.latest_id)
      } {}

      self.latest_id
   }
}

impl std::ops::Index<FlowId> for FlowStore {
   type Output = flow::Flow;
   fn index(&self, id: FlowId) -> &flow::Flow { self.get(id).unwrap() }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct FlowId(pub(super) u64);