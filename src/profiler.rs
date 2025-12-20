//! Profiler - Performance Analysis and Optimization
//!
//! Comprehensive profiler for analyzing execution performance, memory usage,
//! fuel consumption, and identifying bottlenecks in smart contracts.

use crate::error::SlvrResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Performance metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
}

/// Function profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionProfile {
    pub name: String,
    pub call_count: u64,
    pub total_time_ms: f64,
    pub average_time_ms: f64,
    pub min_time_ms: f64,
    pub max_time_ms: f64,
    pub fuel_consumed: u64,
    pub memory_used: u64,
    pub call_stack_depth: u32,
}

/// Operation profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationProfile {
    pub operation: String,
    pub count: u64,
    pub total_time_ms: f64,
    pub average_time_ms: f64,
    pub fuel_per_op: u64,
}

/// Memory profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryProfile {
    pub allocated: u64,
    pub freed: u64,
    pub peak_usage: u64,
    pub current_usage: u64,
    pub allocations: u64,
    pub deallocations: u64,
}

/// Fuel profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuelProfile {
    pub total_fuel: u64,
    pub fuel_by_operation: HashMap<String, u64>,
    pub fuel_by_function: HashMap<String, u64>,
    pub average_fuel_per_op: u64,
}

/// Execution profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionProfile {
    pub id: String,
    pub name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_ms: f64,
    pub function_profiles: HashMap<String, FunctionProfile>,
    pub operation_profiles: HashMap<String, OperationProfile>,
    pub memory_profile: MemoryProfile,
    pub fuel_profile: FuelProfile,
    pub call_graph: CallGraph,
}

/// Call graph for function calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallGraph {
    pub nodes: HashMap<String, CallGraphNode>,
    pub edges: Vec<CallGraphEdge>,
}

/// Call graph node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallGraphNode {
    pub name: String,
    pub call_count: u64,
    pub total_time_ms: f64,
}

/// Call graph edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallGraphEdge {
    pub from: String,
    pub to: String,
    pub call_count: u64,
}

/// Profiler event
#[derive(Debug, Clone)]
enum ProfilerEvent {
    FunctionEnter {
        name: String,
        timestamp: Instant,
    },
    FunctionExit {
        name: String,
        timestamp: Instant,
    },
    OperationStart {
        operation: String,
        timestamp: Instant,
    },
    OperationEnd {
        operation: String,
        timestamp: Instant,
    },
    #[allow(dead_code)]
    MemoryAllocate {
        size: u64,
    },
    #[allow(dead_code)]
    MemoryFree {
        size: u64,
    },
    #[allow(dead_code)]
    FuelConsume {
        amount: u64,
        operation: String,
    },
}

/// Profiler
pub struct Profiler {
    profile: Arc<Mutex<ExecutionProfile>>,
    events: Arc<Mutex<Vec<ProfilerEvent>>>,
    start_time: Instant,
    current_function_stack: Arc<Mutex<Vec<String>>>,
}

