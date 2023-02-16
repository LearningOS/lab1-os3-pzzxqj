core::arch::global_asm!(
    r#"
    .section .text
    .global __init_task_time
__init_task_time:
    call init_task_time
    j __restore    
"#
);

use super::TaskContext;

core::arch::global_asm!(include_str!("switch.S"));

extern "C" {
    pub fn __switch(current_task_cs_ptr: *mut TaskContext, next_task_cx_ptr: *const TaskContext);
}
