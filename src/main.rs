#![allow(unused, dead_code)] // Shaked-TODO: delete this

mod parser;

use clap::Parser;
use parser::Categories;

fn main() {
    let mut categories = Categories::parse();
    println!("{categories:?}");
}
