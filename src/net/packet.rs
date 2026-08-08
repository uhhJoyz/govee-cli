use serde::{Deserialize, Serialize};
use serde_with::{BoolFromInt, serde_as};

#[derive(Serialize, Deserialize, Debug)]
pub struct GoveeDeviceInfo {
    pub ip: String,
    #[serde(alias = "device")]
    pub mac: String,
    pub sku: String,
    #[serde(alias = "bleVersionHard")]
    pub bt_hard_version: String,
    #[serde(alias = "bleVersionSoft")]
    pub bt_soft_version: String,
    #[serde(alias = "wifiVersionHard")]
    pub wifi_hard_version: String,
    #[serde(alias = "wifiVersionSoft")]
    pub wifi_soft_version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GoveeCmd {
    pub value: Option<u32>,
    pub color: Option<Color>,
    #[serde(alias = "colorTemInKelvin")]
    pub temp: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GoveeStatus {
    #[serde_as(as = "BoolFromInt")]
    #[serde(alias = "onOff")]
    pub is_on: bool,
    pub brightness: u16,
    pub color: Color,
    #[serde(alias = "colorTemInKelvin")]
    pub temp: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DevStatusCmd {
    #[serde(rename = "devStatus")]
    DevStatus,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DevScanCmd {
    #[serde(rename = "scan")]
    Scan,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum GoveeMessage {
    Handshake { cmd: DevScanCmd, data: GoveeDeviceInfo },
    Response { cmd: DevStatusCmd, data: GoveeStatus },
    // NOTE: the order in which these are defined matters
    // this commadn acts as a catch-all for anything that
    // does not fit into the buckets of the previous two
    // packet types
    Command { cmd: String, data: GoveeCmd },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GoveePacket {
    pub msg: GoveeMessage,
}
