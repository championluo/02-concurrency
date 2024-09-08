use std::ops::{Add, AddAssign, Deref, Mul};

use anyhow::anyhow;
use anyhow::Result;

pub struct Vector<T> {
    data: Vec<T>,
}

impl<T> Deref for Vector<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        // &self.data 就是 data: Vec<T>
        &self.data
    }
}

// 无需下面的代码, 因为 Vector<T> 已经实现了Deref trait, 所以可以直接调用deref方法
//处理对数据结构生成new 方法, 还提供了len 和 iter 方法, 用于获取数据长度和迭代器
impl<T> Vector<T> {
    pub fn new(data: impl Into<Vec<T>>) -> Self {
        Self { data: data.into() }
    }

    //     pub fn len(&self) -> usize {
    //         self.data.len()
    //     }

    //     pub fn iter(&self) -> Iter<T> {
    //         self.data.iter()
    //     }s
}

//点乘方法, 相同长度的数字,相同位置相乘的结果进行累加, 最后返回累加值
// 这里对 入参进行封装, 自定义Vector类型
pub fn dot_product<T>(a: Vector<T>, b: Vector<T>) -> Result<T>
where
    T: Copy + Default + Add<Output = T> + AddAssign + Mul<Output = T>,
{
    if a.len() != b.len() {
        return Err(anyhow!("Dot product error: a.len != b.len"));
    }

    let mut sum = T::default();
    for i in 0..a.len() {
        sum += a[i] * b[i];
    }

    Ok(sum)
}
