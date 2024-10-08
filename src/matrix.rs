use anyhow::{anyhow, Result};
use std::{
    fmt,
    ops::{Add, AddAssign, Mul},
    sync::mpsc,
    thread,
};

use crate::{dot_product, Vector};

//4个线程
const NUM_THREADS: usize = 4;

//多线程出入参定义
pub struct MsgInput<T> {
    idx: usize,
    row: Vector<T>,
    col: Vector<T>,
}

impl<T> MsgInput<T> {
    fn new(idx: usize, row: Vector<T>, col: Vector<T>) -> Self {
        Self { idx, row, col }
    }
}

pub struct MsgOutput<T> {
    idx: usize,
    //返回累加和
    value: T,
}

//组装 输入消息和输出消息到一个数据结构中, reduce的时候可以获取到所有的信息
// 这个数据结构的返回应该是获取MsgOutput的流,
pub struct Msg<T> {
    input: MsgInput<T>,
    //sender to send the result back
    sender: oneshot::Sender<MsgOutput<T>>,
}

impl<T> Msg<T> {
    fn new(input: MsgInput<T>, sender: oneshot::Sender<MsgOutput<T>>) -> Self {
        Self { input, sender }
    }
}

//先声明一个矩阵结构
// 3*2 的矩阵 [[1,2], [3,4], [5,6]] => 也可以定义成 [1,2,3,4,5,6], 然后在代码中去分行列
//rust中,后面的形式要比前面的形式要好, 因为 Vec(Vec,Vec,Vec) 这种数据结构访问效率没有 直接平铺然后通过index进行逻辑区分的高
//定义一个数据结构, data数据的类型最好是泛型

//如果要自定义Matrix 的 debug内容, 则自行实现 display 和 debug trait, 这里就先注释掉
// #[derive(Debug)]
// pub struct Matrix<T: fmt::Debug> {
pub struct Matrix<T> {
    data: Vec<T>,
    row: usize,
    col: usize,
}

impl<T: fmt::Debug> Matrix<T> {
    //这里使用 impl Into<Vec<T>> , 表示的是 只要能转化成 Vec<T> 就可以作为参数传入
    pub fn new(data: impl Into<Vec<T>>, row: usize, col: usize) -> Self {
        Self {
            data: data.into(),
            row,
            col,
        }
    }
}

impl<T> Mul for Matrix<T>
where
    T: fmt::Debug + Add<Output = T> + Copy + AddAssign + Mul<Output = T> + Default + Send + 'static,
{
    type Output = Self;

    fn mul(self, rhs: Matrix<T>) -> Self::Output {
        multiply(&self, &rhs).expect("Matrix multiply error")
    }
}

