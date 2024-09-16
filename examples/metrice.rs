use anyhow::Result;
use concurrency::Metrics;
use rand::Rng;
use std::{thread, time::Duration};

const N: usize = 2;
const M: usize = 4;

fn main() -> Result<()> {
    let metrics = Metrics::new();

    println!("{:?}", metrics);

    // 将上面的代码改成多线程处理
    for _ in 0..N {
        request_worker(metrics.clone());
    }
    for i in 0..M {
        task_worker(i, metrics.clone());
    }

    //这里也改成3s打印一次
    loop {
        thread::sleep(Duration::from_secs(3));
        //print! 宏不会自动换行，这里一定要换行,否则不会立即打印,直到一行溢出
        println!("{:?}", metrics);
    }
}

fn task_worker(idx: usize, mut metrics: Metrics) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(
            rand::thread_rng().gen_range(100..5000),
        ));
        metrics.inc(format!("call.thread.worker.{}", idx));
    });
}

fn request_worker(mut metrics: Metrics) {
    thread::spawn(move || loop {
        let rng = &mut rand::thread_rng();
        thread::sleep(Duration::from_millis(rng.gen_range(50..800)));
        let page = rng.gen_range(0..256);
        metrics.inc(format!("request.page.{}", page));
    });
}
