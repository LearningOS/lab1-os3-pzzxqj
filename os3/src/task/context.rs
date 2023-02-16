#[repr(C)]
#[derive(Clone, Copy)]
pub struct TaskContext {
    ra: usize,
    sp: usize,
    s: [usize; 12],
}

impl TaskContext {
    pub fn zero_init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }

    pub fn goto_init_time(kstack_ptr: usize) -> Self {
        extern "C" {
            fn __init_task_time();
        }
        Self {
            ra: __init_task_time as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }
}
