use clap::Parser;
use std::env;
use std::fs;
mod github_api;
mod json_parser;

fn main() {
    //get file path
    let args: Vec<String> = env::args().collect();
    println!("In file {}", filename);
    //reading the filepah
    let contents = fs::read_to_string(filename)
        .expect("Something went wrong with reading the file with students");
    //debug -> test that file reading works
    println!("With text: \n{}", contents);
}
