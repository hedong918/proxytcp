use std::collections::HashMap;
use std::net::IpAddr;
use std::net::SocketAddr;
use tokio::io;
use tokio::net::{TcpStream, TcpListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::env;
use chrono::Local;


#[tokio::main]
async fn main() -> io::Result<()> {

    //获取命令行参数
    let mut args: Vec<String> = env::args().collect();

    //校验参数数量
    if args.len() < 4 {
        panic!("请正确输入启动参数！");
    }

    //获取本机监听地址
    let local_addr = args[1].clone();
    
    //获取代理类型
    let proxy_mod = match args[2].clone().as_str() {
        "iphash" => {Mode::Iphash} ,
        "poll" => {Mode::Poll},
        _ => {
            panic!("代理类型输入错误，可用参数：iphash、poll！");
        },

    };

    //获取服务器组
    let mut server_addrs:Vec<String> =vec![];
    
    for i in args[3..].iter(){
        server_addrs.push(i.to_string());
    }

    let mut num = 0;
    let mut map = HashMap::new();


    //打印启动信息
    let now = Local::now();
    println!("{} 代理程序启动！监听地址:{};目标服务器地址:{:?}",now, local_addr, server_addrs);

    //开始监听本地地址
    let listener = TcpListener::bind(local_addr).await.expect("无法监听该端口！");

    //进入循环处理状态
    loop {
        //获取客户端连接
        let (mut socket, addr) = listener.accept().await.expect("系统故障无法继续监听！");


        //根据代理类型获取服务器地址
        let server_addr = choose_server(&proxy_mod,&server_addrs,&mut num,&mut map,addr);

        //进行数据转发
        match tcpproxy(socket, addr, server_addr).await {
            Ok(_) => {},
            Err(e) => {
                let now = Local::now();
                println!("{} 和服务端连接出现错误！:{}",now,e);
            },
        }
    }
}

async fn tcpproxy(mut socket: TcpStream, addr: SocketAddr, server_addr: String) -> io::Result<()> {
    //let (mut rd1,mut wr1) = io::split(socket);
    let mut stream = TcpStream::connect(server_addr.clone()).await?;
    //let (mut rd2,mut wr2) = io::split(stream);

    println!("双向连接已建立，开始转发！src:{}-->des:{}", addr, server_addr);

    tokio::spawn(async move {
        io::copy_bidirectional(&mut socket, &mut stream).await.unwrap();
    });

    // tokio::spawn(async move {
    //     if io::copy(&mut rd2, &mut wr1).await.is_err() {
    //         eprintln!("failed to copy2");
    //     }
    // });

    Ok(())
}

enum Mode {
    Iphash,
    Poll,
}


fn choose_server(mode:&Mode,addrs:&Vec<String>,num:&mut usize,map:&mut HashMap<IpAddr,usize>,addr:SocketAddr) -> String {
    match mode {
        Mode::Iphash => {
            let ip = addr.ip();
            match map.get(&ip) {
                Some(x) => {
                    addrs[*x].clone()
                },
                None => {
                    let max = addrs.len() -1;
                    if *num < max {
                        *num += 1;
                        map.insert(ip, *num);
                        addrs[(*num-1)].clone()
                    }else {
                        *num = 0;
                        map.insert(ip, max);
                        addrs.last().unwrap().clone()
                    }
                },
            }
        },
        Mode::Poll => {
            let max = addrs.len() -1;
            if *num < max {
                *num += 1;
                addrs[(*num-1)].clone()
            }else {
                *num = 0;
                addrs.last().unwrap().clone()
            }
        },
    }
}
