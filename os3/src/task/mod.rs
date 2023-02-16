mod context;
mod switch;
mod task;

use crate::{
    config::MAX_APP_NUM,
    loader::{get_num_app, init_app_cx},
    sync::SyncRefCell,
    syscall::TaskInfo,
    task::switch::__switch,
    timer::get_time_us,
};
pub use context::TaskContext;
use lazy_static::lazy_static;
pub use task::TaskStatus;

use task::TaskControBlock;

pub struct TaskManager {
    num_app: usize,
    inner: SyncRefCell<TaskManagerInner>,
}

struct TaskManagerInner {
    tasks: [TaskControBlock; MAX_APP_NUM],
    current_task: usize,
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [TaskControBlock::new(); MAX_APP_NUM];
        for (i, t) in tasks.iter_mut().enumerate().take(num_app) {
            t.task_cx = TaskContext::goto_init_time(init_app_cx(i));
            t.task_status = TaskStatus::Ready;
        }
        TaskManager {
            num_app,
            inner: unsafe {
                SyncRefCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}

impl TaskManager {
    fn run_first_task(&self) -> ! {
        let inner = self.inner.borrow_mut();
        let mut _unused = TaskContext::zero_init();
        let mut task0 = inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        drop(inner);
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    fn mark_current_suspended(&self) {
        let mut inner = self.inner.borrow_mut();

        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    /// Switch task, but doesn't modify current task state
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.borrow_mut();
            let current = inner.current_task;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            inner.current_task = next;
            inner.tasks[next].task_status = TaskStatus::Running;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            panic!("All applications completed!");
        }
    }

    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.borrow_mut();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|&id| inner.tasks[id].task_status == TaskStatus::Ready)
    }

    fn count_syscall(&self, syscall_id: usize) {
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current].count_syscall(syscall_id);
    }

    fn get_current_task_info(&self) -> TaskInfo {
        let inner = self.inner.borrow_mut();
        let task = inner.tasks[inner.current_task];
        TaskInfo {
            time: (get_time_us() - inner.tasks[inner.current_task].start_time_us) / 1000,
            status: task.task_status,
            syscall_times: task.syscall_times,
        }
    }

    fn init_current_task_start_time(&self) {
        let inner = self.inner.borrow_mut();
        let mut task = inner.tasks[inner.current_task];
        if task.start_time_us == 0 {
            task.init_start_time();
        }
    }
}

/// Run the first task in task list.
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

/// Switch current `Running` task to the task we have found,
/// or there is no `Ready` task and we can exit with all applications completed
fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

/// Change the status of current `Running` task into `Ready`.
fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

/// Change the status of current `Running` task into `Exited`.
fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

/// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

/// Exit the current 'Running' task and run the next task in task list.
pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

// LAB1: Public functions implemented here provide interfaces.
// You may use TASK_MANAGER member functions to handle requests.
pub fn get_task_info() -> TaskInfo {
    TASK_MANAGER.get_current_task_info()
}

pub fn count_syscall(syscall_id: usize) {
    TASK_MANAGER.count_syscall(syscall_id);
}

#[no_mangle]
pub fn init_task_time() {
    TASK_MANAGER.init_current_task_start_time();
}
