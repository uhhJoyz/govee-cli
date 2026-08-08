use std::collections::HashMap;
use std::io;

use clap::Parser;
mod cli;
use cli::{Cli, Commands};

mod net;
use net::{packet::GoveeMessage, scan, status};

use crate::{
    cli::alias::{Light, Lights, name_devices, parse_name_or_ip, read_aliases, write_new_aliases},
    net::{packet::Color, set_brightness, set_color, set_on_off},
};

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let lights: HashMap<String, Light> = read_aliases()?;

    match &cli.command {
        Some(Commands::FindNew { auto }) => {
            let discovered_devices = scan().await?;
            let newly_added_devices = name_devices(discovered_devices, &lights, *auto).await?;

            let num_newly_added = newly_added_devices.len();
            let dev_names: Vec<String> =
                newly_added_devices.iter().map(|l| l.name.clone()).collect();
            if !newly_added_devices.is_empty() {
                let lights: Lights = Lights {
                    light: Some(newly_added_devices),
                };
                let new_aliases = toml::to_string(&lights)
                    .expect("Serialization of new devices failed. Try using different names.");
                write_new_aliases(&new_aliases)?;
            }
            println!("Added {} devices.", num_newly_added);
            if *auto && !dev_names.is_empty() {
                println!("Devices added with names: {:?}", dev_names);
            }
        }
        Some(Commands::RemoveAlias) => {
            todo!("Need to implement removing aliases.")
        }
        Some(Commands::List) => {
            let discovered_devices = scan().await?;
            let light_by_ip: HashMap<String, Light> =
                lights.values().map(|l| (l.ip.clone(), l.clone())).collect();

            println!("Found {} devices on LAN.", discovered_devices.len());
            for (i, p) in discovered_devices.iter().enumerate() {
                match &p.msg {
                    GoveeMessage::Handshake { cmd: _, data } => {
                        println!("Device {}:", i);
                        println!(
                            "  alias: {}",
                            light_by_ip
                                .get(&data.ip)
                                .map_or(String::from("Not previously discovered."), |l| l
                                    .name
                                    .clone())
                        );
                        println!("  ip: {}", data.ip);
                        println!("  mac: {}", data.mac);
                        println!("  sku: {}", data.sku);
                        println!(
                            "  bluetooth version: {} (hardware) / {} (software)",
                            data.bt_hard_version, data.bt_soft_version
                        );
                        println!(
                            "  wifi version: {} (hardware) / {} (software)",
                            data.wifi_hard_version, data.wifi_soft_version
                        );
                    }
                    _ => {
                        unreachable!("Non-response found in device list.");
                    }
                }
            }
        }
        Some(Commands::ListAliases) => {
            for l in lights.values() {
                println!("{}", l.name);
            }
        }
        Some(Commands::Status { ip_or_alias }) => {
            let (name, ip) = parse_name_or_ip(&lights, ip_or_alias)?;
            let stat = status(&ip).await?;
            match &stat.msg {
                GoveeMessage::Response { cmd, data } => {
                    println!("Device '{name}' status:");
                    println!("  on: {}", data.is_on);
                    println!("  brightness: {}", data.brightness);
                    println!("  color: {:?}", data.color);
                    println!("  temp: {}", data.temp);
                }
                response => {
                    println!("Response: {response:?}");
                    unreachable!("Response packet was not a status packet.");
                }
            }
        }
        Some(Commands::Brightness {
            ip_or_alias,
            brightness,
        }) => {
            let (_, ip) = parse_name_or_ip(&lights, ip_or_alias)?;
            set_brightness(&ip, brightness).await?;
        }
        Some(Commands::Power { ip_or_alias, state }) => {
            let (_, ip) = parse_name_or_ip(&lights, ip_or_alias)?;
            set_on_off(
                &ip,
                &match state {
                    cli::PowerState::Off => 0,
                    cli::PowerState::On => 1,
                },
            )
            .await?;
        }
        Some(Commands::Color {
            ip_or_alias,
            r: Some(r),
            g: Some(g),
            b: Some(b),
            temp,
            hex: _,
        }) => {
            let (_, ip) = parse_name_or_ip(&lights, ip_or_alias)?;
            set_color(
                &ip,
                Color {
                    r: *r,
                    g: *g,
                    b: *b,
                },
                *temp,
            )
            .await?;
        }
        Some(Commands::Color {
            ip_or_alias,
            r: None,
            g: None,
            b: None,
            temp,
            hex: Some(hex),
        }) => {
            let (_, ip) = parse_name_or_ip(&lights, ip_or_alias)?;
            let hex = hex.trim_start_matches('#');
            let r: u8;
            let g: u8;
            let b: u8;
            match hex.len() {
                6 => {
                    r = u8::from_str_radix(&hex[0..2], 16).expect("Failed to parse red from hex.");
                    g = u8::from_str_radix(&hex[2..4], 16)
                        .expect("Failed to parse green from hex.");
                    b = u8::from_str_radix(&hex[4..6], 16).expect("Failed to parse blue from hex.");
                }
                3 => {
                    r = u8::from_str_radix(&hex[0..1], 16).expect("Failed to parse red from hex.")
                        * 16;
                    g = u8::from_str_radix(&hex[1..2], 16)
                        .expect("Failed to parse green from hex.")
                        * 16;
                    b = u8::from_str_radix(&hex[2..3], 16).expect("Failed to parse blue from hex.")
                        * 16;
                }
                _ => {
                    panic!(
                        "Failed to read hex value. Reason: invalid length ({}, must be 3 or 6 characters)",
                        hex.len()
                    );
                }
            }
            set_color(&ip, Color { r, g, b }, *temp).await?;
        }
        Some(Commands::Color { .. }) => unreachable!(
            "Color definition should be protected via CLI, malformed color command found."
        ),
        None => println!("Run this command with the --help flag for usage information."),
    }

    Ok(())

    // loop {
    //     send_sock
    //         .send_to(MSG_SCAN.as_bytes(), (multicast_addr, 4001))
    //         .await?;
    // }
}
