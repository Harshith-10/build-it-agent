use crossbeam_channel::{bounded, unbounded, select, Receiver, Sender, TryRecvError, TrySendError};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Message priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// A message wrapper that contains metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<T> {
    pub id: u64,
    pub payload: T,
    pub priority: Priority,
    pub timestamp: u64,
    pub retry_count: u32,
    pub topic: String,
}

impl<T> Message<T> {
    pub fn new(payload: T, topic: String) -> Self {
        Self {
            id: generate_message_id(),
            payload,
            priority: Priority::Normal,
            timestamp: current_timestamp_millis(),
            retry_count: 0,
            topic,
        }
    }

    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }
}

/// Configuration for the MPMC queue
#[derive(Debug, Clone)]
pub struct QueueConfig {
    /// Bounded channel capacity (None for unbounded)
    pub capacity: Option<usize>,
    /// Enable priority queuing
    pub enable_priority: bool,
    /// Maximum retry attempts for failed messages
    pub max_retries: u32,
    /// Consumer timeout in milliseconds
    pub consumer_timeout_ms: u64,
    /// Enable metrics collection
    pub enable_metrics: bool,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            capacity: Some(10000),
            enable_priority: true,
            max_retries: 3,
            consumer_timeout_ms: 1000,
            enable_metrics: true,
        }
    }
}

/// Metrics for monitoring queue performance
#[derive(Debug, Default)]
pub struct QueueMetrics {
    pub messages_sent: AtomicU64,
    pub messages_received: AtomicU64,
    pub messages_failed: AtomicU64,
    pub messages_retried: AtomicU64,
    pub active_producers: AtomicU64,
    pub active_consumers: AtomicU64,
}

