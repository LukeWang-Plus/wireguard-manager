//! Tests for CLI argument parsing.

use super::*;

// ── Init command ────────────────────────────────────────────────

#[test]
fn test_parse_init_basic() {
    let cli = Cli::try_parse_from([
        "wireguard",
        "init",
        "--subnet",
        "10.0.0.0/24",
        "--server-range",
        "[1,10]",
    ])
    .unwrap();
    if let Some(Commands::Init {
        subnet,
        server_range,
        force,
    }) = cli.command
    {
        assert_eq!(subnet, "10.0.0.0/24");
        assert_eq!(server_range, "[1,10]");
        assert!(!force);
    } else {
        panic!("Expected Init variant");
    }
}

#[test]
fn test_parse_init_with_force() {
    let cli = Cli::try_parse_from([
        "wireguard",
        "init",
        "--subnet",
        "10.0.0.0/24",
        "--server-range",
        "[1,10]",
        "--force",
    ])
    .unwrap();
    if let Some(Commands::Init { force, .. }) = cli.command {
        assert!(force);
    } else {
        panic!("Expected Init variant");
    }
}

#[test]
fn test_parse_init_missing_subnet() {
    let result = Cli::try_parse_from(["wireguard", "init", "--server-range", "[1,10]"]);
    assert!(result.is_err());
}

#[test]
fn test_parse_init_missing_server_range() {
    let result = Cli::try_parse_from(["wireguard", "init", "--subnet", "10.0.0.0/24"]);
    assert!(result.is_err());
}

#[test]
fn test_parse_init_requires_both_args() {
    let result = Cli::try_parse_from(["wireguard", "init"]);
    assert!(result.is_err());
}

// ── Config command ──────────────────────────────────────────────

#[test]
fn test_parse_config_show() {
    let cli = Cli::try_parse_from(["wireguard", "config", "show"]).unwrap();
    if let Some(Commands::Config { action }) = cli.command {
        assert!(matches!(action, ConfigAction::Show));
    } else {
        panic!("Expected Config variant");
    }
}

#[test]
fn test_parse_config_set_subnet() {
    let cli =
        Cli::try_parse_from(["wireguard", "config", "set", "--subnet", "172.16.0.0/16"]).unwrap();
    if let Some(Commands::Config {
        action: ConfigAction::Set {
            subnet,
            server_range,
        },
    }) = cli.command
    {
        assert_eq!(subnet.unwrap(), "172.16.0.0/16");
        assert!(server_range.is_none());
    } else {
        panic!("Expected Config Set variant");
    }
}

#[test]
fn test_parse_config_set_server_range() {
    let cli =
        Cli::try_parse_from(["wireguard", "config", "set", "--server-range", "[1,20]"]).unwrap();
    if let Some(Commands::Config {
        action: ConfigAction::Set {
            subnet,
            server_range,
        },
    }) = cli.command
    {
        assert!(subnet.is_none());
        assert_eq!(server_range.unwrap(), "[1,20]");
    } else {
        panic!("Expected Config Set variant");
    }
}

#[test]
fn test_parse_config_set_both() {
    let cli = Cli::try_parse_from([
        "wireguard",
        "config",
        "set",
        "--subnet",
        "192.168.0.0/24",
        "--server-range",
        "[1,5]",
    ])
    .unwrap();
    if let Some(Commands::Config {
        action: ConfigAction::Set {
            subnet,
            server_range,
        },
    }) = cli.command
    {
        assert_eq!(subnet.unwrap(), "192.168.0.0/24");
        assert_eq!(server_range.unwrap(), "[1,5]");
    } else {
        panic!("Expected Config Set variant");
    }
}

#[test]
fn test_parse_config_set_no_options() {
    let cli = Cli::try_parse_from(["wireguard", "config", "set"]).unwrap();
    if let Some(Commands::Config {
        action: ConfigAction::Set {
            subnet,
            server_range,
        },
    }) = cli.command
    {
        assert!(subnet.is_none());
        assert!(server_range.is_none());
    } else {
        panic!("Expected Config Set variant");
    }
}

