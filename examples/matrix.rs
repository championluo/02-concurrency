use concurrency::Matrix;

// 执行命令  cargo run --example matrix
fn main() -> anyhow::Result<()> {
    let a = Matrix::new([1, 2, 3, 4, 5, 6], 2, 3);
    let b = Matrix::new([1, 2, 3, 4, 5, 6], 3, 2);
    println!("a * b: {}", a * b);
    Ok(())
}
