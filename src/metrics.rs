use std::collections::HashMap;

// metrics 的 data structure
#[derive(Debug, Clone)]
pub struct Metrics {
    //key 和 value 的类型都可以自定义\
    //这里如果类型写死的话, 就不需要再结构体签名上加泛型<>符号了
    data: HashMap<String, i64>,
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
            data: HashMap::new(),
        }
    }

    pub fn inc(&mut self, key: impl Into<String>) {
        let count = self.data.entry(key.into()).or_insert(0);
        *count += 1
    }

    pub fn dec(&mut self, key: impl Into<String>) {
        let count = self.data.entry(key.into()).or_insert(0);
        *count -= 1
    }

    pub fn snapshot(&self) -> HashMap<String, i64> {
        self.data.clone()
    }
}
