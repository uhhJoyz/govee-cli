# Welcome

This is a CLI, written in Rust, which allows you to use every defined Govee LAN API function from your terminal. This CLI also supports aliasing of lights through an interactive setup process or a TOML file declaration.

# Supported Commands

This CLI supports the following commands.

Commands:
  **find-new**      perform interactive setup for new devices (config found in ~/.config/govee-cli/)
  **remove-alias**  remove an alias from the config
  **list**          list all devices on local network
  **list-aliases**  list all currently registered aliases on separate lines
  **status**        query device status by ip or alias
  **brightness**    set brightness (clamped to 0-100)
  **power**         set power to on or off
  **color**         set color by passing an IP or alias then either r g b or a hex value with --hex prefixed
  **help**          Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
