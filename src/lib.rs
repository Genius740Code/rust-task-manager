pub mod app;
pub mod system;
pub mod ui;

pub use app::App;
pub use system::{ProcessInfo, SystemMonitor, SortOrder};

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_system_monitor_creation() {
        let monitor = SystemMonitor::new();
        assert!(monitor.get_total_memory() > 0);
    }

    #[test] 
    fn test_process_sorting() {
        let monitor = SystemMonitor::new();
        let processes_cpu = monitor.get_processes(&SortOrder::Cpu);
        let processes_memory = monitor.get_processes(&SortOrder::Memory);
        
        // just check that we get some processes back
        assert!(!processes_cpu.is_empty());
        assert!(!processes_memory.is_empty());
    }

    #[test]
    fn test_app_creation() {
        let app = App::new(Duration::from_millis(1000), false);
        assert!(app.is_ok());
    }
}