use std::net::SocketAddr;
use tokio::io;
use tokio::net::{TcpStream, TcpListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::env;
use chrono::Local;


#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        panic!("请正确输入启动参数！");
    }

    let local_addr = args[1].clone();
    let server_addr = args[2].clone();
    let now = Local::now();
    println!("{} 代理程序启动！监听地址:{};目标服务器地址:{}",now, local_addr, server_addr);


    let listener = TcpListener::bind(local_addr).await.expect("无法监听该端口！");
    loop {
        let (mut socket, addr) = listener.accept().await.expect("系统故障无法继续监听！");
        match tcpproxy(socket, addr, server_addr.clone()).await {
            Ok(_) => {}
            Err(e) => {
                let now = Local::now();
                println!("{} 和服务端连接出现错误！:{}",now,e);
            }
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