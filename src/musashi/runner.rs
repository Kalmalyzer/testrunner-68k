
#![allow(nonstandard_style)]
#![allow(dead_code)]

use amiga_hunk_parser::Hunk;

use super::super::testcases::{TestCase, TestResult};
use super::context::Context;

include!(concat!(env!("OUT_DIR"), "/musashi.bindings.rs"));

// Compute start address for each hunk
pub fn layout_hunks(hunks: &Vec<Hunk>, start_address: u32) -> Vec<u32> {

    let mut layout_hunks = Vec::new();

    let mut hunk_start_address = start_address;

    for hunk_index in 0..hunks.len() {

        let hunk = &hunks[hunk_index];
        layout_hunks.push(hunk_start_address);
        hunk_start_address = ((hunk_start_address + (hunk.alloc_size as u32)) + 3) & 0xfffffffc;
    }

    return layout_hunks;
}

fn load_hunk_into_emulator_memory(context: &mut Context, hunk: &Hunk, hunk_start_address: u32) {
    if !hunk.code_data.is_none() {
        let code_data = &hunk.code_data.as_ref().unwrap();
        for offset in 0..code_data.len() {
            context.write_memory_8(hunk_start_address + (offset as u32), code_data[offset]);
        }
    }
}

fn load_hunks_into_emulator_memory(context: &mut Context, hunks: &Vec<Hunk>, hunk_layout: &Vec<u32>) {
    for i in 0..hunks.len() {
        let hunk = &hunks[i];
        let hunk_start_address = hunk_layout[i];
        load_hunk_into_emulator_memory(context, &hunk, hunk_start_address);
    }
}

fn get_function_start_address(hunks: &Vec<Hunk>, hunk_layout: &Vec<u32>, test_case_name: &String) -> u32{
    for i in 0..hunks.len() {
        let hunk = &hunks[i];
        if !hunk.symbols.is_none() {
            for symbol in hunk.symbols.as_ref().unwrap().iter() {
                if symbol.name == *test_case_name {
                    return hunk_layout[i] + symbol.offset;
                }
            }
        }
    }

    panic!("Symbol {} not found", test_case_name);
}

fn setup_emulator_init_and_trampoline(context: &mut Context, stack_ptr: u32, program_done_ptr: u32, test_function_start: u32) {
    context.write_memory_16(program_done_ptr, 0x60fe);           // BRA.S *
    context.write_memory_32(stack_ptr, program_done_ptr);
    context.write_memory_32(0, stack_ptr);
    context.write_memory_32(4, test_function_start);
}

fn run_emulator_test(context: &mut Context) {

    context.reset();
    context.run(1024);
}

fn clear_emulator_test_result() {
}

fn get_emulator_test_result(context: &mut Context, test_case_name: &String) -> TestResult {
    let d0 = context.read_register(m68k_register_t_M68K_REG_D0);
    TestResult { name: test_case_name.clone(), success: d0 != 0 }
}

pub fn run_test_case(hunks: &Vec<Hunk>, test_case: &TestCase) -> TestResult {

    let memory_size = (1024 * 1024) as u32;
    let stack_size = 4096u32;

    let memory_area_start = 1024u32;
    let _memory_area_end = memory_size - stack_size;

    let program_done_ptr = memory_size - 16;
    let stack_ptr = program_done_ptr - 4;

    let hunk_layout = layout_hunks(&hunks, memory_area_start);

    clear_emulator_test_result();
    let mut context = Context::new();
    load_hunks_into_emulator_memory(&mut context, &hunks, &hunk_layout);
    let test_function_start = get_function_start_address(&hunks, &hunk_layout, &test_case.name);
    setup_emulator_init_and_trampoline(&mut context, stack_ptr, program_done_ptr, test_function_start);
    run_emulator_test(&mut context);
    get_emulator_test_result(&mut context, &test_case.name)
}

pub fn run_test_cases(hunks: &Vec<Hunk>, test_cases: &Vec<TestCase>) -> Vec<TestResult> {

    let mut test_results: Vec<TestResult> = Vec::new();

    for test_case in test_cases {
        test_results.push(run_test_case(&hunks, &test_case));
    }

    test_results
}

#[cfg(test)]
use amiga_hunk_parser::HunkParser;

#[test]
fn run_successful_test() {
    let hunks = HunkParser::parse_file("testdata/test.successful_test_case.amiga.exe").unwrap();
    let test_case = TestCase { name: "test_TestModule_successfulCase".to_string() };
    let test_result = run_test_case(&hunks, &test_case);
    assert_eq!(true, test_result.success)
}

#[test]
fn run_failed_test() {
    let hunks = HunkParser::parse_file("testdata/test.failed_test_case.amiga.exe").unwrap();
    let test_case = TestCase { name: "test_TestModule_failedCase".to_string() };
    let test_result = run_test_case(&hunks, &test_case);
    assert_eq!(false, test_result.success)
}