// ── Add command ─────────────────────────────────────────────────

#[test]
fn test_parse_add_server_basic() {
    let cli = Cli::try_parse_from([
        "wireguard",
        "add",
        "server",
        "gw1",
        "--public-ip",
        "203.0.113.1",
    ])
    .unwrap();
    if let Some(Commands::Add {
        device_type:
            AddDevice::Server {
                name,
                public_ip,
                port,
            },
    }) = cli.command
    {
        assert_eq!(name, "gw1");
        assert_eq!(public_ip, "203.0.113.1");
        assert_eq!(port, 51820);
    } else {
        panic!("Expected Add Server variant");
    }
}

#[test]
fn test_parse_add_server_default_port() {
    let cli = Cli::try_parse_from([
        "wireguard",
        "add",
        "server",
        "srv",
        "--public-ip",
        "1.2.3.4",
    ])
    .unwrap();
    if let Some(Commands::Add {
        device_type: AddDevice::Server { port, .. },
    }) = cli.command
    {
        assert_eq!(port, 51820);
    } else {
        panic!("Expected Add Server variant");
    }
}

#[test]
fn test_parse_add_server_custom_port() {
    let cli = Cli::try_parse_from([
        "wireguard",
        "add",
        "server",
        "srv",
        "--public-ip",
        "1.2.3.4",
        "--port",
        "12345",
    ])
    .unwrap();
    if let Some(Commands::Add {
        device_type: AddDevice::Server { port, .. },
    }) = cli.command
    {
        assert_eq!(port, 12345);
    } else {
        panic!("Expected Add Server variant");
    }
}

#[test]
fn test_parse_add_server_missing_public_ip() {
    let result = Cli::try_parse_from(["wireguard", "add", "server", "srv"]);
    assert!(result.is_err());
}

#[test]
fn test_parse_add_client() {
    let cli = Cli::try_parse_from(["wireguard", "add", "client", "laptop"]).unwrap();
    if let Some(Commands::Add {
        device_type: AddDevice::Client { name },
    }) = cli.command
    {
        assert_eq!(name, "laptop");
    } else {
        panic!("Expected Add Client variant");
    }
}

// ── Edit command ────────────────────────────────────────────────

#[test]
fn test_parse_edit_server_public_ip() {
    let cli = Cli::try_parse_from([
        "wireguard",
        "edit",
        "server",
        "gw1",
        "--public-ip",
        "198.51.100.1",
    ])
    .unwrap();
    if let Some(Commands::Edit {
        device_type:
            EditDevice::Server {
                name,
                public_ip,
                port,
            },
    }) = cli.command
    {
        assert_eq!(name, "gw1");
        assert_eq!(public_ip.unwrap(), "198.51.100.1");
        assert!(port.is_none());
    } else {
        panic!("Expected Edit Server variant");
    }
}

#[test]
fn test_parse_edit_server_port() {
    let cli =
        Cli::try_parse_from(["wireguard", "edit", "server", "gw1", "--port", "9999"]).unwrap();
    if let Some(Commands::Edit {
        device_type:
            EditDevice::Server {
                name,
                public_ip,
                port,
            },
    }) = cli.command
    {
        assert_eq!(name, "gw1");
        assert!(public_ip.is_none());
        assert_eq!(port.unwrap(), 9999);
    } else {
        panic!("Expected Edit Server variant");
    }
}

#[test]
fn test_parse_edit_server_no_changes() {
    let cli = Cli::try_parse_from(["wireguard", "edit", "server", "gw1"]).unwrap();
    if let Some(Commands::Edit {
        device_type:
            EditDevice::Server {
                name,
                public_ip,
                port,
            },
    }) = cli.command
    {
        assert_eq!(name, "gw1");
        assert!(public_ip.is_none());
        assert!(port.is_none());
    } else {
        panic!("Expected Edit Server variant");
    }
}

