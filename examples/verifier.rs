use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, LineWriter, Write};

use ik_rs::core::ik_segmenter::TokenMode;
use ik_rs::IkTokenizer;
use tantivy::tokenizer::*;

pub fn tokenize_text(text: &str, mode: TokenMode) -> Vec<String> {
    let tokenizer = IkTokenizer::new(mode);
    let mut token_stream = tokenizer.token_stream(text);
    let mut token_text = Vec::new();
    while let Some(token) = token_stream.next() {
        token_text.push(token.text.clone());
    }
    token_text
}

fn main() {
    // simple command line interface
    // or we can use enviroment variable instead
    let args: Vec<_> = std::env::args().collect();
    assert!(
        args.len() == 3,
        "should only specify the input file and output file"
    );
    let input_filename = &args[1];
    let output_filename = &args[2];
    let input_file = File::open(input_filename).expect("input file not exists");
    let lines = io::BufReader::new(input_file).lines();

    let mut opts = OpenOptions::new();
    opts.create(true).write(true);
    let output_file = opts.open(output_filename).expect("output file not exists");
    let mut writer = LineWriter::new(output_file);

    for line in lines {
        let mut res = tokenize_text(&line.unwrap(), TokenMode::INDEX);
        res.push("\n".to_string());
        writer.write_all(res.join(",").as_bytes()).unwrap();
    }
    writer.flush().unwrap();
}
