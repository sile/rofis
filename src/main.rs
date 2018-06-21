#[macro_use]
extern crate clap;
extern crate fibers;
extern crate futures;
extern crate rofis;

fn main() {
    let matches = app_from_crate!().get_matches();
}
