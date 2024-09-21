use anyhow::Result;
use concurrency::AmapMetrics;
use rand::Rng;
use std::{thread, time::Duration};

const N: usize = 2;
const M: usize = 4;

//执行命令行: cargo run --example ametrics
fn main() -> Result<()> {
    //这里要先进行key的初始化, 注意是 数组的应用 要加个 &
    let metrics = AmapMetrics::new(&[
        "request.page.1",
        "request.page.2",
        "request.page.3",
        "request.page.4",
        "call.thread.worker.0",
        "call.thread.worker.1",
        "call.thread.worker.2",
        "call.thread.worker.3",
    ]);

    //使用 DashMap 后这里无需再进行 snapshot, 直接打印数据
    println!("{}", metrics);

    // 将上面的代码改成多线程处理
    for _ in 0..N {
        request_worker(metrics.clone())?; //clone原理 Metrics {data: Arc::clone(&metrics.data)}
    }
    for i in 0..M {
        task_worker(i, metrics.clone())?;
    }

    //这里也改成3s打印一次
    loop {
        thread::sleep(Duration::from_secs(3));
        //print! 宏不会自动换行，这里一定要换行,否则不会立即打印,直到一行溢出
        //这里直接改成输出metrics, 但是这里还没有实现fmt::Display, 所以要先实现Display
        println!("{}", metrics.clone());
    }
}

fn task_worker(idx: usize, metrics: AmapMetrics) -> Result<()> {
    thread::spawn(move || {
        loop {
            let mut rng = rand::thread_rng();
            thread::sleep(Duration::from_millis(rng.gen_range(100..5000)));
            metrics.inc(format!("call.thread.worker.{}", idx))?;
        }
        #[allow(unreachable_code)]
        Ok::<_, anyhow::Error>(())
    });
    Ok(())
}

fn request_worker(metrics: AmapMetrics) -> Result<()> {
    //如果使用 move || loop {}  这种简写格式的话， 返回类型是元组，不是Result类型，所以loop中的？号会报错，返回格式不一样
    thread::spawn(move || {
        loop {
            let rng = &mut rand::thread_rng();
            thread::sleep(Duration::from_millis(rng.gen_range(50..800)));
            let page = rng.gen_range(0..5);
            //这里返回？号，就说明整个函数返回的是Reult类型
            metrics.inc(format!("request.page.{}", page))?;
        }
        //在函数的最后，要加上OK，否则编译器会报错
        //打上这个标注，告诉编译器，函数的返回值是Result类型，但是实际上并没有返回Err，所以可以忽略这个错误
        #[allow(unreachable_code)]
        Ok::<_, anyhow::Error>(())
    });
    Ok(())
}
