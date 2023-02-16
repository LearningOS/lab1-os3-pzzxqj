use crate::config::MAX_SYSCALL_NUM;
use crate::timer::get_time_us;

use super::TaskContext;

#[derive(Clone, Copy)]
pub struct TaskControBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub start_time_us: usize,
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
}
impl TaskControBlock {
    pub fn new() -> Self {
        Self {
            task_status: TaskStatus::UnInit,
            task_cx: TaskContext::zero_init(),
            start_time_us: 0,
            syscall_times: [0; MAX_SYSCALL_NUM],
        }
    }

    pub fn init_start_time(&mut self) {
        self.start_time_us = get_time_us();
    }

    pub fn count_syscall(&mut self, syscall_id: usize) {
        if let Some(count) = self.syscall_times.get_mut(syscall_id) {
            *count += 1;
        }
    }
}
#[derive(Clone, Copy, PartialEq)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}
