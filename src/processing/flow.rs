use super::{InputNo, OutputNo, Type, element, store::FlowStore};

use std::collections::BTreeMap;
use linear_map::LinearMap;
use daggy::stable_dag::{StableDag, NodeIndex, EdgeIndex, Walker};

pub type NodeIx = NodeIndex<u32>;
pub type EdgeIx = EdgeIndex<u32>;

#[derive(Clone, Debug)]
pub struct Flow {
   graph: StableDag<Node, Edge>,
   visit_order: Vec<NodeIx>,
   input_types: BTreeMap<InputNo, Type>,
   output_types: BTreeMap<OutputNo, Type>,
}

impl Flow {
   pub fn new() -> Self {
      Self {
         graph: StableDag::new(),
         visit_order: Vec::new(),
         input_types: BTreeMap::new(),
         output_types: BTreeMap::new(),
      }
   }

   pub fn add_element(&mut self, element: element::Element) -> NodeIx {
      self.graph.add_node(Node::Element(element))
   }

   pub fn add_input(&mut self, ty: Type) -> NodeIx {
      let no = next_no(&self.input_types);
      self.input_types.insert(no, ty);
      self.graph.add_node(Node::Input{no, ty})
   }

   pub fn add_output(&mut self, ty: Type) -> NodeIx {
      let no = next_no(&self.output_types);
      self.output_types.insert(no, ty);
      self.graph.add_node(Node::Output{no, ty})
   }

   pub fn remove_node(&mut self, node_ix: NodeIx) -> Option<Node> {
      self.graph.remove_node(node_ix).map(|node| {
         self.update_visit_order();
         match node {
            Node::Input{no, ..} => {self.input_types.remove(&no);}
            Node::Output{no, ..} => {self.output_types.remove(&no);}
            _ => {}
         }
         node
      })
   }

   pub fn add_edge(&mut self, source_node: NodeIx, output_no: OutputNo, target_node: NodeIx, input_no: InputNo) -> Result<EdgeIx, WouldCycle> {
      self.graph.add_edge(source_node, target_node, Edge{output_no, input_no})
         .map(|edge| {
            self.update_visit_order();
            edge
         })
         .map_err(|_| WouldCycle)
   }

   pub fn remove_edge(&mut self, edge: EdgeIx) -> bool {
      if self.graph.remove_edge(edge).is_some() {
         self.update_visit_order();
         true
      }
      else { false }
   }

   pub fn input_types<'a>(&'a self) -> impl Iterator<Item = (InputNo, Type)> + 'a {
      self.input_types.iter().map(|(k,v)| (*k,*v))
   }

   pub fn output_types<'a>(&'a self) -> impl Iterator<Item = (OutputNo, Type)> + 'a {
      self.output_types.iter().map(|(k,v)| (*k,*v))
   }

   pub fn input_nos<'a>(&'a self) -> impl Iterator<Item = InputNo> + 'a {
      self.input_types.keys().copied()
   }

   pub fn output_nos<'a>(&'a self) -> impl Iterator<Item = OutputNo> + 'a {
      self.output_types.keys().copied()
   }

   fn update_visit_order(&mut self) {
      self.visit_order = daggy::petgraph::algo::toposort(self.graph.graph(), None).unwrap();
   }

   pub(super) fn visit_order(&self) -> impl Iterator<Item=&NodeIx> { self.visit_order.iter() }

   pub fn node(&self, node_ix: NodeIx) -> &Node { &self.graph[node_ix] }

   pub fn input_edges_with_node<'a>(&'a self, node_ix: NodeIx, input_no: InputNo) -> impl Iterator<Item = (EdgeIx, NodeIx)> + 'a {
      self.graph.parents(node_ix).iter(&self.graph).filter(move |(edge,_)| self.graph[*edge].input_no == input_no)
   }

   pub fn input_edges<'a>(&'a self, node_ix: NodeIx, input_no: InputNo) -> impl Iterator<Item = EdgeIx> + 'a {
      self.input_edges_with_node(node_ix, input_no).map(|(edge_ix,_)| edge_ix)
   }

   pub fn output_edges_with_node<'a>(&'a self, node_ix: NodeIx, output_no: OutputNo) -> impl Iterator<Item = (EdgeIx, NodeIx)> + 'a {
      self.graph.parents(node_ix).iter(&self.graph).filter(move |(edge,_)| self.graph[*edge].output_no == output_no)
   }

   pub fn output_edges<'a>(&'a self, node_ix: NodeIx, output_no: OutputNo) -> impl Iterator<Item = EdgeIx> + 'a {
      self.output_edges_with_node(node_ix, output_no).map(|(edge_ix,_)| edge_ix)
   }
}


fn next_no<T: From<u32> + Into<u32> + Ord + Copy>(types: &BTreeMap<T, Type>) -> T {
   assert!(types.len() <= std::u32::MAX as usize);

   if types.is_empty() { 0.into() }
   else {
      let mut next;
      while {
         next = T::from((*types.keys().last().unwrap()).into().wrapping_add(1));
         types.contains_key(&next)
      } {}
      next
   }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct WouldCycle;

#[derive(Clone, Debug)]
pub enum Node {
   Element(element::Element),
   Input{no: InputNo, ty: Type},
   Output{no: OutputNo, ty: Type},
}
impl Node {
   pub fn input_types(&self, store: &FlowStore) -> LinearMap<InputNo, Type> {
      match self {
         Node::Element(e) => e.input_types(store),
         Node::Input{..} => LinearMap::new(),
         Node::Output{ty, ..} => std::iter::once((InputNo(0), *ty)).collect(),
      }
   }

   pub fn output_types(&self, store: &FlowStore) -> LinearMap<OutputNo, Type> {
      match self {
         Node::Element(e) => e.output_types(store),
         Node::Input{ty, ..} => std::iter::once((OutputNo(0), *ty)).collect(),
         Node::Output{..} => LinearMap::new(),
      }
   }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Edge {
   output_no: OutputNo,
   input_no: InputNo,
}