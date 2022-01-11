use super::{Task, TaskId};
use crate::interrupts;
use alloc::{collections::BTreeMap, sync::Arc, task::Wake, string::String};
use core::task::{Context, Poll, Waker};
use crossbeam_queue::ArrayQueue;

type TaskQueue = Arc<ArrayQueue<TaskId>>;

struct TaskWaker {
    task_id: TaskId,
    task_queue: TaskQueue,
}

impl TaskWaker {
    fn new(task_id: TaskId, task_queue: TaskQueue) -> Self {
        Self {
            task_id,
            task_queue,
        }
    }

    fn create_waker(task_id: TaskId, task_queue: TaskQueue) -> Waker {
        Waker::from(Arc::new(Self::new(task_id, task_queue)))
    }

    fn wake_task(&self) -> Result<(), TaskId> {
        self.task_queue.push(self.task_id)?;
        Ok(())
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        let _ = self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        let _ = self.wake_task();
    }
}

pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: TaskQueue,
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub fn new(queue_limit: usize) -> Self {
        Self {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(queue_limit)),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) -> Result<(), String> {
        let task_id = task.id;
        if let Some(_) = self.tasks.insert(task.id, task) {
            return Err(format!("Task {task_id} already exists"));
        }

        match self.task_queue.push(task_id) {
            Ok(()) => Ok(()),
            Err(task_id) => Err(format!("Failed to queue task {task_id}")) 
        }
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_read_tasks();
            self.sleep_if_idle();
        }
    }

    fn run_read_tasks(&mut self) {
        while let Some(task_id) = self.task_queue.pop() {
            let task = match self.tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue,
            };

            let waker = self
                .waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::create_waker(task_id, self.task_queue.clone()));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(_) => {
                    self.tasks.remove(&task_id);
                    self.waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }

    fn sleep_if_idle(&self) {
        interrupts::disable();

        if self.task_queue.is_empty() {
            interrupts::enable_and_hlt();
            return;
        }

        interrupts::enable();
    }
}
