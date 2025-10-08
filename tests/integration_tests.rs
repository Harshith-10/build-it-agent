// Integration tests for build-it-agent
use build_it_agent::*;

#[cfg(test)]
mod type_integration_tests {
    use super::*;

    #[test]
    fn test_full_execute_request_workflow() {
        // Create test cases
        let test_cases = vec![
            TestCase {
                id: 1,
                input: "5\n10\n".to_string(),
                expected: Some("15\n".to_string()),
                timeout_ms: Some(1000),
            },
            TestCase {
                id: 2,
                input: "3\n7\n".to_string(),
                expected: Some("10\n".to_string()),
                timeout_ms: Some(1000),
            },
        ];

        // Create execute request
        let request = ExecuteRequest {
            language: "python3".to_string(),
            code: "a = int(input())\nb = int(input())\nprint(a + b)".to_string(),
            testcases: test_cases,
        };

        // Serialize and deserialize
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: ExecuteRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.language, "python3");
        assert_eq!(deserialized.testcases.len(), 2);
    }

    #[test]
    fn test_execute_response_creation() {
        let response = ExecuteResponse {
            compiled: true,
            language: "java".to_string(),
            status: Some(ExecutionStatus::Success),
            message: None,
            results: vec![
                CaseResult {
                    id: 1,
                    ok: true,
                    passed: true,
                    input: "input".to_string(),
                    expected: Some("output".to_string()),
                    stdout: "output".to_string(),
                    stderr: "".to_string(),
                    timed_out: false,
                    duration_ms: 100,
                    memory_kb: 2048,
                    exit_code: Some(0),
                    term_signal: None,
                }
            ],
            total_duration_ms: 150,
        };

        assert!(response.compiled);
        assert_eq!(response.results.len(), 1);
        assert!(response.results[0].passed);
    }
}

#[cfg(test)]
mod language_integration_tests {
    use super::*;
    use language::generate_language_configs;

    #[test]
    fn test_language_config_completeness() {
        let configs = generate_language_configs();
        
        // Ensure each config has the necessary fields
        for (name, config) in configs.iter() {
            assert!(!config.display_name.is_empty(), "{} missing display name", name);
            assert!(!config.version_command.is_empty(), "{} missing version command", name);
            assert!(!config.run_command.is_empty(), "{} missing run command", name);
            
            // File name can be empty for some languages like psql
            if !config.file_name.is_empty() {
                let ext = std::path::Path::new(&config.file_name)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                assert_eq!(config.file_extension, ext, "{} file extension mismatch", name);
            }
        }
    }

    #[test]
    fn test_multiple_language_support() {
        let configs = generate_language_configs();
        
        let expected_languages = vec![
            "python3", "python", "java", "gcc", "gpp", "clang", "clangpp"
        ];
        
        for lang in expected_languages {
            assert!(
                configs.contains_key(lang),
                "Expected language {} not found in configs",
                lang
            );
        }
    }
}

#[cfg(test)]
mod rusq_integration_tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_mpmc_queue_workflow() {
        let config = RusqConfig::default();
        let queue = MpmcQueue::new(config);
        
        let producer = queue.producer();
        let consumer = queue.consumer();

        // Send messages
        for i in 0..10 {
            producer.send(
                format!("Message {}", i),
                "test_topic".to_string()
            ).unwrap();
        }

        // Receive messages
        let mut received = Vec::new();
        for _ in 0..10 {
            if let Ok(msg) = consumer.try_recv() {
                received.push(msg.payload);
            }
        }

