use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::{self, Read, Write, stdin, stdout},
    net::Ipv4Addr,
    path::Path,
    sync::{Arc, atomic::AtomicBool},
};

use crate::net::{
    packet::{GoveeMessage, GoveePacket},
    pairing_animation,
};

#[derive(Debug)]
enum ConfigError {
    AliasLookUpError(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Light {
    pub name: String,
    pub ip: String,
    pub mac: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Lights {
    pub light: Option<Vec<Light>>,
}

pub fn read_aliases() -> io::Result<HashMap<String, Light>> {
    // set up our paths
    let cfg_path = Path::new(&std::env::var("HOME").expect(
        "Could not retrieve $HOME environment variable. Please ensure that it is defined to use configuration.",
    ))
    .join(Path::new(".config/govee-cli"));
    let alias_path = cfg_path.join(Path::new("aliases.toml"));

    // create them if they do not exist
    if !cfg_path.exists() {
        fs::create_dir_all(cfg_path).expect("Failed to create config directory.");
    }
    if !alias_path.exists() {
        fs::File::create(alias_path.clone())?;
    }

    // read our aliases
    let mut aliases_file = fs::File::open(alias_path)?;
    let mut aliases_string = String::new();
    aliases_file.read_to_string(&mut aliases_string)?;
    let lights: Lights =
        toml::from_str(&aliases_string.to_owned()).expect("Failed to read config file.");

    match lights.light {
        Some(l) => Ok(l
            .into_iter()
            .map(|light| (light.name.clone(), light))
            .collect::<HashMap<String, Light>>()),
        None => Ok(HashMap::new()),
    }
}

pub fn write_new_aliases(aliases_string: &str) -> io::Result<()> {
    // set up our paths
    let cfg_path = Path::new(&std::env::var("HOME").expect(
        "Could not retrieve $HOME environment variable. Please ensure that it is defined to use configuration.",
    ))
    .join(Path::new(".config/govee-cli"));
    let alias_path = cfg_path.join(Path::new("aliases.toml"));

    // create them if they do not exist
    if !cfg_path.exists() {
        fs::create_dir_all(cfg_path).expect("Failed to create config directory.");
    }
    if !alias_path.exists() {
        fs::File::create(alias_path.clone())?;
    }

    // write our new aliases
    let mut aliases_file = OpenOptions::new()
        .read(true)
        .append(true)
        .open(alias_path)?;
    let mut existing_aliases = String::new();
    aliases_file.read_to_string(&mut existing_aliases)?;

    if existing_aliases.len() >= 2 {
        let last_char = existing_aliases.chars().next_back();
        let stl_char = existing_aliases.chars().nth_back(1);
        if let (Some('\n'), Some('\n')) = (last_char, stl_char) {
        } else {
            writeln!(aliases_file)?;
        }
    }

    write!(aliases_file, "{}", aliases_string)?;
    Ok(())
}

pub fn parse_name_or_ip(
    lights: &HashMap<String, Light>,
    ip_or_alias: &String,
) -> io::Result<(String, String)> {
    let name: String = ip_or_alias.clone();
    let ip: String = lights
        .get(ip_or_alias)
        .map_or(ip_or_alias, |light: &Light| &light.ip)
        .clone();

    if ip.parse::<Ipv4Addr>().is_ok() {
        Ok((name, ip))
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Alias '{name}' not found."),
        ))
    }
}

pub async fn name_devices(
    discovered_devices: Vec<GoveePacket>,
    lights: &HashMap<String, Light>,
    auto: bool,
) -> io::Result<Vec<Light>> {
    let mut new_dev: Vec<Light> = Vec::new();
    let light_by_ip: HashMap<String, Light> =
        lights.values().map(|l| (l.ip.clone(), l.clone())).collect();

    for (i, d) in discovered_devices.into_iter().enumerate() {
        if let GoveeMessage::Handshake { cmd: _, data } = d.msg
            && !light_by_ip.contains_key(&data.ip)
        {
            if !auto {
                if i != 0 {
                    println!("----------------");
                }
                println!("New device found at IP {}", data.ip);
                println!("It should now be alternating between blue and red.");
                println!(
                    "Please locate the light and enter your preferred alias (only alphanumeric characters, '-', and '_' allowed)."
                );
                let end = Arc::new(AtomicBool::new(false));
                let ec = end.clone();
                let ip_clone = data.ip.clone();

                let animation = tokio::spawn(async move {
                    pairing_animation(&ip_clone, ec).await.ok();
                });

                let mut name = String::new();
                print!("Enter desired alias (leave empty to skip): ");
                let _ = stdout().flush();
                stdin().read_line(&mut name).expect("Invalid input.");
                let _ = match name.chars().next_back() {
                    Some('\n') | Some('\r') => name.pop(),
                    _ => None,
                };

                let filtered_name: String = name
                    .chars()
                    .map(|c| if c == ' ' { '-' } else { c })
                    .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                    .collect();

                println!("Named device: {}", filtered_name);
                if !filtered_name.is_empty() {
                    new_dev.push(Light {
                        name: filtered_name,
                        ip: data.ip.clone(),
                        mac: data.mac.clone(),
                    });
                } else {
                    println!("Skipped device with IP {}", data.ip);
                }

                end.store(true, std::sync::atomic::Ordering::Relaxed);
                animation.await?;
            } else {
                let dev_count = lights.len() + new_dev.len();
                let new_name = format!("dev{}", dev_count);
                new_dev.push(Light {
                    name: new_name,
                    ip: data.ip.clone(),
                    mac: data.mac.clone(),
                });
            }
        }
    }

    Ok(new_dev)
}
