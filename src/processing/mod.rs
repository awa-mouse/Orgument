macro_rules! enumerate_prim_types {
   {$callback:ident ! ($($prefix:expr),*)} => {
      $callback!($($prefix),* F32, C32, U32, I32);
   };

   {$callback:ident} => {
      enumerate_prim_types!{$callback!()}
   }
}

pub mod flow;
pub mod element;
pub mod flow_store;
pub mod store;
pub mod processor;
pub mod prim_element;

pub use store::Store;
pub use flow_store::FlowId;
pub use processor::Buffer;


#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct InputNo(u32);
impl From<u32> for InputNo {
   fn from(n: u32) -> Self { InputNo(n) }
}
impl Into<u32> for InputNo {
   fn into(self) -> u32 { self.0 }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct OutputNo(u32);
impl From<u32> for OutputNo {
   fn from(n: u32) -> Self { OutputNo(n) }
}
impl Into<u32> for OutputNo {
   fn into(self) -> u32 { self.0 }
}


#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Type {
   Sampled{ty: PrimType, f_nyq: u64},
   Event(PrimType),
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum PrimType {
   F32,
   C32,
   U32,
   I32,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Value {
   F32(eq_float::F32),
   C32(num::complex::Complex<eq_float::F32>),
   U32(u32),
   I32(i32),
}
impl Value {
   pub fn Type(&self) -> PrimType {
      match self {
         Self::F32(_) => PrimType::F32,
         Self::C32(_) => PrimType::C32,
         Self::U32(_) => PrimType::U32,
         Self::I32(_) => PrimType::I32,
      }
   }
}