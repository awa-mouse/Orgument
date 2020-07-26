mod parser_org;
mod ui;
mod audio;

fn main() {
   audio::run_with(|tx| ui::run(tx)).unwrap();
}