#[test]
fn test_parse_edit_client_rename() {
    let cli = Cli::try_parse_from([
        "wireguard",
        "edit",
        "client",
        "phone",
        "--new-name",
        "phone-old",
    ])
    .unwrap();
    if let Some(Commands::Edit {
        device_type: EditDevice::Client { name, new_name },
    }) = cli.command
    {
        assert_eq!(name, "phone");
        assert_eq!(new_name.unwrap(), "phone-old");
    } else {
        panic!("Expected Edit Client variant");
    }
}

#[test]
fn test_parse_edit_client_no_changes() {
    let cli = Cli::try_parse_from(["wireguard", "edit", "client", "phone"]).unwrap();
    if let Some(Commands::Edit {
        device_type: EditDevice::Client { name, new_name },
    }) = cli.command
    {
        assert_eq!(name, "phone");
        assert!(new_name.is_none());
    } else {
        panic!("Expected Edit Client variant");
    }
}

// ── Del command ─────────────────────────────────────────────────

#[test]
fn test_parse_del_server() {
    let cli = Cli::try_parse_from(["wireguard", "del", "server", "gw1"]).unwrap();
    if let Some(Commands::Del {
        device_type: DelDevice::Server { name },
    }) = cli.command
    {
        assert_eq!(name, "gw1");
    } else {
        panic!("Expected Del Server variant");
    }
}

#[test]
fn test_parse_del_client() {
    let cli = Cli::try_parse_from(["wireguard", "del", "client", "laptop"]).unwrap();
    if let Some(Commands::Del {
        device_type: DelDevice::Client { name },
    }) = cli.command
    {
        assert_eq!(name, "laptop");
    } else {
        panic!("Expected Del Client variant");
    }
}

// ── Gen command ─────────────────────────────────────────────────

#[test]
fn test_parse_gen_single_device() {
    let cli = Cli::try_parse_from(["wireguard", "gen", "laptop"]).unwrap();
    if let Some(Commands::Gen { name, output_dir }) = cli.command {
        assert_eq!(name, "laptop");
        assert!(output_dir.is_none());
    } else {
        panic!("Expected Gen variant");
    }
}

#[test]
fn test_parse_gen_with_output_dir() {
    let cli =
        Cli::try_parse_from(["wireguard", "gen", "laptop", "--output-dir", "/tmp/wg"]).unwrap();
    if let Some(Commands::Gen { name, output_dir }) = cli.command {
        assert_eq!(name, "laptop");
        assert_eq!(output_dir.unwrap(), "/tmp/wg");
    } else {
        panic!("Expected Gen variant");
    }
}

#[test]
fn test_parse_gen_all() {
    let cli = Cli::try_parse_from(["wireguard", "gen", "all"]).unwrap();
    if let Some(Commands::Gen { name, .. }) = cli.command {
        assert_eq!(name, "all");
    } else {
        panic!("Expected Gen variant");
    }
}

// ── List / Show ─────────────────────────────────────────────────

#[test]
fn test_parse_list() {
    let cli = Cli::try_parse_from(["wireguard", "list"]).unwrap();
    assert!(matches!(cli.command, Some(Commands::List)));
}

#[test]
fn test_parse_show() {
    let cli = Cli::try_parse_from(["wireguard", "show", "gw1"]).unwrap();
    if let Some(Commands::Show { name }) = cli.command {
        assert_eq!(name, "gw1");
    } else {
        panic!("Expected Show variant");
    }
}

// ── Edge cases ──────────────────────────────────────────────────

#[test]
fn test_parse_no_command() {
    let cli = Cli::try_parse_from(["wireguard"]).unwrap();
    assert!(cli.command.is_none());
}

#[test]
fn test_parse_unknown_command() {
    let result = Cli::try_parse_from(["wireguard", "foobar"]);
    assert!(result.is_err());
}

#[test]
fn test_parse_help_flag() {
    let result = Cli::try_parse_from(["wireguard", "--help"]);
    assert!(result.is_err());
}
