
#include <setjmp.h>
#include <stddef.h>
#include "musashi_rust_wrapper.h"

// Taken from m68kcpu.h
#define MODE_WRITE 0

extern RustM68KReadResult rust_m68k_read_memory_8(void* execution_context, uint32_t address);
extern RustM68KReadResult rust_m68k_read_memory_16(void* execution_context, uint32_t address);
extern RustM68KReadResult rust_m68k_read_memory_32(void* execution_context, uint32_t address);

extern RustM68KWriteResult rust_m68k_write_memory_8(void* execution_context, uint32_t address, uint32_t value);
extern RustM68KWriteResult rust_m68k_write_memory_16(void* execution_context, uint32_t address, uint32_t value);
extern RustM68KWriteResult rust_m68k_write_memory_32(void* execution_context, uint32_t address, uint32_t value);

extern RustM68KInstructionHookResult rust_m68k_instruction_hook(void* execution_context);
extern RustM68KInstructionHookResult rust_m68k_exception_privilege_violation_hook(void* execution_context);
extern RustM68KInstructionHookResult rust_m68k_exception_1010_hook(void* execution_context);
extern RustM68KInstructionHookResult rust_m68k_exception_1111_hook(void* execution_context);
extern RustM68KInstructionHookResult rust_m68k_exception_illegal_hook(void* execution_context);
extern RustM68KInstructionHookResult rust_m68k_exception_address_error_hook(void* execution_context, uint32_t address, bool write, uint32_t function_code);


extern int m68k_execute(int num_cycles);
extern void m68k_pulse_reset(void);

static void* s_execution_context = NULL;
static jmp_buf s_abort_execution;

uint32_t m68k_read_memory_8(uint32_t address)
{
    RustM68KReadResult result = rust_m68k_read_memory_8(s_execution_context, address);
    if (!result.continue_simulation)
        longjmp(s_abort_execution, 1);
    return result.value;
}

uint32_t m68k_read_memory_16(uint32_t address)
{
    RustM68KReadResult result = rust_m68k_read_memory_16(s_execution_context, address);
    if (!result.continue_simulation)
        longjmp(s_abort_execution, 1);
    return result.value;
}

uint32_t m68k_read_memory_32(uint32_t address)
{
    RustM68KReadResult result = rust_m68k_read_memory_32(s_execution_context, address);
    if (!result.continue_simulation)
        longjmp(s_abort_execution, 1);
    return result.value;
}

void m68k_write_memory_8(uint32_t address, uint32_t value)
{
    RustM68KWriteResult result = rust_m68k_write_memory_8(s_execution_context, address, value);
    if (!result.continue_simulation)
        longjmp(s_abort_execution, 1);
}

void m68k_write_memory_16(uint32_t address, uint32_t value)
{
    RustM68KWriteResult result = rust_m68k_write_memory_16(s_execution_context, address, value);
    if (!result.continue_simulation)
        longjmp(s_abort_execution, 1);
}

void m68k_write_memory_32(uint32_t address, uint32_t value)
{
    RustM68KWriteResult result = rust_m68k_write_memory_32(s_execution_context, address, value);
    if (!result.continue_simulation)
        longjmp(s_abort_execution, 1);
}

void m68k_instruction_hook()
{
    RustM68KInstructionHookResult result = rust_m68k_instruction_hook(s_execution_context);
    if (!result.continue_simulation)
        longjmp(s_abort_execution, 1);
}

void m68k_exception_privilege_violation_hook()
{
    RustM68KInstructionHookResult result = rust_m68k_exception_privilege_violation_hook(s_execution_context);
    if (!result.continue_simulation)
        longjmp(s_abort_execution, 1);
}

void m68k_exception_1010_hook()
{
    RustM68KInstructionHookResult result = rust_m68k_exception_1010_hook(s_execution_context);
    if (!result.continue_simulation)
        longjmp(s_abort_execution, 1);
}

void m68k_exception_1111_hook()
{
    RustM68KInstructionHookResult result = rust_m68k_exception_1111_hook(s_execution_context);
    if (!result.continue_simulation)
        longjmp(s_abort_execution, 1);
}

void m68k_exception_illegal_hook()
{
    RustM68KInstructionHookResult result = rust_m68k_exception_illegal_hook(s_execution_context);
    if (!result.continue_simulation)
        longjmp(s_abort_execution, 1);
}

void m68k_exception_address_error_hook(uint32_t address, uint32_t write_mode, uint32_t function_code)
{
    RustM68KInstructionHookResult result = rust_m68k_exception_address_error_hook(s_execution_context, address, (write_mode == MODE_WRITE ? 1 : 0), function_code);
    if (!result.continue_simulation)
        longjmp(s_abort_execution, 1);
}

void wrapped_m68k_pulse_reset(void* execution_context)
{
    s_execution_context = execution_context;

    if (setjmp(s_abort_execution) == 0)
        m68k_pulse_reset();

    s_execution_context = NULL;
}

int wrapped_m68k_execute(void* execution_context, int num_cycles)
{
    s_execution_context = execution_context;

    if (setjmp(s_abort_execution) == 0)
    {
        int cycles_used = m68k_execute(num_cycles);

        s_execution_context = NULL;
        return cycles_used;
    }
    else
    {
        s_execution_context = NULL;
        return 0;
    }
}
