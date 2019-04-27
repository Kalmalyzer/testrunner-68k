#[macro_use]
extern crate lazy_static;

use std::env;

use amiga_hunk_parser::HunkParser;

mod amigahunk;
mod musashi;
mod testcases;

use amigahunk::get_test_cases;

use musashi::run_test_cases;

use testcases::TestResult;

fn pretty_print_results(test_results: &Vec<TestResult>) {
    for test_result in test_results {
        println!("{}: {}", test_result.name, test_result.success);
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: testrunner-m68k <executable name>");
        return;
    }

    let hunks = HunkParser::parse_file(&args[1]).unwrap();
    let test_cases = get_test_cases(&hunks);
    let test_results = run_test_cases(&hunks, &test_cases);
    pretty_print_results(&test_results);
}

