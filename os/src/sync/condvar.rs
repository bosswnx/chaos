// //! Conditian variable

// use crate::sync::UPSafeCell;
// use crate::task::{block_current_and_run_next, current_task, wakeup_task, TaskControlBlock};
// use alloc::{collections::VecDeque, sync::Arc};

// use super::mutex::MutexSupport;

// /// Condition variable structure
// pub struct Condvar {
//     /// Condition variable inner
//     pub inner: UPSafeCell<CondvarInner>,
// }

// pub struct CondvarInner {
//     pub wait_queue: VecDeque<Arc<TaskControlBlock>>,
// }

// impl Condvar {
//     /// Create a new condition variable
//     pub fn new() -> Self {
//         trace!("kernel: Condvar::new");
//         Self {
//             inner: unsafe {
//                 UPSafeCell::new(CondvarInner {
//                     wait_queue: VecDeque::new(),
//                 })
//             },
//         }
//     }

//     /// Signal a task waiting on the condition variable
//     pub fn signal(&self) {
//         let mut inner = self.inner.exclusive_access(file!(), line!());
//         if let Some(task) = inner.wait_queue.pop_front() {
//             wakeup_task(task);
//         }
//     }

//     /// blocking current task, let it wait on the condition variable
//     pub fn wait(&self, mutex: Arc<dyn MutexSupport>) {
//         trace!("kernel: Condvar::wait_with_mutex");
//         mutex.unlock();
//         let mut inner = self.inner.exclusive_access(file!(), line!());
//         inner.wait_queue.push_back(current_task().unwrap());
//         drop(inner);
//         block_current_and_run_next();
//         mutex.lock();
//     }
// }
