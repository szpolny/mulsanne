use crate::lmu::shared_memory::SharedMemory;

mod shared_memory_data;

struct LmuData {
    telemetry: SharedMemory,
    scoring: SharedMemory,
    rules: SharedMemory,
    multi_rules: SharedMemory,
    force_feedback: SharedMemory,
    pit_info: SharedMemory,
    weather: SharedMemory,
    extended: SharedMemory,
}

impl LmuData {
    pub fn new() -> Self {
        Self {
            telemetry: SharedMemory::new("$rFactor2SMMP_Telemetry$"),
            scoring: SharedMemory::new("$rFactor2SMMP_Scoring$"),
            rules: SharedMemory::new("$rFactor2SMMP_Rules$"),
            multi_rules: SharedMemory::new("$rFactor2SMMP_MultiRules$"),
            force_feedback: SharedMemory::new("$rFactor2SMMP_ForceFeedback$"),
            pit_info: SharedMemory::new("$rFactor2SMMP_PitInfo$"),
            weather: SharedMemory::new("$rFactor2SMMP_Weather$"),
            extended: SharedMemory::new("$rFactor2SMMP_Extended$"),
        }
    }
}