pub fn multiply<T>(a: &Matrix<T>, b: &Matrix<T>) -> Result<Matrix<T>>
//对泛型T进行约束,
//简单的约束可以直接在 <T: ...> 中进行, 复杂的约束就再函数签名后面 加上 where
where
    T: fmt::Debug + Add<Output = T> + Copy + AddAssign + Mul<Output = T> + Default + Send + 'static,
{
    //这个边界值不懂
    if a.col != b.row {
        return Err(anyhow!(
            "Matrix dimensions do not match error: a.col != b.row"
        ));
    }

    //每个线程1个channel
    let senders = (0..NUM_THREADS)
        .map(|_| {
            let (tx, rx) = mpsc::channel::<Msg<T>>();
            thread::spawn(move || {
                for msg in rx {
                    //这里泛型T没有实现send trait, 无法安全的在线程中传递, 上面的where中添加Send+'static trait
                    let value = dot_product(msg.input.row, msg.input.col)?;
                    if let Err(e) = msg.sender.send(MsgOutput {
                        idx: msg.input.idx,
                        value,
                    }) {
                        eprintln!("Send error: {:?}", e);
                    }
                }
                //这里无法直接返回 OK(()))
                Ok::<_, anyhow::Error>(())
            });
            //这里返回tx, rx被必包到上面的线程处理中
            tx
        })
        //将sender收集起来
        .collect::<Vec<_>>();

    //这里确定结果的容量
    //这里我们要使用 mut 获取 data的所有权, 因为后面需要修改 data的内容, 有修改操作
    //这样写有个问题就是 生成的 Vec 元素 = 空, T泛型没有默认值处理
    // let mut data = Vec::with_capacity(a.row*b.col);
    let length = a.row * b.col;
    let mut data = vec![T::default(); length];

    //多线程返回多个rx，收集rx进行reduce
    let mut receives = Vec::with_capacity(length);

    //矩阵乘法算法
    //先遍历a的每一行, 再遍历b的每一列, 然后计算对应位置的乘积, 然后加到结果矩阵的对应位置上
    for i in 0..a.row {
        for j in 0..b.col {
            // 这一步用 dot_product 替代
            // for k in 0..a.col {
            //     // 这里的+= 实际上是 泛型T进行  += 操作, 需要定义明确的trait来实现这样的功能, 需要在上面的 where 中加入 AddAssign trait
            //     // 同时,要对data里面的数据结构Vec<T>进行访问, T同样需要满足借用规则, 这里对T实现copy trait, 在借用时如果不能借用,直接复制
            //     data[i * b.col + j] += a.data[i * a.col + k] * b.data[k * b.col + j];
            // }

            //取a的行值, 因为这里是 a矩阵的切片类型
            //切片类型本身不是一个 Vec<T>，而是一个指向 Vec<T> 内部元素的引用。
            //所以这里需要进行取引用符号
            let row = Vector::new(&a.data[a.col * i..a.col * (i + 1)]);
            //copied() 表示将引用里面的值转出值, 而不是引用本身
            let col_data = b.data[j..]
                .iter()
                .step_by(b.col)
                .copied()
                .collect::<Vec<T>>();
            let col = Vector::new(col_data);
            //这里改成多线程处理
            // data[i * b.col + j] += dot_product(row, col)?;

            // 每个都生成 msgInput 和  新增一个channel ，将 tx放入到msg中， rx保留做最后的reduce
            let idx = i * b.col + j;
            let msg_input = MsgInput::new(idx, row, col);
            let (tx, rx) = oneshot::channel();
            let msg = Msg::new(msg_input, tx);
            //sender返回的不是Result, 所以不能用？号
            if let Err(e) = senders[idx % NUM_THREADS].send(msg) {
                eprint!("Send error: {:?}", e)
            }
            receives.push(rx);
        }
    }

    //reduce rx结果
    for rx in receives {
        let recv = rx.recv()?;
        data[recv.idx] = recv.value
    }

    Ok(Matrix {
        data,
        col: b.col,
        row: a.row,
    })
}

//md, 这里居然不会提示我实现 fmt 方法, 只是报了一个错...垃圾
impl<T> fmt::Display for Matrix<T>
where
    T: fmt::Display,
{
    // generate code: display a 2x3 as {1 2 3, 4 5 6}, 3x2 as {1 2, 3 4, 5 6}
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //矩阵显示的算法
        write!(f, "{{")?;
        for i in 0..self.row {
            for j in 0..self.col {
                write!(f, "{}", self.data[i * self.col + j])?;
                if j != self.col - 1 {
                    write!(f, " ")?;
                }
            }
            if i != self.row - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "}}")?;
        Ok(())
    }
}

//实现 display后, 在实现debug trait, debug trait中的方法可以复用display的结果
impl<T> fmt::Debug for Matrix<T>
where
    //debug 复用 t的display 格式, 这里T显示指定需要实现 Display trait
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Matrix(row={}, col={}, {})", self.row, self.col, self)
    }
}

//实现一个简单的test
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_multiply() -> Result<()> {
        //这里无需用vec!宏 ,因为实现了into trait, 可以直接传入数组的引用
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4, 5, 6], 3, 2);
        let c = a * b;
        //进行这几个比较
        assert_eq!(c.row, 2);
        assert_eq!(c.col, 2);
        assert_eq!(c.data, [22, 28, 49, 64]);
        //这个是总的比较
        assert_eq!(format!("{:?}", c), "Matrix(row=2, col=2, {22 28, 49 64})");

        Ok(())
    }

    //在添加一个简单的测试, 测试矩阵的显示效果
    #[test]
    fn test_matrix_display() -> Result<()> {
        let a = Matrix::new([1, 2, 3, 4], 2, 2);
        let b = Matrix::new([1, 2, 3, 4], 2, 2);
        let c = a * b;
        assert_eq!(c.data, vec![7, 10, 15, 22]);
        assert_eq!(format!("{}", c), "{7 10, 15 22}");
        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_matrix_display_should_panic() {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4], 2, 2);
        let _matrix = a * b;
    }

    //直接测试 multiply 方法， 是否返回错误
    #[test]
    fn test_a_can_not_multiply_b() {
        let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
        let b = Matrix::new([1, 2, 3, 4], 2, 2);
        let c = multiply(&a, &b);
        assert!(c.is_err());
    }
}
