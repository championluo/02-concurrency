use anyhow::{anyhow, Result};
use std::{
    // rc::Rc,
    sync::mpsc::{self, channel},
    thread,
    time::Duration,
};

const NUM_PRODUCERS: usize = 4;

// 使用 #[warn(dead_code)] 忽略未使用的变量告警
#[allow(dead_code)]
#[derive(Debug)]
struct Msg {
    index: usize,
    random: usize,
}

impl Msg {
    fn new(index: usize, random: usize) -> Self {
        Self { index, random }
    }
}

//cargo add anyhow;
// main函数先使用helloworld , 还行 cargo run --example thread1 执行一下, 调通代码先
fn main() -> Result<()> {
    //这里example 使用 mpsc 的 channel 特性， 因为 channel 的tx 可以clone 无数次， 然后往channel发数据
    // receive 只能有1个 ，形成生产者-消费者模型
    let (tx, rx) = channel();

    //创建producer
    for index in 0..NUM_PRODUCERS {
        let tx = tx.clone();
        thread::spawn(move || producer(tx, index));
    }

    //因为后面增加了tx结束动作， consumer 发现如果发现所有的tx都关闭了，没有数据， 也会退出线程
    //上面的tx都是clone的， 最原始的tx还没有销毁，consumer永远也不会退出
    //使用 drop 方法， 直接进行释放
    drop(tx);

    //创建consumer
    //这里rx会隐式的move到闭包中，后面无法直接使用 rx
    let consumer = thread::spawn(|| {
        for msg in rx {
            println!(" received: {:?}", msg);
        }
        //没有消息后，退出线程
        println!("consumer exit");
        //consumer 在线程结束（消费结束）的时候，返回一个值
        42
        //但是这个返回值必须是 'static + Send 类型的，部分引用类型的数据结构是无法作为返回值的
        //比如如下 的 Rc 没有实现 Send trait， 所以无法作为返回值
        // let rc = Rc::new("secret".to_string());
        // rc
    });

    //这里无法使用rx的
    // rx.recv();

    let consumer_return = consumer
        .join()
        // 当我们没办法用？将一种错误转换成另一种错误的时候，就需要显示的进行错误的转换
        // 这里使用 map_err() 方法来转换错误类型
        //这里 e 实现了 debug  trait，可以直接打印出来
        .map_err(|e| anyhow!("Consumer thread panicked: {:?}", e))?;

    println!("consumer return {}", consumer_return); // 42

    Ok(())
}

fn producer(tx: mpsc::Sender<Msg>, index: usize) -> Result<()> {
    //每个produce 都循环发送一个随机数
    loop {
        let random = rand::random::<usize>();
        //这里返回个对象， 不用unwrap()
        // tx.send(random)?;
        //改成返回一个对象
        tx.send(Msg::new(index, random))?;
        // thread::sleep(Duration::from_millis(1000));
        //随机时间出数据, 这里必须要转成u64， 因为 u8 * 10 的最大值无法用 u8 装下了，u8最大255，u8*10 = 2550
        let random_sleep_time = rand::random::<u8>() as u64 * 10;
        //函数入参是可以进行显示的类型转换， as _ 表示由编译器自行推导类型， 必须显示转换，rust的规则
        thread::sleep(Duration::from_millis(random_sleep_time));
        //这里增加一个结束流程
        if rand::random::<u8>() % 5 == 0 {
            println!("producer {} exit", index);
            break;
        }
    }
    Ok(())
}
