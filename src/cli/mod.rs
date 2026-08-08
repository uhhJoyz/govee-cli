use clap::{Parser, Subcommand};
pub mod alias;

#[derive(Parser, Debug)]
#[command(version, about = "...", long_about = None, color = clap::ColorChoice::Auto)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum PowerState {
    On,
    Off,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// perform interactive setup for new devices (config found in ~/.config/goveectl/)
    FindNew {
        #[arg(long)]
        auto: bool,
    },
    /// remove an alias from the config
    RemoveAlias,
    /// list all devices on local network
    List,
    /// list all currently registered aliases on separate lines
    ListAliases,
    /// query device status by ip or alias
    Status { ip_or_alias: String },
    /// set brightness (clamped to 0-100)
    Brightness {
        ip_or_alias: String,
        brightness: u32,
    },
    /// set power to on or off
    Power {
        ip_or_alias: String,
        state: PowerState,
    },
    /// set color by passing an IP or alias then either r g b or a hex value with --hex prefixed
    Color {
        ip_or_alias: String,
        #[arg(requires_all = ["g", "b"], conflicts_with = "hex")]
        r: Option<u8>,
        #[arg(requires_all = ["r", "b"], conflicts_with = "hex")]
        g: Option<u8>,
        #[arg(requires_all = ["g", "r"], conflicts_with = "hex")]
        b: Option<u8>,
        #[arg(short, long)]
        temp: Option<u32>,
        #[arg(long, conflicts_with_all = ["r", "g", "b"], required_unless_present = "r")]
        hex: Option<String>,
    },
}
