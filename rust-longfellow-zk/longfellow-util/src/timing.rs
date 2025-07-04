/// Timing and benchmarking utilities

use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Timer for measuring execution time
#[derive(Debug, Clone)]
pub struct Timer {
    start: Instant,
    checkpoints: Vec<(String, Duration)>,
}

impl Timer {
    /// Create a new timer
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            checkpoints: Vec::new(),
        }
    }
    
    /// Add a checkpoint
    pub fn checkpoint(&mut self, name: impl Into<String>) {
        let elapsed = self.start.elapsed();
        self.checkpoints.push((name.into(), elapsed));
    }
    
    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
    
    /// Get elapsed milliseconds
    pub fn elapsed_ms(&self) -> f64 {
        self.elapsed().as_secs_f64() * 1000.0
    }
    
    /// Reset the timer
    pub fn reset(&mut self) {
        self.start = Instant::now();
        self.checkpoints.clear();
    }
    
    /// Get checkpoints
    pub fn checkpoints(&self) -> &[(String, Duration)] {
        &self.checkpoints
    }
    
    /// Print summary
    pub fn print_summary(&self) {
        println!("Timer Summary:");
        println!("  Total: {:.3}ms", self.elapsed_ms());
        
        let mut last_time = Duration::ZERO;
        for (name, time) in &self.checkpoints {
            let delta = *time - last_time;
            println!(
                "  {}: {:.3}ms (delta: {:.3}ms)",
                name,
                time.as_secs_f64() * 1000.0,
                delta.as_secs_f64() * 1000.0
            );
            last_time = *time;
        }
    }
}

/// Time a function execution
pub fn time_operation<F, R>(name: &str, f: F) -> (R, Duration)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = f();
    let elapsed = start.elapsed();
    
    log::debug!(
        "{} completed in {:.3}ms",
        name,
        elapsed.as_secs_f64() * 1000.0
    );
    
    (result, elapsed)
}

/// Time a function with detailed stats
pub fn time_with_stats<F, R>(name: &str, iterations: usize, mut f: F) -> TimingStats
where
    F: FnMut() -> R,
{
    let mut times = Vec::with_capacity(iterations);
    
    for _ in 0..iterations {
        let start = Instant::now();
        f();
        times.push(start.elapsed());
    }
    
    TimingStats::new(name, times)
}

/// Timing statistics
#[derive(Debug, Clone)]
pub struct TimingStats {
    pub name: String,
    pub min: Duration,
    pub max: Duration,
    pub mean: Duration,
    pub median: Duration,
    pub std_dev: Duration,
    pub samples: usize,
}

impl TimingStats {
    /// Create from samples
    pub fn new(name: impl Into<String>, mut times: Vec<Duration>) -> Self {
        assert!(!times.is_empty(), "No timing samples");
        
        times.sort();
        
        let min = times[0];
        let max = times[times.len() - 1];
        let median = times[times.len() / 2];
        
        let sum: Duration = times.iter().sum();
        let mean = sum / times.len() as u32;
        
        // Calculate standard deviation
        let mean_nanos = mean.as_nanos() as f64;
        let variance = times.iter()
            .map(|t| {
                let diff = t.as_nanos() as f64 - mean_nanos;
                diff * diff
            })
            .sum::<f64>() / times.len() as f64;
        let std_dev_nanos = variance.sqrt();
        let std_dev = Duration::from_nanos(std_dev_nanos as u64);
        
        Self {
            name: name.into(),
            min,
            max,
            mean,
            median,
            std_dev,
            samples: times.len(),
        }
    }
    
    /// Print statistics
    pub fn print(&self) {
        println!("Timing Statistics for '{}':", self.name);
        println!("  Samples: {}", self.samples);
        println!("  Min:     {:.3}ms", self.min.as_secs_f64() * 1000.0);
        println!("  Max:     {:.3}ms", self.max.as_secs_f64() * 1000.0);
        println!("  Mean:    {:.3}ms", self.mean.as_secs_f64() * 1000.0);
        println!("  Median:  {:.3}ms", self.median.as_secs_f64() * 1000.0);
        println!("  Std Dev: {:.3}ms", self.std_dev.as_secs_f64() * 1000.0);
    }
}

/// Multi-timer for tracking multiple operations
#[derive(Debug, Clone)]
pub struct MultiTimer {
    timers: HashMap<String, Timer>,
}

impl MultiTimer {
    /// Create a new multi-timer
    pub fn new() -> Self {
        Self {
            timers: HashMap::new(),
        }
    }
    
    /// Start a timer
    pub fn start(&mut self, name: impl Into<String>) {
        self.timers.insert(name.into(), Timer::new());
    }
    
    /// Stop a timer and get elapsed time
    pub fn stop(&mut self, name: &str) -> Option<Duration> {
        self.timers.get(name).map(|timer| timer.elapsed())
    }
    
    /// Add checkpoint to a timer
    pub fn checkpoint(&mut self, timer_name: &str, checkpoint_name: impl Into<String>) {
        if let Some(timer) = self.timers.get_mut(timer_name) {
            timer.checkpoint(checkpoint_name);
        }
    }
    
    /// Get all timers
    pub fn timers(&self) -> &HashMap<String, Timer> {
        &self.timers
    }
    
    /// Print all timers
    pub fn print_all(&self) {
        println!("Multi-Timer Summary:");
        for (name, timer) in &self.timers {
            println!("\n[{}]", name);
            timer.print_summary();
        }
    }
}

/// Scope guard for automatic timing
pub struct ScopeTimer {
    name: String,
    start: Instant,
    print_on_drop: bool,
}

impl ScopeTimer {
    /// Create a new scope timer
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            print_on_drop: true,
        }
    }
    
    /// Create without printing on drop
    pub fn silent(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            print_on_drop: false,
        }
    }
    
    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Drop for ScopeTimer {
    fn drop(&mut self) {
        if self.print_on_drop {
            let elapsed = self.start.elapsed();
            log::debug!(
                "Scope '{}' completed in {:.3}ms",
                self.name,
                elapsed.as_secs_f64() * 1000.0
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_timer() {
        let mut timer = Timer::new();
        thread::sleep(Duration::from_millis(10));
        timer.checkpoint("first");
        thread::sleep(Duration::from_millis(10));
        timer.checkpoint("second");
        
        assert!(timer.elapsed() >= Duration::from_millis(20));
        assert_eq!(timer.checkpoints().len(), 2);
    }
    
    #[test]
    fn test_time_operation() {
        let (result, duration) = time_operation("test op", || {
            thread::sleep(Duration::from_millis(10));
            42
        });
        
        assert_eq!(result, 42);
        assert!(duration >= Duration::from_millis(10));
    }
    
    #[test]
    fn test_timing_stats() {
        let times = vec![
            Duration::from_millis(10),
            Duration::from_millis(20),
            Duration::from_millis(15),
            Duration::from_millis(25),
            Duration::from_millis(30),
        ];
        
        let stats = TimingStats::new("test", times);
        assert_eq!(stats.min, Duration::from_millis(10));
        assert_eq!(stats.max, Duration::from_millis(30));
        assert_eq!(stats.median, Duration::from_millis(20));
        assert_eq!(stats.samples, 5);
    }
    
    #[test]
    fn test_scope_timer() {
        {
            let _timer = ScopeTimer::silent("test scope");
            thread::sleep(Duration::from_millis(10));
        }
        // Timer automatically logs on drop
    }
}