mod flow;
mod element;
mod flow_store;
mod store;

macro_rules! enumerate_prim_types {
   {$callback:ident ! ($($prefix:expr),*)} => {
      $callback!($($prefix),* F32, C32, U32, I32);
   };

   {$callback:ident} => {
      enumerate_prim_types!{$callback!()}
   }
}

mod processor;


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