        assert_eq!(received.len(), 10);
    }

    #[test]
    fn test_priority_queue_integration() {
        let config = RusqConfig::default();
        let queue = MpmcQueue::new(config);
        
        let producer = queue.producer();
        let consumer = queue.consumer();

        // Send messages with different priorities
        let messages = vec![
            ("Low priority", Priority::Low),
            ("Normal priority", Priority::Normal),
            ("High priority", Priority::High),
            ("Critical priority", Priority::Critical),
        ];

        for (msg, priority) in messages {
            producer.send_with_priority(
                msg.to_string(),
                "priority_test".to_string(),
                priority
            ).unwrap();
        }

        // First message should be Critical
        let first = consumer.try_recv().unwrap();
        assert_eq!(first.payload, "Critical priority");
        assert_eq!(first.priority, Priority::Critical);
    }

    #[test]
    fn test_metrics_tracking() {
        let config = RusqConfig {
            enable_metrics: true,
            ..Default::default()
        };
        let queue = MpmcQueue::new(config);
        
        let producer = queue.producer();
        let consumer = queue.consumer();

        // Send and receive some messages
        for i in 0..5 {
            producer.send(
                format!("Message {}", i),
                "metrics_test".to_string()
            ).unwrap();
        }

        for _ in 0..3 {
            let _ = consumer.try_recv();
        }

        let metrics = queue.metrics();
        assert_eq!(metrics.messages_sent, 5);
        assert_eq!(metrics.messages_received, 3);
    }

    #[test]
    fn test_queue_shutdown_integration() {
        let config = RusqConfig::default();
        let queue = Arc::new(MpmcQueue::new(config));
        
        let queue_clone = queue.clone();
        let handle = thread::spawn(move || {
            let producer = queue_clone.producer();
            // Try to send after shutdown
            thread::sleep(Duration::from_millis(50));
            producer.send("test".to_string(), "topic".to_string())
        });

        thread::sleep(Duration::from_millis(10));
        queue.shutdown();

        let result = handle.join().unwrap();
        assert!(matches!(result, Err(RusqError::QueueShutdown)));
    }
}

#[cfg(test)]
mod cross_module_integration_tests {
    use super::*;

    #[test]
    fn test_execute_request_with_language_config() {
        use language::generate_language_configs;
        
        let configs = generate_language_configs();
        let python_config = configs.get("python3").unwrap();

        let request = ExecuteRequest {
            language: "python3".to_string(),
            code: "print('Hello, World!')".to_string(),
            testcases: vec![
                TestCase {
                    id: 1,
                    input: "".to_string(),
                    expected: Some("Hello, World!".to_string()),
                    timeout_ms: Some(1000),
                }
            ],
        };

        // Verify request language matches a valid config
        assert!(configs.contains_key(&request.language));
        assert_eq!(python_config.display_name, "Python 3");
    }

    #[test]
    fn test_message_queue_with_execute_requests() {
        let config = RusqConfig::default();
        let queue = MpmcQueue::new(config);
        
        let producer = queue.producer();
        let consumer = queue.consumer();

        let request = ExecuteRequest {
            language: "python3".to_string(),
            code: "print('test')".to_string(),
            testcases: vec![],
        };

        // Send execute request through queue
        producer.send(request.clone(), "execute_queue".to_string()).unwrap();

        // Receive and verify
        let received = consumer.try_recv().unwrap();
        assert_eq!(received.payload.language, "python3");
        assert_eq!(received.topic, "execute_queue");
    }

    #[test]
    fn test_execution_status_in_response() {
        let statuses = vec![
            ExecutionStatus::Success,
            ExecutionStatus::Error,
            ExecutionStatus::Timeout,
            ExecutionStatus::CompileError,
            ExecutionStatus::RuntimeError,
            ExecutionStatus::UnsupportedLanguage,
        ];

        for status in statuses {
            let response = ExecuteResponse {
                compiled: false,
                language: "test".to_string(),
                status: Some(status.clone()),
                message: Some("Test message".to_string()),
                results: vec![
                    CaseResult {
                        id: 1,
                        ok: true,
                        passed: true,
                        input: "".to_string(),
                        expected: None,
                        stdout: "".to_string(),
                        stderr: "".to_string(),
                        timed_out: false,
                        duration_ms: 0,
                        memory_kb: 0,
                        exit_code: Some(0),
                        term_signal: None,
                    }
                ],
                total_duration_ms: 0,
            };

            // Serialize and verify
            let json = serde_json::to_string(&response).unwrap();
            let deserialized: ExecuteResponse = serde_json::from_str(&json).unwrap();
            
            assert!(deserialized.status.is_some());
            assert!(!deserialized.results.is_empty());
        }
    }
}

