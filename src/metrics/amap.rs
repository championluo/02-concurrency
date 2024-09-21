use std::{
    collections::HashMap,
    fmt,
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
};

#[derive(Debug)]
pub struct AmapMetrics {
    //这里key 的类型是 &'static str 是因为 监控指标通常在设计初期我们就已经确定我们要监控什么, 所以key是可以预先预知的;
    // 键为 &'static str，意味着键是不可变的，一旦插入到 HashMap 中，键不能被修改。
    // 键类型为 &'static str，表示键是一个静态生命周期的字符串引用。这意味着键必须是静态分配的字符串，通常是字面量字符串
    // 使用静态生命周期的字符串引用可以避免动态分配字符串，从而提高性能。但是这也限制了键只能是常量字符串。
    data: Arc<HashMap<&'static str, AtomicI64>>,
}

//这里仅演示如何实现Clone trait, 当然也可以在上面的derive上标注 Clone trait
impl Clone for AmapMetrics {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl AmapMetrics {
    //这里 metrics_names 是个可以迭代的静态字符串数组, 相当于预先初始化所有指标的 key, value 可以都=0
    pub fn new(metrics_names: &[&'static str]) -> Self {
        let map = metrics_names
            .iter()
            //迭代成Hashmap, 需要将key, value 组成一个 tuple
            .map(|&name| (name, AtomicI64::new(0)))
            //这里不使用 下面这种写法, 因为rust会自行推断map中的类型
            // .collect::<HashMap<_,_>>();
            .collect();
        AmapMetrics {
            data: Arc::new(map),
        }
    }

    pub fn inc(&self, key: impl AsRef<str>) -> anyhow::Result<()> {
        let value = self
            .data
            .get(key.as_ref())
            .ok_or_else(|| anyhow::anyhow!("key {} not found", key.as_ref()))?;
        //这里使用 Relaxed , 无需对优先级进行额外处理
        //fetch_add 先读后加, 原子操作
        value.fetch_add(1, Ordering::Relaxed);
        anyhow::Ok(())
    }
}

impl fmt::Display for AmapMetrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (key, value) in self.data.iter() {
            writeln!(f, "{}: {}", key, value.load(Ordering::Relaxed))?;
        }
        Ok(())
    }
}
