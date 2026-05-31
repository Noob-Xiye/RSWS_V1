use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

// 雪花ID生成器
pub struct SnowflakeGenerator {
    machine_id: u64,
    sequence: u64,
    last_timestamp: u64,
}

static SNOWFLAKE_GENERATOR: Lazy<Mutex<SnowflakeGenerator>> = Lazy::new(|| {
    Mutex::new(SnowflakeGenerator::new(1)) // 机器ID可以从配置读取
});

impl SnowflakeGenerator {
    const EPOCH: u64 = 1640995200000; // 2022-01-01 00:00:00 UTC
    const MACHINE_ID_BITS: u64 = 10;
    const SEQUENCE_BITS: u64 = 12;
    const MAX_MACHINE_ID: u64 = (1 << Self::MACHINE_ID_BITS) - 1;
    const MAX_SEQUENCE: u64 = (1 << Self::SEQUENCE_BITS) - 1;
    const MACHINE_ID_SHIFT: u64 = Self::SEQUENCE_BITS;
    const TIMESTAMP_SHIFT: u64 = Self::SEQUENCE_BITS + Self::MACHINE_ID_BITS;

    pub fn new(machine_id: u64) -> Self {
        assert!(
            machine_id <= Self::MAX_MACHINE_ID,
            "Machine ID exceeds maximum value"
        );
        Self {
            machine_id,
            sequence: 0,
            last_timestamp: 0,
        }
    }

    pub fn next_id(&mut self) -> Result<u64, String> {
        let mut timestamp = self.current_timestamp();

        if timestamp < self.last_timestamp {
            return Err("Clock moved backwards".to_string());
        }

        if timestamp == self.last_timestamp {
            self.sequence = (self.sequence + 1) & Self::MAX_SEQUENCE;
            if self.sequence == 0 {
                timestamp = self.wait_next_millis(self.last_timestamp);
            }
        } else {
            self.sequence = 0;
        }

        self.last_timestamp = timestamp;

        Ok(((timestamp - Self::EPOCH) << Self::TIMESTAMP_SHIFT)
            | (self.machine_id << Self::MACHINE_ID_SHIFT)
            | self.sequence)
    }

    fn current_timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    fn wait_next_millis(&self, last_timestamp: u64) -> u64 {
        let mut timestamp = self.current_timestamp();
        while timestamp <= last_timestamp {
            timestamp = self.current_timestamp();
        }
        timestamp
    }
}

pub fn next_id() -> i64 {
    let mut generator = SNOWFLAKE_GENERATOR.lock().unwrap();
    generator.next_id().unwrap() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snowflake_generator() {
        let mut generator = SnowflakeGenerator::new(1);
        let id1 = generator.next_id().unwrap();
        let id2 = generator.next_id().unwrap();
        assert_ne!(id1, id2);
        assert!(id2 > id1);
    }

    #[test]
    fn test_next_id() {
        let id1 = next_id();
        let id2 = next_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_ids_are_positive() {
        for _ in 0..100 {
            let id = next_id();
            assert!(id > 0);
        }
    }
}
