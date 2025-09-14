use std::collections::VecDeque;
use sysinfo::{CpuExt, PidExt, ProcessExt, System, SystemExt};

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory: u64,
    pub memory_percent: f32,
}

#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub name: String,
    pub usage: f32,
    pub history: VecDeque<f32>, // keep last 60 readings for sparkline
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortOrder {
    Cpu,
    Memory,
    Pid,
    Name,
}

pub struct SystemMonitor {
    system: System,
    cpu_history: Vec<CpuInfo>,
    memory_history: VecDeque<f64>, // memory usage percentage over time
    max_history_len: usize,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        // initialize cpu history
        let cpu_history: Vec<CpuInfo> = system
            .cpus()
            .iter()
            .map(|cpu| CpuInfo {
                name: cpu.name().to_string(),
                usage: 0.0,
                history: VecDeque::with_capacity(60),
            })
            .collect();

        Self {
            system,
            cpu_history,
            memory_history: VecDeque::with_capacity(60),
            max_history_len: 60,
        }
    }

    pub fn refresh(&mut self) {
        self.system.refresh_all();
        
        // update cpu history
        for (i, cpu) in self.system.cpus().iter().enumerate() {
            if let Some(cpu_info) = self.cpu_history.get_mut(i) {
                cpu_info.usage = cpu.cpu_usage();
                
                if cpu_info.history.len() >= self.max_history_len {
                    cpu_info.history.pop_front();
                }
                cpu_info.history.push_back(cpu.cpu_usage());
            }
        }
        
        // update memory history
        let memory_percent = (self.system.used_memory() as f64 / self.system.total_memory() as f64) * 100.0;
        if self.memory_history.len() >= self.max_history_len {
            self.memory_history.pop_front();
        }
        self.memory_history.push_back(memory_percent);
    }

    pub fn get_processes(&self, sort_order: &SortOrder) -> Vec<ProcessInfo> {
        let mut processes: Vec<ProcessInfo> = self
            .system
            .processes()
            .values()
            .map(|proc| ProcessInfo {
                pid: proc.pid().as_u32(),
                name: proc.name().to_string(),
                cpu_usage: proc.cpu_usage(),
                memory: proc.memory(),
                memory_percent: (proc.memory() as f32 / self.system.total_memory() as f32) * 100.0,
            })
            .collect();

        // sort processes based on the current sort order
        match sort_order {
            SortOrder::Cpu => {
                processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
            }
            SortOrder::Memory => {
                processes.sort_by(|a, b| b.memory.cmp(&a.memory));
            }
            SortOrder::Pid => {
                processes.sort_by(|a, b| a.pid.cmp(&b.pid));
            }
            SortOrder::Name => {
                processes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            }
        }

        processes
    }

    pub fn get_cpu_info(&self) -> &Vec<CpuInfo> {
        &self.cpu_history
    }

    pub fn get_total_memory(&self) -> u64 {
        self.system.total_memory()
    }

    pub fn get_used_memory(&self) -> u64 {
        self.system.used_memory()
    }

    pub fn get_memory_percent(&self) -> f64 {
        (self.system.used_memory() as f64 / self.system.total_memory() as f64) * 100.0
    }

    pub fn get_memory_history(&self) -> &VecDeque<f64> {
        &self.memory_history
    }

    pub fn get_system_info(&self) -> SystemInfo {
        SystemInfo {
            hostname: self.system.host_name().unwrap_or_else(|| "unknown".to_string()),
            kernel_version: self.system.kernel_version().unwrap_or_else(|| "unknown".to_string()),
            os_version: self.system.long_os_version().unwrap_or_else(|| "unknown".to_string()),
            uptime: self.system.uptime(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub hostname: String,
    pub kernel_version: String,
    pub os_version: String,
    pub uptime: u64,
}