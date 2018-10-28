use std::collections::HashMap;
use std::sync::Arc;

use serde::Serialize;

use super::Config;
use super::Result;
use super::TaskQueue;
use super::config::Backend as BackendConfig;


mod backend;

use self::backend::Backend;
use self::backend::kafka::Kafka;


/// Request a task to be queued for processing
pub struct TaskRequest<Q: TaskQueue> {
    headers: HashMap<String, String>,
    queue: Q,
}

impl<Q: TaskQueue> TaskRequest<Q> {
    /// Attach or update an header to the task.
    pub fn header<S1, S2>(&mut self, header: S1, value: S2)
        where S1: Into<String>,
              S2: Into<String>,
    {
        self.headers.insert(header.into(), value.into());
    }

    /// Attach or update a set of headers to the task.
    pub fn headers(&mut self, headers: HashMap<String, String>) {
        for (key, value) in headers {
            self.headers.insert(key, value);
        }
    }

    /// Create a new task for the given queue and carring the given message.
    pub fn new(queue: Q) -> TaskRequest<Q> {
        TaskRequest {
            headers: HashMap::new(),
            queue,
        }
    }

    /// Access information about the task's queue.
    pub fn queue(&self) -> &Q {
        &self.queue
    }
}


/// Manages task requests to the queue system.
#[derive(Clone)]
pub struct Tasks<Q: TaskQueue>(Arc<Backend<Q>>);

impl<Q: TaskQueue> Tasks<Q> {
    /// Create a new `Tasks` interface to enqueue new tasks.
    pub fn new(config: Config) -> Tasks<Q> {
        let backend = match config.backend {
            BackendConfig::Kafka(backend) => Arc::new(Kafka::new(backend)),
        };
        Tasks(backend)
    }

    /// Request a new task to be performed.
    ///
    /// Tasks are performed asynchronously and, likely, in separate processes.
    /// There is no guarantee about times within which tasks are completed.
    pub fn request<M: Serialize>(&self, task: TaskRequest<Q>, message: M) -> Result<()> {
        let message = ::serde_json::to_vec(&message)?;
        self.0.request(task, &message)
    }
}


/// Mock tools to test `Tasks` users.
#[cfg(debug_assertions)]
pub struct MockTasks<Q: TaskQueue> {
    pub requests: Arc<::std::sync::Mutex<Vec<(TaskRequest<Q>, ::serde_json::Value)>>>,
}

#[cfg(debug_assertions)]
impl<Q: TaskQueue + 'static> MockTasks<Q> {
    /// Create a mock tasks instance to be used for tests.
    pub fn new() -> MockTasks<Q> {
        MockTasks {
            requests: Arc::new(::std::sync::Mutex::new(Vec::new())),
        }
    }

    /// Return the non-mock interface to interact with this mock.
    pub fn mock(&self) -> Tasks<Q> {
        Tasks(Arc::new(self::backend::mock::Mock {
            requests: self.requests.clone(),
        }))
    }
}


#[cfg(test)]
mod tests {
    use super::MockTasks;
    use super::TaskQueue;
    use super::TaskRequest;

    #[derive(Clone, Debug, Eq, PartialEq)]
    enum TestQueues {
        Test
    }

    impl TaskQueue for TestQueues {
        fn name(&self) -> String {
            match self {
                TestQueues::Test => "test".into(),
            }
        }
    }

    #[test]
    fn enqueue_request() {
        let task = TaskRequest::new(TestQueues::Test);
        let message: String = "Some text".into();
        let mock: MockTasks<TestQueues> = MockTasks::new();
        mock.mock().request(task, message).expect("failed to request task");
        let found = &mock.requests.lock().expect("failed to lock")[0];
        assert_eq!(found.0.queue(), &TestQueues::Test);
        assert_eq!("Some text", found.1);
    }

    #[test]
    fn request_unit() {
        let task = TaskRequest::new(TestQueues::Test);
        let mock: MockTasks<TestQueues> = MockTasks::new();
        mock.mock().request(task, ()).expect("failed to request task");
        let found = &mock.requests.lock().expect("failed to lock")[0];
        assert_eq!(found.0.queue(), &TestQueues::Test);
    }
}
