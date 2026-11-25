//! A process as the system tells it, and what it is using.

/// One entry of the list of processes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProcessEntry {
    pub pid: u32,
    /// The pid that started it, which may be gone, and its number even given to
    /// another process.
    pub parent: u32,
    /// The name of the program's file, such as `warp.exe`.
    pub image: String,
}

/// What a process is using at one moment.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProcessUsage {
    pub private_bytes: u64,
    pub working_set_bytes: u64,
    /// Processor time since it started, kernel and user, in ticks of 100 ns.
    pub cpu_100ns: u64,
    pub threads: u32,
    pub handles: u32,
}
