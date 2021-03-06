
use amiga_hunk_parser::Hunk;
use amiga_hunk_parser::RelocInfo32;

use super::super::testcases::TestCase;
use super::context::Context;
use super::musashi_test_result::MusashiTestResult;
use super::simulation_event::SimulationEvent;


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

fn apply_relocations_to_hunk(context: &mut Context, reloc_infos: &Vec<RelocInfo32>, hunk_start_address: u32, hunk_layout: &Vec<u32>) {
    for i in 0..reloc_infos.len() {
        let reloc_info = &reloc_infos[i];
        let target_hunk_id = reloc_info.target;
        let target_hunk_start_address = hunk_layout[target_hunk_id];
        for j in 0 .. reloc_info.data.len() {
            let hunk_relative_relocation_source_address = reloc_info.data[j];
            let absolute_relocation_source_address = hunk_start_address + (hunk_relative_relocation_source_address as u32);
            let original_value = context.read_memory_32(absolute_relocation_source_address);
            let relocated_value = original_value + target_hunk_start_address;
            context.write_memory_32(absolute_relocation_source_address, relocated_value);
        }
    }
}

fn apply_relocations_to_hunks(context: &mut Context, hunks: &Vec<Hunk>, hunk_layout: &Vec<u32>) {
    for i in 0..hunks.len() {
        let hunk = &hunks[i];
        let hunk_start_address = hunk_layout[i];
        if !hunk.reloc_32.is_none() {
            apply_relocations_to_hunk(context, hunk.reloc_32.as_ref().unwrap(), hunk_start_address, hunk_layout);
        }
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
    context.write_memory_16(program_done_ptr, 0x4eb9);   // JSR $f0fff0
    context.write_memory_32(program_done_ptr + 2, 0xf0fff0); // <address>

    context.write_memory_32(stack_ptr, program_done_ptr);

    context.write_memory_32(0, stack_ptr);
    context.write_memory_32(4, test_function_start);
}

fn run_emulator_test(context: &mut Context) -> (bool, Vec<SimulationEvent>) {

    context.run(1024*1024)
}

pub fn run_test_case(hunks: &Vec<Hunk>, test_case: &TestCase) -> MusashiTestResult {

    let memory_size = (1024 * 1024) as u32;
    let stack_size = 4096u32;

    let memory_area_start = 1024u32;
    let _memory_area_end = memory_size - stack_size;

    let program_done_ptr = memory_size - 16;
    let stack_ptr = program_done_ptr - 4;

    let hunk_layout = layout_hunks(&hunks, memory_area_start);

    let mut context = Context::new();
    load_hunks_into_emulator_memory(&mut context, &hunks, &hunk_layout);
    apply_relocations_to_hunks(&mut context, &hunks, &hunk_layout);
    let test_function_start = get_function_start_address(&hunks, &hunk_layout, &test_case.name);
    setup_emulator_init_and_trampoline(&mut context, stack_ptr, program_done_ptr, test_function_start);
    let (success, events) = run_emulator_test(&mut context);
    MusashiTestResult { name: test_case.name.clone(), success: success, events: events }
}

pub fn run_test_cases(hunks: &Vec<Hunk>, test_cases: &Vec<TestCase>) -> Vec<MusashiTestResult> {

    let mut test_results: Vec<MusashiTestResult> = Vec::new();

    for test_case in test_cases {
        test_results.push(run_test_case(&hunks, &test_case));
    }

    test_results
}

#[cfg(test)]
use amiga_hunk_parser::HunkParser;

#[cfg(test)]
use super::simulation_event::{OperationSize, Registers};


#[test]
fn run_successful_test() {
    let hunks = HunkParser::parse_file("testdata/test.successful_test_case.amiga.exe").unwrap();
    let test_case = TestCase { name: "test_TestModule_successfulCase".to_string() };
    let test_result = run_test_case(&hunks, &test_case);
    assert_eq!(true, test_result.success);
    assert_eq!(vec!(SimulationEvent::Passed { registers: Some(Registers { dn: vec!(1, 0, 0, 0, 0, 0, 0, 0), an: vec!(0, 0, 0, 0, 0, 0, 0, 0xfffec), pc: 0xf0fff0, sr: 0x2700 }) } ), test_result.events)
}

#[test]
fn run_failed_test() {
    let hunks = HunkParser::parse_file("testdata/test.failed_test_case.amiga.exe").unwrap();
    let test_case = TestCase { name: "test_TestModule_failedCase".to_string() };
    let test_result = run_test_case(&hunks, &test_case);
    assert_eq!(false, test_result.success);
    assert_eq!(vec!(SimulationEvent::Failed { registers: Some(Registers { dn: vec!(0, 0, 0, 0, 0, 0, 0, 0), an: vec!(0, 0, 0, 0, 0, 0, 0, 0xfffec), pc: 0xf0fff0, sr: 0x2704 }) } ), test_result.events)
}

#[test]
fn run_relocation_test() {
    let hunks = HunkParser::parse_file("testdata/test.reloc32.amiga.exe").unwrap();
    let test_case = TestCase { name: "test_TestModule_reloc32".to_string() };
    let test_result = run_test_case(&hunks, &test_case);
    assert_eq!(true, test_result.success);
    assert_eq!(vec!(SimulationEvent::Passed { registers: Some(Registers { dn: vec!(1, 0, 0, 0, 0, 0, 0, 0), an: vec!(0x430, 0x438, 0, 0, 0, 0, 0, 0xfffec), pc: 0xf0fff0, sr: 0x2700 }) } ), test_result.events)
}

#[test]
fn run_privilege_violation_test() {
    let hunks = HunkParser::parse_file("testdata/test.privilege_violation.amiga.exe").unwrap();
    let test_case = TestCase { name: "test_TestModule_privilegeViolation".to_string() };
    let test_result = run_test_case(&hunks, &test_case);
    assert_eq!(false, test_result.success);
    assert_eq!(vec!(SimulationEvent::PrivilegeViolation { registers: Some(Registers { dn: vec!(0, 0, 0, 0, 0, 0, 0, 0), an: vec!(0, 0, 0, 0, 0, 0, 0, 0), pc: 0x406, sr: 0 }) } ), test_result.events)
}

#[test]
fn run_line_a_exception_test() {
    let hunks = HunkParser::parse_file("testdata/test.line_a_exception.amiga.exe").unwrap();
    let test_case = TestCase { name: "test_TestModule_lineAException".to_string() };
    let test_result = run_test_case(&hunks, &test_case);
    assert_eq!(false, test_result.success);
    assert_eq!(vec!(SimulationEvent::LineAException { registers: Some(Registers { dn: vec!(0, 0, 0, 0, 0, 0, 0, 0), an: vec!(0, 0, 0, 0, 0, 0, 0, 0xfffec), pc: 0x402, sr: 0x2700 }) } ), test_result.events)
}

#[test]
fn run_line_f_exception_test() {
    let hunks = HunkParser::parse_file("testdata/test.line_f_exception.amiga.exe").unwrap();
    let test_case = TestCase { name: "test_TestModule_lineFException".to_string() };
    let test_result = run_test_case(&hunks, &test_case);
    assert_eq!(false, test_result.success);
    assert_eq!(vec!(SimulationEvent::LineFException { registers: Some(Registers { dn: vec!(0, 0, 0, 0, 0, 0, 0, 0), an: vec!(0, 0, 0, 0, 0, 0, 0, 0xfffec), pc: 0x402, sr: 0x2700 }) } ), test_result.events)
}

#[test]
fn run_illegal_instruction_test() {
    let hunks = HunkParser::parse_file("testdata/test.illegal_instruction.amiga.exe").unwrap();
    let test_case = TestCase { name: "test_TestModule_illegalInstruction".to_string() };
    let test_result = run_test_case(&hunks, &test_case);
    assert_eq!(false, test_result.success);
    assert_eq!(vec!(SimulationEvent::IllegalInstruction { registers: Some(Registers { dn: vec!(0, 0, 0, 0, 0, 0, 0, 0), an: vec!(0, 0, 0, 0, 0, 0, 0, 0xfffec), pc: 0x402, sr: 0x2700 }) } ), test_result.events)
}

#[test]
fn run_address_error_test() {
    let hunks = HunkParser::parse_file("testdata/test.address_error.amiga.exe").unwrap();
    let test_case = TestCase { name: "test_TestModule_addressError".to_string() };
    let test_result = run_test_case(&hunks, &test_case);
    assert_eq!(false, test_result.success);
    assert_eq!(vec!(SimulationEvent::AddressError { address: 0x4321, write: false, function_code: 5, registers: Some(Registers { dn: vec!(0, 0, 0, 0, 0, 0, 0, 0), an: vec!(0x4321, 0, 0, 0, 0, 0, 0, 0xfffec), pc: 0x406, sr: 0x2700 }) } ), test_result.events)
}

#[test]
fn run_bus_error_test() {
    let hunks = HunkParser::parse_file("testdata/test.bus_error.amiga.exe").unwrap();
    let test_case = TestCase { name: "test_TestModule_busError".to_string() };
    let test_result = run_test_case(&hunks, &test_case);
    assert_eq!(false, test_result.success);
    assert_eq!(vec!(SimulationEvent::BusError { address: 0xf00000, write: false, operation_size: OperationSize::LongWord, registers: Some(Registers { dn: vec!(0, 0, 0, 0, 0, 0, 0, 0), an: vec!(0xf00000, 0, 0, 0, 0, 0, 0, 0xfffec), pc: 0x408, sr: 0x2700 }) } ), test_result.events)
}
