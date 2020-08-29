use super::{
   flow_store::{FlowStore, FlowId},
   flow::{self, Flow, NodeIx, EdgeIx, EdgeError},
   processor::{ProcessorStore, Buffer, Processor},
   element,
   Type, OutputNo, InputNo,
};

use daggy::stable_dag::Walker;
use linear_map::LinearMap;
use std::ops::DerefMut;

pub struct Store {
   flows: FlowStore,
   processors: ProcessorStore,
}

impl Store {
   pub fn new() -> Store {
      Self{flows: FlowStore::new(), processors: ProcessorStore::new()}
   }

   pub fn flow_store(&self) -> &FlowStore { &self.flows }
   pub fn processor_store(&self) -> &ProcessorStore { &self.processors }

   pub fn add_flow(&mut self) -> FlowId {
      let flow_id = self.flows.add();
      self.processors.add(flow_id);
      flow_id
   }

   pub fn remove_flow(&mut self, flow_id: FlowId) -> (Flow, Processor) {
      (self.flows.remove(flow_id), self.processors.remove(flow_id))
   }

   pub fn add_element(&mut self, flow_id: FlowId, element: element::Element) -> NodeIx {
      if let element::Element::Prim(pe_id) = element {
         self.processors.use_prim_element_processor(pe_id);
      }
      self.flows[flow_id].add_element(element)
   }

   pub fn add_input(&mut self, flow_id: FlowId, ty: Type) -> (InputNo, NodeIx) {
      self.flows[flow_id].add_input(ty)
   }

   pub fn add_output(&mut self, flow_id: FlowId, ty: Type) -> (OutputNo, NodeIx) {
      self.flows[flow_id].add_output(ty)
   }

   pub fn remove_node(&mut self, flow_id: FlowId, node_ix: NodeIx) -> Option<flow::Node> {
      let processors = &mut self.processors;
      let flow = &mut self.flows[flow_id];
      let graph = flow.graph();
      graph.parents(node_ix).iter(graph).chain(graph.children(node_ix).iter(graph))
         .for_each(|(edge_ix, _)| processors.processor_mut(flow_id).remove_edge(edge_ix));

      flow.remove_node(node_ix).map(|node| {
         if let flow::Node::Element(element::Element::Prim(pe_id)) = node {
            self.processors.free_prim_element_processor(pe_id);
         }
         node
      })
   }

   pub fn add_edge(&mut self, flow_id: FlowId, source: NodeIx, output_no: OutputNo, target: NodeIx, input_no: InputNo)
      -> Result<(EdgeIx, Type), EdgeError>
   {
      let (edge_ix, ty) = self.flows.alter(flow_id, |flow_store, flow| flow.add_edge(source, output_no, target, input_no, flow_store))?;
      self.processors.processor_mut(flow_id).add_edge(edge_ix, ty);
      Ok((edge_ix, ty))
   }

   pub fn remove_edge(&mut self, flow_id: FlowId, edge_ix: EdgeIx) -> bool {
      self.processors.processor_mut(flow_id).remove_edge(edge_ix);
      self.flows[flow_id].remove_edge(edge_ix)
   }

   pub fn compute_outplace<BufferRefMut>(
      &self, flow_id: FlowId, output: &mut LinearMap<OutputNo, BufferRefMut>, input: &LinearMap<InputNo, BufferRefMut>, buffer_sz: usize,
   ) where BufferRefMut: DerefMut<Target=Buffer>
   {
      self.processors.compute_outplace(flow_id, output, input, buffer_sz, &self.flows)
   }
}