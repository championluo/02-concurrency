use std::fmt;
use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;

// metrics 的 data structure
#[derive(Debug, Clone)]
pub struct Metrics {
    //key 和 value 的类型都可以自定义\
    //这里如果类型写死的话, 就不需要再结构体签名上加泛型<>符号了
    data: Arc<DashMap<String, i64>>,
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
            data: Arc::new(DashMap::new()),
        }
    }

    pub fn inc(&self, key: impl Into<String>) -> Result<()> {
        let mut count = self.data.entry(key.into()).or_insert(0);
        *count += 1;
        Ok(())
    }

    //因为使用了 并发安全的DashMap, 所以这里也无需提供snapshot, 需要打印的时候直接打印即可, dashmap内部保证线程安全
    // pub fn snapshot(&self) -> Result<HashMap<String, i64>> {
    //     Ok(self
    //         .data
    //         .read()
    //         .map_err(|e| anyhow!(e.to_string()))?
    //         .clone())
    // }
}

impl fmt::Display for Metrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        for entry in self.data.iter() {
            writeln!(f, "{}: {}", entry.key(), entry.value())?;
        }
        Ok(())
    }
}
