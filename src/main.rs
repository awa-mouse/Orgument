#![feature(const_generics)]

#[macro_use]
extern crate matches2;

mod parser_org;
mod ui;
mod audio;
mod processing;

fn main() {
   return;
   audio::run_with(|tx| ui::run(tx)).unwrap();
}