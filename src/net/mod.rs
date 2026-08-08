mod msg;
use msg::MSG_SCAN;
pub mod packet;
use packet::GoveePacket;

use std::{
    io, net::Ipv4Addr, sync::{Arc, atomic::{AtomicBool, Ordering}}, thread, time::Duration,
};
use tokio::net::UdpSocket;

use crate::net::{
    msg::MSG_STATUS,
    packet::{Color, GoveeCmd},
};

pub async fn pairing_animation(ip: &str, end: Arc<AtomicBool>) -> io::Result<()> {
    // we do this manually to prevent repeatedly binding to sockets on the device
    let send_sock = UdpSocket::bind("0.0.0.0:0").await?;
    let send = async |packet: &GoveePacket| -> io::Result<()> {
        send_sock
            .send_to(
                serde_json::to_string(&packet)
                    .expect("Failed to serialize brightness message.")
                    .as_bytes(),
                (ip, 4003),
            )
            .await?;

        Ok(())
    };

    let msg_red = GoveePacket {
        msg: packet::GoveeMessage::Command {
            cmd: String::from("colorwc"),
            data: GoveeCmd {
                value: None,
                color: Some(Color { r: 255, g: 0, b: 0 }),
                temp: None,
            },
        },
    };

    let msg_blue = GoveePacket {
        msg: packet::GoveeMessage::Command {
            cmd: String::from("colorwc"),
            data: GoveeCmd {
                value: None,
                color: Some(Color { r: 0, g: 0, b: 255 }),
                temp: None,
            },
        },
    };

    let msg_off = GoveePacket {
        msg: packet::GoveeMessage::Command {
            cmd: String::from("turn"),
            data: GoveeCmd {
                value: Some(0),
                color: None,
                temp: None,
            },
        },
    };
    let msg_on = GoveePacket {
        msg: packet::GoveeMessage::Command {
            cmd: String::from("turn"),
            data: GoveeCmd {
                value: Some(1),
                color: None,
                temp: None,
            },
        },
    };
    send(&msg_on).await?;

    loop {
        send(&msg_red).await?;
        thread::sleep(Duration::from_secs(1));
        if end.load(Ordering::Relaxed) {
            send(&msg_off).await?;
            break;
        }
        send(&msg_blue).await?;
        thread::sleep(Duration::from_secs(1));
        if end.load(Ordering::Relaxed) {
            send(&msg_off).await?;
            break;
        }
    }

    Ok(())
}

async fn send_cmd_packet(ip: &str, packet: GoveePacket) -> io::Result<()> {
    let send_sock = UdpSocket::bind("0.0.0.0:0").await?;

    send_sock
        .send_to(
            serde_json::to_string(&packet)
                .expect("Failed to serialize brightness message.")
                .as_bytes(),
            (ip, 4003),
        )
        .await?;

    Ok(())
}

pub async fn set_color(ip: &str, color: Color, temp: Option<u32>) -> io::Result<()> {
    let msg_color = GoveePacket {
        msg: packet::GoveeMessage::Command {
            cmd: String::from("colorwc"),
            data: GoveeCmd {
                value: None,
                color: Some(color),
                temp,
            },
        },
    };

    send_cmd_packet(ip, msg_color).await?;
    Ok(())
}

pub async fn set_on_off(ip: &str, state: &u32) -> io::Result<()> {
    let msg_on_off = GoveePacket {
        msg: packet::GoveeMessage::Command {
            cmd: String::from("turn"),
            data: GoveeCmd {
                value: Some(*state),
                color: None,
                temp: None,
            },
        },
    };

    send_cmd_packet(ip, msg_on_off).await?;
    Ok(())
}

pub async fn set_brightness(ip: &str, brightness: &u32) -> io::Result<()> {
    let msg_brightness = GoveePacket {
        msg: packet::GoveeMessage::Command {
            cmd: String::from("brightness"),
            data: GoveeCmd {
                value: Some(*brightness.clamp(&0u32, &100u32)),
                color: None,
                temp: None,
            },
        },
    };

    send_cmd_packet(ip, msg_brightness).await?;
    Ok(())
}

pub async fn status(ip: &str) -> io::Result<GoveePacket> {
    // create udp sockets
    let send_sock = UdpSocket::bind("0.0.0.0:0").await?;
    let recv_sock = UdpSocket::bind("0.0.0.0:4002").await?;

    // initialize a transfer buffer
    let mut buf = [0; 2048];

    send_sock.send_to(MSG_STATUS.as_bytes(), (ip, 4003)).await?;

    let (mut msg_len, mut src_ip) = recv_sock.recv_from(&mut buf).await?;

    // TODO: find a more elegant way to handle this
    // TODO: also add timeout in case of wrong ip entry
    while src_ip.ip().to_string() != ip {
        (msg_len, src_ip) = tokio::time::timeout(
            std::time::Duration::from_millis(150),
            recv_sock.recv_from(&mut buf),
        )
        .await?
        .expect("Response not received from queried device.");
    }

    let pkt: GoveePacket = serde_json::from_str(
        str::from_utf8(&buf[..msg_len]).expect("Received unexpected data, not in UTF-8 form."),
    )
    .expect("Could not deserialize response.");

    Ok(pkt)
}

pub async fn scan() -> io::Result<Vec<GoveePacket>> {
    // create udp sockets
    let send_sock = UdpSocket::bind("0.0.0.0:4001").await?;
    let recv_sock = UdpSocket::bind("0.0.0.0:4002").await?;

    // define and connect to the standard
    // multicast address of govee products
    let multicast_addr: Ipv4Addr = Ipv4Addr::new(239, 255, 255, 250);
    // port: 4001
    recv_sock.join_multicast_v4(multicast_addr, Ipv4Addr::UNSPECIFIED)?;

    // initialize a transfer buffer
    let mut buf = [0; 2048];

    send_sock
        .send_to(MSG_SCAN.as_bytes(), (multicast_addr, 4001))
        .await?;

    let mut discovery_packets: Vec<GoveePacket> = Vec::new();

    loop {
        // TODO: make the length of timeout configurable
        let maybe_response =
            tokio::time::timeout(Duration::from_millis(150), recv_sock.recv_from(&mut buf)).await;
        match maybe_response {
            Ok(Ok((msg_len, _src_ip))) => {
                discovery_packets.push(
                    serde_json::from_str(
                        str::from_utf8(&buf[..msg_len])
                            .expect("Received unexpected data, not in UTF-8 form."),
                    )
                    .expect("Could not deserialize response."),
                );
            }
            // TODO: should this be silently ignored?
            Ok(Err(_)) => panic!("Socket error, could not receive data."),
            Err(_) => {
                break;
            }
        }
    }

    Ok(discovery_packets
        .into_iter()
        .filter(|pkt| matches!(pkt.msg, packet::GoveeMessage::Handshake { .. }))
        .collect())
}