impl Profiler {
    /// Create new profiler
    pub fn new(name: String) -> Self {
        let profile = ExecutionProfile {
            id: Uuid::new_v4().to_string(),
            name,
            start_time: Utc::now(),
            end_time: None,
            duration_ms: 0.0,
            function_profiles: HashMap::new(),
            operation_profiles: HashMap::new(),
            memory_profile: MemoryProfile {
                allocated: 0,
                freed: 0,
                peak_usage: 0,
                current_usage: 0,
                allocations: 0,
                deallocations: 0,
            },
            fuel_profile: FuelProfile {
                total_fuel: 0,
                fuel_by_operation: HashMap::new(),
                fuel_by_function: HashMap::new(),
                average_fuel_per_op: 0,
            },
            call_graph: CallGraph {
                nodes: HashMap::new(),
                edges: Vec::new(),
            },
        };

        Self {
            profile: Arc::new(Mutex::new(profile)),
            events: Arc::new(Mutex::new(Vec::new())),
            start_time: Instant::now(),
            current_function_stack: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Record function entry
    pub fn function_enter(&self, name: String) -> SlvrResult<()> {
        let event = ProfilerEvent::FunctionEnter {
            name: name.clone(),
            timestamp: Instant::now(),
        };

        let mut events = self.events.lock().unwrap();
        events.push(event);

        let mut stack = self.current_function_stack.lock().unwrap();
        stack.push(name);

        Ok(())
    }

    /// Record function exit
    pub fn function_exit(&self, name: String) -> SlvrResult<()> {
        let event = ProfilerEvent::FunctionExit {
            name: name.clone(),
            timestamp: Instant::now(),
        };

        let mut events = self.events.lock().unwrap();
        events.push(event);

        let mut stack = self.current_function_stack.lock().unwrap();
        if stack.last() == Some(&name) {
            stack.pop();
        }

        Ok(())
    }

    /// Record operation start
    pub fn operation_start(&self, operation: String) -> SlvrResult<()> {
        let event = ProfilerEvent::OperationStart {
            operation,
            timestamp: Instant::now(),
        };

        let mut events = self.events.lock().unwrap();
        events.push(event);

        Ok(())
    }

    /// Record operation end
    pub fn operation_end(&self, operation: String) -> SlvrResult<()> {
        let event = ProfilerEvent::OperationEnd {
            operation,
            timestamp: Instant::now(),
        };

        let mut events = self.events.lock().unwrap();
        events.push(event);

        Ok(())
    }

    /// Record memory allocation
    pub fn memory_allocate(&self, size: u64) -> SlvrResult<()> {
        let event = ProfilerEvent::MemoryAllocate { size };

        let mut events = self.events.lock().unwrap();
        events.push(event);

        let mut profile = self.profile.lock().unwrap();
        profile.memory_profile.allocated += size;
        profile.memory_profile.current_usage += size;
        profile.memory_profile.allocations += 1;

        if profile.memory_profile.current_usage > profile.memory_profile.peak_usage {
            profile.memory_profile.peak_usage = profile.memory_profile.current_usage;
        }

        Ok(())
    }

    /// Record memory deallocation
    pub fn memory_free(&self, size: u64) -> SlvrResult<()> {
        let event = ProfilerEvent::MemoryFree { size };

        let mut events = self.events.lock().unwrap();
        events.push(event);

        let mut profile = self.profile.lock().unwrap();
        profile.memory_profile.freed += size;
        profile.memory_profile.current_usage = profile.memory_profile.current_usage.saturating_sub(size);
        profile.memory_profile.deallocations += 1;

        Ok(())
    }

    /// Record fuel consumption
    pub fn consume_fuel(&self, amount: u64, operation: String) -> SlvrResult<()> {
        let event = ProfilerEvent::FuelConsume {
            amount,
            operation: operation.clone(),
        };

        let mut events = self.events.lock().unwrap();
        events.push(event);

        let mut profile = self.profile.lock().unwrap();
        profile.fuel_profile.total_fuel += amount;

        *profile
            .fuel_profile
            .fuel_by_operation
            .entry(operation.clone())
            .or_insert(0) += amount;

        let stack = self.current_function_stack.lock().unwrap();
        if let Some(func) = stack.last() {
            *profile
                .fuel_profile
                .fuel_by_function
                .entry(func.clone())
                .or_insert(0) += amount;
        }

        Ok(())
    }

    /// Finalize profiling
    pub fn finalize(&self) -> SlvrResult<ExecutionProfile> {
        let mut profile = self.profile.lock().unwrap();
        profile.end_time = Some(Utc::now());
        profile.duration_ms = self.start_time.elapsed().as_secs_f64() * 1000.0;

        // Process events to build profiles
        self.process_events(&mut profile)?;

        Ok(profile.clone())
    }

    /// Process recorded events
    fn process_events(&self, profile: &mut ExecutionProfile) -> SlvrResult<()> {
        let events = self.events.lock().unwrap();

        let mut function_times: HashMap<String, Vec<Duration>> = HashMap::new();
        let mut operation_times: HashMap<String, Vec<Duration>> = HashMap::new();
        let mut function_stack: Vec<(String, Instant)> = Vec::new();
        let mut operation_stack: Vec<(String, Instant)> = Vec::new();

        for event in events.iter() {
            match event {
                ProfilerEvent::FunctionEnter { name, timestamp } => {
                    function_stack.push((name.clone(), *timestamp));
                }
                ProfilerEvent::FunctionExit { name, timestamp } => {
                    if let Some((func_name, start_time)) = function_stack.pop() {
                        if func_name == *name {
                            let duration = timestamp.duration_since(start_time);
                            function_times
                                .entry(name.clone())
                                .or_default()
                                .push(duration);
                        }
                    }
                }
                ProfilerEvent::OperationStart { operation, timestamp } => {
                    operation_stack.push((operation.clone(), *timestamp));
                }
                ProfilerEvent::OperationEnd { operation, timestamp } => {
                    if let Some((op_name, start_time)) = operation_stack.pop() {
                        if op_name == *operation {
                            let duration = timestamp.duration_since(start_time);
                            operation_times
                                .entry(operation.clone())
                                .or_default()
                                .push(duration);
                        }
                    }
                }
                _ => {}
            }
        }

        // Build function profiles
        for (name, times) in function_times {
            let count = times.len() as u64;
            let total_time_ms = times.iter().map(|d| d.as_secs_f64() * 1000.0).sum::<f64>();
            let average_ms = total_time_ms / count as f64;
            let min_ms = times
                .iter()
                .map(|d| d.as_secs_f64() * 1000.0)
                .fold(f64::INFINITY, f64::min);
            let max_ms = times
                .iter()
                .map(|d| d.as_secs_f64() * 1000.0)
                .fold(0.0, f64::max);

            profile.function_profiles.insert(
                name.clone(),
                FunctionProfile {
                    name: name.clone(),
                    call_count: count,
                    total_time_ms,
                    average_time_ms: average_ms,
                    min_time_ms: min_ms,
                    max_time_ms: max_ms,
                    fuel_consumed: profile
                        .fuel_profile
                        .fuel_by_function
                        .get(&name)
                        .copied()
                        .unwrap_or(0),
                    memory_used: 0,
                    call_stack_depth: 0,
                },
            );
        }

        // Build operation profiles
        for (operation, times) in operation_times {
            let count = times.len() as u64;
            let total_time_ms = times.iter().map(|d| d.as_secs_f64() * 1000.0).sum::<f64>();
            let average_ms = total_time_ms / count as f64;

            profile.operation_profiles.insert(
                operation.clone(),
                OperationProfile {
                    operation: operation.clone(),
                    count,
                    total_time_ms,
                    average_time_ms: average_ms,
                    fuel_per_op: profile
                        .fuel_profile
                        .fuel_by_operation
                        .get(&operation)
                        .copied()
                        .unwrap_or(0)
                        / count.max(1),
                },
            );
        }

        // Calculate average fuel per operation
        if !profile.operation_profiles.is_empty() {
            let total_ops: u64 = profile.operation_profiles.values().map(|op| op.count).sum();
            if total_ops > 0 {
                profile.fuel_profile.average_fuel_per_op = profile.fuel_profile.total_fuel / total_ops;
            }
        }

        Ok(())
    }

    /// Get profile
    pub fn get_profile(&self) -> SlvrResult<ExecutionProfile> {
        let profile = self.profile.lock().unwrap();
        Ok(profile.clone())
    }

    /// Get function profile
    pub fn get_function_profile(&self, name: &str) -> SlvrResult<Option<FunctionProfile>> {
        let profile = self.profile.lock().unwrap();
        Ok(profile.function_profiles.get(name).cloned())
    }

    /// Get operation profile
    pub fn get_operation_profile(&self, operation: &str) -> SlvrResult<Option<OperationProfile>> {
        let profile = self.profile.lock().unwrap();
        Ok(profile.operation_profiles.get(operation).cloned())
    }

    /// Get memory profile
    pub fn get_memory_profile(&self) -> SlvrResult<MemoryProfile> {
        let profile = self.profile.lock().unwrap();
        Ok(profile.memory_profile.clone())
    }

    /// Get fuel profile
    pub fn get_fuel_profile(&self) -> SlvrResult<FuelProfile> {
        let profile = self.profile.lock().unwrap();
        Ok(profile.fuel_profile.clone())
    }

    /// Get hotspots (functions/operations taking most time)
    pub fn get_hotspots(&self, limit: usize) -> SlvrResult<Vec<(String, f64)>> {
        let profile = self.profile.lock().unwrap();

        let mut hotspots: Vec<(String, f64)> = profile
            .function_profiles
            .iter()
            .map(|(name, fp)| (name.clone(), fp.total_time_ms))
            .collect();

        hotspots.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        hotspots.truncate(limit);

        Ok(hotspots)
    }

    /// Get bottlenecks (operations with highest fuel consumption)
    pub fn get_bottlenecks(&self, limit: usize) -> SlvrResult<Vec<(String, u64)>> {
        let profile = self.profile.lock().unwrap();

        let mut bottlenecks: Vec<(String, u64)> = profile
            .fuel_profile
            .fuel_by_operation
            .iter()
            .map(|(op, fuel)| (op.clone(), *fuel))
            .collect();

        bottlenecks.sort_by(|a, b| b.1.cmp(&a.1));
        bottlenecks.truncate(limit);

        Ok(bottlenecks)
    }

    /// Generate report
    pub fn generate_report(&self) -> SlvrResult<String> {
        let profile = self.profile.lock().unwrap();

        let mut report = String::new();
        report.push_str(&format!("=== Profiling Report: {} ===\n", profile.name));
        report.push_str(&format!("Duration: {:.2}ms\n", profile.duration_ms));
        report.push_str(&format!("Total Fuel: {}\n", profile.fuel_profile.total_fuel));
        report.push_str(&format!("Peak Memory: {} bytes\n", profile.memory_profile.peak_usage));
        report.push('\n');

        report.push_str("=== Top Functions ===\n");
        let mut functions: Vec<_> = profile.function_profiles.values().collect();
        functions.sort_by(|a, b| b.total_time_ms.partial_cmp(&a.total_time_ms).unwrap_or(std::cmp::Ordering::Equal));
        for func in functions.iter().take(10) {
            report.push_str(&format!(
                "{}: {:.2}ms (calls: {}, avg: {:.2}ms)\n",
                func.name, func.total_time_ms, func.call_count, func.average_time_ms
            ));
        }

        report.push_str("\n=== Top Operations ===\n");
        let mut operations: Vec<_> = profile.operation_profiles.values().collect();
        operations.sort_by(|a, b| b.total_time_ms.partial_cmp(&a.total_time_ms).unwrap_or(std::cmp::Ordering::Equal));
        for op in operations.iter().take(10) {
            report.push_str(&format!(
                "{}: {:.2}ms (count: {}, avg: {:.2}ms, fuel/op: {})\n",
                op.operation, op.total_time_ms, op.count, op.average_time_ms, op.fuel_per_op
            ));
        }

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_creation() {
        let profiler = Profiler::new("test".to_string());
        let profile = profiler.get_profile().unwrap();
        assert_eq!(profile.name, "test");
    }

    #[test]
    fn test_function_profiling() {
        let profiler = Profiler::new("test".to_string());
        profiler.function_enter("test_func".to_string()).unwrap();
        std::thread::sleep(Duration::from_millis(10));
        profiler.function_exit("test_func".to_string()).unwrap();

        let profile = profiler.finalize().unwrap();
        assert!(profile.function_profiles.contains_key("test_func"));
    }

    #[test]
    fn test_memory_profiling() {
        let profiler = Profiler::new("test".to_string());
        profiler.memory_allocate(1000).unwrap();
        profiler.memory_allocate(2000).unwrap();
        profiler.memory_free(500).unwrap();

        let profile = profiler.get_profile().unwrap();
        assert_eq!(profile.memory_profile.allocated, 3000);
        assert_eq!(profile.memory_profile.freed, 500);
        assert_eq!(profile.memory_profile.current_usage, 2500);
    }

    #[test]
    fn test_fuel_profiling() {
        let profiler = Profiler::new("test".to_string());
        profiler.consume_fuel(100, "operation1".to_string()).unwrap();
        profiler.consume_fuel(200, "operation2".to_string()).unwrap();

        let profile = profiler.get_profile().unwrap();
        assert_eq!(profile.fuel_profile.total_fuel, 300);
    }

    #[test]
    fn test_hotspots() {
        let profiler = Profiler::new("test".to_string());
        profiler.function_enter("func1".to_string()).unwrap();
        std::thread::sleep(Duration::from_millis(20));
        profiler.function_exit("func1".to_string()).unwrap();

        profiler.function_enter("func2".to_string()).unwrap();
        std::thread::sleep(Duration::from_millis(10));
        profiler.function_exit("func2".to_string()).unwrap();

        let _profile = profiler.finalize().unwrap();
        let hotspots = profiler.get_hotspots(5).unwrap();
        assert!(!hotspots.is_empty());
    }
}
