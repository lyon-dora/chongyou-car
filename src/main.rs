use std::{
    env,
    fs::File,
    io::{Read, Write},
    path::Path,
    thread,
    time::Duration,
};

use clap::Parser;
use device_query::{DeviceQuery, DeviceState, Keycode};
use enums::CommandType;
use error::Error;
use json_data::JsonData;
use serial::SerialPort;
use tokio::sync::mpsc;

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

    // serial port
    let serial_port = env::var(config::SERIAL_PORT).map_err(|_| Error::SerialPortFail)?;

    // 连接串口
    const COM_SETTINGS: serial::PortSettings = serial::PortSettings {
        baud_rate: serial::Baud115200,
        char_size: serial::Bits8,
        parity: serial::ParityNone,
        stop_bits: serial::Stop1,
        flow_control: serial::FlowNone,
    };

    let mut com = serial::open(&serial_port).map_err(|_| Error::SerialConnectFail)?;
    com.configure(&COM_SETTINGS)
        .map_err(|_| Error::SerialSettingsSetFail)?;
    com.set_timeout(Duration::from_millis(1000))
        .map_err(|_| Error::SerialSetTimeoutFail)?;

    // 消息通道
    let (tx_key, mut rx_key) = mpsc::channel::<(f64, f64)>(100);

    tokio::spawn(async move {
        while let Some((x, w)) = rx_key.recv().await {
            let data = command::send_speed_to_x4chassis(x, 0.0, w);
            com.write_all(&data).ok();
        }
    });

    let mut r = 1.0;
    let device_state = DeviceState::new();
    let mut prev_keys = vec![];
    loop {
        let keys = device_state.get_keys();
        // if !keys.is_empty() {
        // match keys[0] {
        //     Keycode::W => {
        //         tx_key.send((0.1, 0.0)).await.ok();
        //     }
        //     Keycode::A => {
        //         tx_key.send((0.0, 0.1)).await.ok();
        //     }
        //     Keycode::S => {
        //         tx_key.send((0.0, 0.0)).await.ok();
        //     }
        //     Keycode::D => {
        //         tx_key.send((0.0, -0.1)).await.ok();
        //     }
        //     Keycode::X => {
        //         tx_key.send((-0.1, 0.0)).await.ok();
        //     }
        //     _ => {}
        // }
        // println!("{:?}", keys);
        // }
        if keys != prev_keys {
            // println!("{:?}", keys);
            if !keys.is_empty() {
                match keys[0] {
                    Keycode::W => {
                        tx_key.send((0.2 * r, 0.0)).await.ok();
                    }
                    Keycode::A => {
                        tx_key.send((0.0, 0.2 * r)).await.ok();
                    }
                    Keycode::S => {
                        tx_key.send((0.0, 0.0)).await.ok();
                    }
                    Keycode::D => {
                        tx_key.send((0.0, -0.2 * r)).await.ok();
                    }
                    Keycode::X => {
                        tx_key.send((-0.2 * r, 0.0)).await.ok();
                    }
                    Keycode::Key1 => r = 1.0,
                    Keycode::Key2 => r = 2.0,
                    Keycode::Key3 => r = 3.0,
                    _ => {}
                }
                // println!("{:?}", keys);
            }
        }
        prev_keys = keys;
    }
}