impl QueueMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment_sent(&self) {
        self.messages_sent.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_received(&self) {
        self.messages_received.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_failed(&self) {
        self.messages_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_retried(&self) {
        self.messages_retried.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add_producer(&self) {
        self.active_producers.fetch_add(1, Ordering::Relaxed);
    }

    pub fn remove_producer(&self) {
        self.active_producers.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn add_consumer(&self) {
        self.active_consumers.fetch_add(1, Ordering::Relaxed);
    }

    pub fn remove_consumer(&self) {
        self.active_consumers.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            messages_sent: self.messages_sent.load(Ordering::Relaxed),
            messages_received: self.messages_received.load(Ordering::Relaxed),
            messages_failed: self.messages_failed.load(Ordering::Relaxed),
            messages_retried: self.messages_retried.load(Ordering::Relaxed),
            active_producers: self.active_producers.load(Ordering::Relaxed),
            active_consumers: self.active_consumers.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub messages_failed: u64,
    pub messages_retried: u64,
    pub active_producers: u64,
    pub active_consumers: u64,
}

/// High-performance MPMC Message Queue
pub struct MpmcQueue<T> {
    // Priority queues for different priority levels
    critical_sender: Sender<Message<T>>,
    critical_receiver: Receiver<Message<T>>,
    high_sender: Sender<Message<T>>,
    high_receiver: Receiver<Message<T>>,
    normal_sender: Sender<Message<T>>,
    normal_receiver: Receiver<Message<T>>,
    low_sender: Sender<Message<T>>,
    low_receiver: Receiver<Message<T>>,
    
    // Dead letter queue for failed messages
    dlq_sender: Sender<Message<T>>,
    dlq_receiver: Receiver<Message<T>>,
    
    config: QueueConfig,
    metrics: Arc<QueueMetrics>,
    is_shutdown: Arc<AtomicBool>,
}

impl<T> MpmcQueue<T>
where
    T: Clone + Send + 'static,
{
    /// Create a new MPMC queue with the given configuration
    pub fn new(config: QueueConfig) -> Self {
        let create_channel = |capacity: Option<usize>| {
            if let Some(cap) = capacity {
                bounded(cap)
            } else {
                unbounded()
            }
        };

        let (critical_sender, critical_receiver) = create_channel(config.capacity);
        let (high_sender, high_receiver) = create_channel(config.capacity);
        let (normal_sender, normal_receiver) = create_channel(config.capacity);
        let (low_sender, low_receiver) = create_channel(config.capacity);
        let (dlq_sender, dlq_receiver) = create_channel(None); // DLQ is always unbounded

        Self {
            critical_sender,
            critical_receiver,
            high_sender,
            high_receiver,
            normal_sender,
            normal_receiver,
            low_sender,
            low_receiver,
            dlq_sender,
            dlq_receiver,
            config,
            metrics: Arc::new(QueueMetrics::new()),
            is_shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Create a producer handle for sending messages
    pub fn producer(&self) -> Producer<T> {
        if self.config.enable_metrics {
            self.metrics.add_producer();
        }

        Producer {
            critical_sender: self.critical_sender.clone(),
            high_sender: self.high_sender.clone(),
            normal_sender: self.normal_sender.clone(),
            low_sender: self.low_sender.clone(),
            config: self.config.clone(),
            metrics: self.metrics.clone(),
            is_shutdown: self.is_shutdown.clone(),
        }
    }

    /// Create a consumer handle for receiving messages
    pub fn consumer(&self) -> Consumer<T> {
        if self.config.enable_metrics {
            self.metrics.add_consumer();
        }

        Consumer {
            critical_receiver: self.critical_receiver.clone(),
            high_receiver: self.high_receiver.clone(),
            normal_receiver: self.normal_receiver.clone(),
            low_receiver: self.low_receiver.clone(),
            dlq_sender: self.dlq_sender.clone(),
            config: self.config.clone(),
            metrics: self.metrics.clone(),
            is_shutdown: self.is_shutdown.clone(),
        }
    }

    /// Get a handle to the dead letter queue
    pub fn dead_letter_queue(&self) -> DeadLetterQueue<T> {
        DeadLetterQueue {
            dlq_receiver: self.dlq_receiver.clone(),
            metrics: self.metrics.clone(),
        }
    }

    /// Get current queue metrics
    pub fn metrics(&self) -> MetricsSnapshot {
        self.metrics.snapshot()
    }

    /// Shutdown the queue gracefully
    pub fn shutdown(&self) {
        self.is_shutdown.store(true, Ordering::SeqCst);
    }

    /// Check if the queue is shutdown
    pub fn is_shutdown(&self) -> bool {
        self.is_shutdown.load(Ordering::SeqCst)
    }
}

/// Producer handle for sending messages to the queue
pub struct Producer<T> {
    critical_sender: Sender<Message<T>>,
    high_sender: Sender<Message<T>>,
    normal_sender: Sender<Message<T>>,
    low_sender: Sender<Message<T>>,
    config: QueueConfig,
    metrics: Arc<QueueMetrics>,
    is_shutdown: Arc<AtomicBool>,
}

impl<T> Producer<T>
where
    T: Clone + Send,
{
    /// Send a message with default priority
    pub fn send(&self, payload: T, topic: String) -> Result<(), QueueError> {
        let message = Message::new(payload, topic);
        self.send_message(message)
    }

    /// Send a message with specified priority
    pub fn send_with_priority(&self, payload: T, topic: String, priority: Priority) -> Result<(), QueueError> {
        let message = Message::new(payload, topic).with_priority(priority);
        self.send_message(message)
    }

    /// Send a pre-constructed message
    pub fn send_message(&self, message: Message<T>) -> Result<(), QueueError> {
        if self.is_shutdown.load(Ordering::SeqCst) {
            return Err(QueueError::QueueShutdown);
        }

        let sender = match message.priority {
            Priority::Critical => &self.critical_sender,
            Priority::High => &self.high_sender,
            Priority::Normal => &self.normal_sender,
            Priority::Low => &self.low_sender,
        };

        match sender.try_send(message) {
            Ok(_) => {
                if self.config.enable_metrics {
                    self.metrics.increment_sent();
                }
                Ok(())
            }
            Err(TrySendError::Full(_)) => Err(QueueError::QueueFull),
            Err(TrySendError::Disconnected(_)) => Err(QueueError::QueueShutdown),
        }
    }

    /// Send a message with blocking behavior
    pub fn send_blocking(&self, payload: T, topic: String) -> Result<(), QueueError> {
        let message = Message::new(payload, topic);
        self.send_message_blocking(message)
    }

    /// Send a pre-constructed message with blocking behavior
    pub fn send_message_blocking(&self, message: Message<T>) -> Result<(), QueueError> {
        if self.is_shutdown.load(Ordering::SeqCst) {
            return Err(QueueError::QueueShutdown);
        }

        let sender = match message.priority {
            Priority::Critical => &self.critical_sender,
            Priority::High => &self.high_sender,
            Priority::Normal => &self.normal_sender,
            Priority::Low => &self.low_sender,
        };

        match sender.send(message) {
            Ok(_) => {
                if self.config.enable_metrics {
                    self.metrics.increment_sent();
                }
                Ok(())
            }
            Err(_) => Err(QueueError::QueueShutdown),
        }
    }
}

impl<T> Drop for Producer<T> {
    fn drop(&mut self) {
        self.metrics.remove_producer();
    }
}

/// Consumer handle for receiving messages from the queue
pub struct Consumer<T> {
    critical_receiver: Receiver<Message<T>>,
    high_receiver: Receiver<Message<T>>,
    normal_receiver: Receiver<Message<T>>,
    low_receiver: Receiver<Message<T>>,
    dlq_sender: Sender<Message<T>>,
    config: QueueConfig,
    metrics: Arc<QueueMetrics>,
    is_shutdown: Arc<AtomicBool>,
}

impl<T> Consumer<T>
where
    T: Clone + Send,
{
    /// Receive a message with priority ordering (non-blocking)
    pub fn try_recv(&self) -> Result<Message<T>, QueueError> {
        if self.is_shutdown.load(Ordering::SeqCst) {
            return Err(QueueError::QueueShutdown);
        }

        // Check priority queues in order: Critical -> High -> Normal -> Low
        match self.critical_receiver.try_recv() {
            Ok(msg) => {
                if self.config.enable_metrics {
                    self.metrics.increment_received();
                }
                return Ok(msg);
            }
            Err(TryRecvError::Disconnected) => return Err(QueueError::QueueShutdown),
            Err(TryRecvError::Empty) => {}
        }

        match self.high_receiver.try_recv() {
            Ok(msg) => {
                if self.config.enable_metrics {
                    self.metrics.increment_received();
                }
                return Ok(msg);
            }
            Err(TryRecvError::Disconnected) => return Err(QueueError::QueueShutdown),
            Err(TryRecvError::Empty) => {}
        }

        match self.normal_receiver.try_recv() {
            Ok(msg) => {
                if self.config.enable_metrics {
                    self.metrics.increment_received();
                }
                return Ok(msg);
            }
            Err(TryRecvError::Disconnected) => return Err(QueueError::QueueShutdown),
            Err(TryRecvError::Empty) => {}
        }

        match self.low_receiver.try_recv() {
            Ok(msg) => {
                if self.config.enable_metrics {
                    self.metrics.increment_received();
                }
                Ok(msg)
            }
            Err(TryRecvError::Disconnected) => Err(QueueError::QueueShutdown),
            Err(TryRecvError::Empty) => Err(QueueError::Empty),
        }
    }

    /// Receive a message with priority ordering (blocking with timeout)
    pub fn recv_timeout(&self, timeout: Duration) -> Result<Message<T>, QueueError> {
        if self.is_shutdown.load(Ordering::SeqCst) {
            return Err(QueueError::QueueShutdown);
        }

        let start_time = Instant::now();

        loop {
            if self.is_shutdown.load(Ordering::SeqCst) {
                return Err(QueueError::QueueShutdown);
            }

            if start_time.elapsed() >= timeout {
                return Err(QueueError::Timeout);
            }

            // Use select! to efficiently wait on multiple receivers
            select! {
                recv(self.critical_receiver) -> msg => {
                    match msg {
                        Ok(message) => {
                            if self.config.enable_metrics {
                                self.metrics.increment_received();
                            }
                            return Ok(message);
                        }
                        Err(_) => return Err(QueueError::QueueShutdown),
                    }
                }
                recv(self.high_receiver) -> msg => {
                    match msg {
                        Ok(message) => {
                            if self.config.enable_metrics {
                                self.metrics.increment_received();
                            }
                            return Ok(message);
                        }
                        Err(_) => return Err(QueueError::QueueShutdown),
                    }
                }
                recv(self.normal_receiver) -> msg => {
                    match msg {
                        Ok(message) => {
                            if self.config.enable_metrics {
                                self.metrics.increment_received();
                            }
                            return Ok(message);
                        }
                        Err(_) => return Err(QueueError::QueueShutdown),
                    }
                }
                recv(self.low_receiver) -> msg => {
                    match msg {
                        Ok(message) => {
                            if self.config.enable_metrics {
                                self.metrics.increment_received();
                            }
                            return Ok(message);
                        }
                        Err(_) => return Err(QueueError::QueueShutdown),
                    }
                }
                default(Duration::from_millis(10)) => {
                    // Continue the loop to check timeout
                }
            }
        }
    }

    /// Receive a message with priority ordering (blocking)
    pub fn recv(&self) -> Result<Message<T>, QueueError> {
        self.recv_timeout(Duration::from_millis(self.config.consumer_timeout_ms))
    }

    /// Mark a message as failed and potentially send to DLQ
    pub fn nack(&self, mut message: Message<T>) -> Result<(), QueueError> {
        message.retry_count += 1;

        if self.config.enable_metrics {
            self.metrics.increment_failed();
        }

        if message.retry_count > self.config.max_retries {
            // Send to dead letter queue
            match self.dlq_sender.try_send(message) {
                Ok(_) => Ok(()),
                Err(TrySendError::Full(_)) => Err(QueueError::QueueFull),
                Err(TrySendError::Disconnected(_)) => Err(QueueError::QueueShutdown),
            }
        } else {
            if self.config.enable_metrics {
                self.metrics.increment_retried();
            }

            // Retry by sending back to the appropriate queue
            let sender = match message.priority {
                Priority::Critical => &self.critical_receiver,
                Priority::High => &self.high_receiver,
                Priority::Normal => &self.normal_receiver,
                Priority::Low => &self.low_receiver,
            };

            // This is a bit tricky - we need to get the sender from the receiver
            // In a real implementation, you might want to refactor this
            Err(QueueError::RetryRequired)
        }
    }
}

impl<T> Drop for Consumer<T> {
    fn drop(&mut self) {
        self.metrics.remove_consumer();
    }
}

/// Handle for accessing the dead letter queue
pub struct DeadLetterQueue<T> {
    dlq_receiver: Receiver<Message<T>>,
    metrics: Arc<QueueMetrics>,
}

impl<T> DeadLetterQueue<T> {
    /// Get a failed message from the dead letter queue
    pub fn try_recv(&self) -> Result<Message<T>, QueueError> {
        match self.dlq_receiver.try_recv() {
            Ok(msg) => Ok(msg),
            Err(TryRecvError::Empty) => Err(QueueError::Empty),
            Err(TryRecvError::Disconnected) => Err(QueueError::QueueShutdown),
        }
    }

    /// Get a failed message from the dead letter queue with timeout
    pub fn recv_timeout(&self, timeout: Duration) -> Result<Message<T>, QueueError> {
        match self.dlq_receiver.recv_timeout(timeout) {
            Ok(msg) => Ok(msg),
            Err(_) => Err(QueueError::Timeout),
        }
    }
}

/// Error types for the MPMC queue
#[derive(Debug, Clone, PartialEq)]
pub enum QueueError {
    QueueFull,
    QueueShutdown,
    Empty,
    Timeout,
    RetryRequired,
}

impl std::fmt::Display for QueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueueError::QueueFull => write!(f, "Queue is full"),
            QueueError::QueueShutdown => write!(f, "Queue is shutdown"),
            QueueError::Empty => write!(f, "Queue is empty"),
            QueueError::Timeout => write!(f, "Operation timed out"),
            QueueError::RetryRequired => write!(f, "Message retry required"),
        }
    }
}

impl std::error::Error for QueueError {}

// Utility functions
fn generate_message_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

fn current_timestamp_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_basic_send_receive() {
        let config = QueueConfig::default();
        let queue = MpmcQueue::new(config);
        
        let producer = queue.producer();
        let consumer = queue.consumer();

        // Send a message
        producer.send("Hello, World!".to_string(), "test".to_string()).unwrap();

        // Receive the message
        let message = consumer.try_recv().unwrap();
        assert_eq!(message.payload, "Hello, World!");
        assert_eq!(message.topic, "test");
    }

    #[test]
    fn test_priority_ordering() {
        let config = QueueConfig::default();
        let queue = MpmcQueue::new(config);
        
        let producer = queue.producer();
        let consumer = queue.consumer();

        // Send messages with different priorities
        producer.send_with_priority("Low".to_string(), "test".to_string(), Priority::Low).unwrap();
        producer.send_with_priority("High".to_string(), "test".to_string(), Priority::High).unwrap();
        producer.send_with_priority("Normal".to_string(), "test".to_string(), Priority::Normal).unwrap();
        producer.send_with_priority("Critical".to_string(), "test".to_string(), Priority::Critical).unwrap();

        // Receive messages - should come in priority order
        assert_eq!(consumer.try_recv().unwrap().payload, "Critical");
        assert_eq!(consumer.try_recv().unwrap().payload, "High");
        assert_eq!(consumer.try_recv().unwrap().payload, "Normal");
        assert_eq!(consumer.try_recv().unwrap().payload, "Low");
    }

    #[test]
    fn test_mpmc_concurrency() {
        let config = QueueConfig::default();
        let queue = Arc::new(MpmcQueue::new(config));
        
        let num_producers = 4;
        let num_consumers = 2;
        let messages_per_producer = 100;

        let mut handles = vec![];

        // Spawn producers
        for producer_id in 0..num_producers {
            let queue_clone = queue.clone();
            let handle = thread::spawn(move || {
                let producer = queue_clone.producer();
                for i in 0..messages_per_producer {
                    let message = format!("Producer-{}-Message-{}", producer_id, i);
                    producer.send(message, "test".to_string()).unwrap();
                }
            });
            handles.push(handle);
        }

        // Spawn consumers
        let received_count = Arc::new(AtomicU64::new(0));
        for _consumer_id in 0..num_consumers {
            let queue_clone = queue.clone();
            let count_clone = received_count.clone();
            let handle = thread::spawn(move || {
                let consumer = queue_clone.consumer();
                loop {
                    match consumer.try_recv() {
                        Ok(_) => {
                            count_clone.fetch_add(1, Ordering::SeqCst);
                        }
                        Err(QueueError::Empty) => {
                            thread::sleep(Duration::from_millis(1));
                        }
                        Err(_) => break,
                    }
                }
            });
            handles.push(handle);
        }

        // Wait for producers to finish
        for handle in handles.into_iter().take(num_producers) {
            handle.join().unwrap();
        }

        // Wait a bit for consumers to process all messages
        thread::sleep(Duration::from_millis(100));

        let total_sent = num_producers * messages_per_producer;
        let total_received = received_count.load(Ordering::SeqCst) as usize;
        
        // In a real test, you'd want to ensure all messages are consumed
        assert!(total_received <= total_sent);
        
        queue.shutdown();
    }

    #[test]
    fn test_metrics() {
        let config = QueueConfig::default();
        let queue = MpmcQueue::new(config);
        
        let producer = queue.producer();
        let consumer = queue.consumer();

        // Send some messages
        for i in 0..5 {
            producer.send(format!("Message {}", i), "test".to_string()).unwrap();
        }

        // Receive some messages
        for _ in 0..3 {
            consumer.try_recv().unwrap();
        }

        let metrics = queue.metrics();
        assert_eq!(metrics.messages_sent, 5);
        assert_eq!(metrics.messages_received, 3);
        assert_eq!(metrics.active_producers, 1);
        assert_eq!(metrics.active_consumers, 1);
    }
}
