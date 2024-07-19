use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("json file is not exists")]
    JsonDataEmpty,
    #[error("read json error")]
    ReadJsonFail,
    #[error("decode json file fail, please check json field")]
    DecodeJsonFail,
    #[error("tcp port error")]
    PortFail,
    #[error("serial port error")]
    SerialPortFail,
    #[error("serial connect fail")]
    SerialConnectFail,
    #[error("serial settings set fail")]
    SerialSettingsSetFail,
    #[error("serial set timeout fail")]
    SerialSetTimeoutFail,
}
