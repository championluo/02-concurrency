use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, Result};

// metrics 的 data structure
#[derive(Debug, Clone)]
pub struct Metrics {
    //key 和 value 的类型都可以自定义\
    //这里如果类型写死的话, 就不需要再结构体签名上加泛型<>符号了
    data: Arc<Mutex<HashMap<String, i64>>>,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

// 基本功能  inc/dec/snapshot
impl Metrics {
    pub fn new() -> Self {
        Metrics {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn inc(&self, key: impl Into<String>) -> Result<()> {
        let mut mutex_guard = self.data.lock().map_err(|e| anyhow!(e.to_string()))?;
        let count = mutex_guard.entry(key.into()).or_insert(0);
        *count += 1;
        Ok(())
    }

    // pub fn dec(&mut self, key: impl Into<String>) {
    //     let mut arc = self.data.lock().map_err(|e| anyhow!(e.to_string())).unwrap();
    //     let count = arc.entry(key.into()).or_insert(0);
    //     *count -= 1
    // }

    pub fn snapshot(&self) -> Result<HashMap<String, i64>> {
        Ok(self
            .data
            .lock()
            .map_err(|e| anyhow!(e.to_string()))?
            .clone())
    }
}
