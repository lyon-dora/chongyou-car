use std::{
    env,
    fs::File,
    io::{Read, Write},
    path::Path,
    thread,
    time::Duration,
};

use clap::Parser;
use enums::CommandType;
use error::Error;
use json_data::JsonData;
use serial::SerialPort;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc,
};

mod command;
mod config;
mod enums;
mod error;
mod json_data;

/// Rebot command from json file, By: Ji An
#[derive(Debug, Parser)]
// #[clap(
//     name = "robot",
//     version = "1.0.0",
//     author = "技安",
//     about = "使用json文件批量测试机械臂运动",
//     arg_required_else_help(true)
// )]
#[command(version, about, long_about = None, arg_required_else_help(true))]
struct Cli {
    /// json file required
    #[arg(short, long)]
    file: String,

    /// ip (eg: 192.168.0.10)
    #[arg(long)]
    ip: Option<String>,

    /// port (eg: 8080)
    #[arg(long)]
    port: Option<u16>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let cli = Cli::parse();

    let file = Path::new(&cli.file);

    if !file.is_file() {
        return Err(Error::JsonDataEmpty.into());
    }

    let mut body = File::open(file).map_err(|_| Error::JsonDataEmpty)?;

    let mut json = String::new();
    body.read_to_string(&mut json)
        .map_err(|_| Error::ReadJsonFail)?;

    println!("read json file content: {}", json);

    let json_data =
        serde_json::from_str::<Vec<JsonData>>(&json).map_err(|_| Error::DecodeJsonFail)?;
    println!("{:?}", json_data);

    // robot ip
    // let robot_ip = if let Some(ip) = cli.ip {
    //     ip
    // } else {
    //     env::var(config::ROBOT_IP)?
    // };
    // // robot port
    // let robot_port = if let Some(port) = cli.port {
    //     port
    // } else {
    //     env::var(config::ROBOT_PORT)?
    //         .parse()
    //         .map_err(|_| Error::PortFail)?
    // };

    // serial port
    let serial_port = env::var(config::SERIAL_PORT).map_err(|_| Error::SerialPortFail)?;

    // 连接串口
    let mut com = serial::open(&serial_port).map_err(|_| Error::SerialConnectFail)?;

    com.reconfigure(&|settings| {
        settings.set_baud_rate(serial::Baud115200)?;
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
    })
    .map_err(|_| Error::SerialSettingsSetFail)?;
    com.set_timeout(Duration::from_millis(1000))
        .map_err(|_| Error::SerialSetTimeoutFail)?;

    // 消息通道
    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(100);

    // 读取文件， 后期读取dora数据流
    tokio::spawn(async move {
        for json in json_data {
            thread::sleep(Duration::from_millis(json.sleep_second));

            let data = match json.command {
                // 差速小车
                CommandType::DifferSpeed { x, y, w } => {
                    // println!("send_speed_to_x4chassis: {x}, {y}, {w}");
                    command::send_speed_to_x4chassis(x, y, w)
                }
            };

            tx.send(data).await.ok();
        }
    });

    while let Some(data) = rx.recv().await {
        com.write_all(&data).ok();
    }

    Ok(())
}
