use clap::{Parser, Subcommand, ValueEnum};
use std::{net::IpAddr, path::PathBuf};

#[derive(Debug, Clone, ValueEnum)]
pub enum Parity {
    None,
    Even,
    Odd,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum StopBits {
    #[value(name = "1")]
    One,
    #[value(name = "2")]
    Two,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum DataBits {
    #[value(name = "5")]
    Five,
    #[value(name = "6")]
    Six,
    #[value(name = "7")]
    Seven,
    #[value(name = "8")]
    Eight,
}

// Custom validation functions for Modbus specification limits
fn validate_coil_qty(s: &str) -> Result<u16, String> {
    let qty: u16 = s
        .parse()
        .map_err(|_| format!("Invalid quantity '{s}': must be a number"))?;

    if !(1..=2000).contains(&qty) {
        Err(format!(
            "Invalid quantity {qty}: Modbus specification limits coil operations to 1-2000 coils per request (FC 01/05/15)"
        ))
    } else {
        Ok(qty)
    }
}

fn validate_register_qty(s: &str) -> Result<u16, String> {
    let qty: u16 = s
        .parse()
        .map_err(|_| format!("Invalid quantity '{s}': must be a number"))?;

    if !(1..=125).contains(&qty) {
        Err(format!(
            "Invalid quantity {qty}: Modbus specification limits register operations to 1-125 registers per request (FC 03/04/06/16)"
        ))
    } else {
        Ok(qty)
    }
}

/// Flags common to every subcommand
#[derive(Debug, clap::Args)]
pub struct Common {
    /// Modbus TCP server IP address (TCP only)
    #[arg(long, value_parser = clap::value_parser!(IpAddr), conflicts_with = "device", display_order = 1)]
    pub ip: Option<IpAddr>,

    /// Modbus TCP server port (TCP only)
    #[arg(long, default_value_t = 502, display_order = 2)]
    pub port: u16,

    /// Serial device path (RTU only)
    #[arg(long, conflicts_with = "ip", display_order = 3)]
    pub device: Option<PathBuf>,

    /// Baud rate for serial communication (RTU only)
    #[arg(long, default_value_t = 9600, display_order = 4)]
    pub baud: u32,

    /// Parity for serial communication (RTU only)
    #[arg(long, value_enum, default_value = "none", display_order = 5)]
    pub parity: Parity,

    /// Stop bits for serial communication (RTU only)
    #[arg(long, value_enum, default_value = "1", display_order = 6)]
    pub stop_bits: StopBits,

    /// Data bits for serial communication (RTU only)
    #[arg(long, value_enum, default_value = "8", display_order = 7)]
    pub data_bits: DataBits,

    /// Modbus slave / unit ID
    #[arg(long, default_value_t = 0, display_order = 8)]
    pub unit: u8,

    /// Timeout for connections and operations in seconds
    #[arg(long, default_value_t = 5, display_order = 9)]
    pub timeout: u64,

    /// Verbose output
    #[arg(long, short, display_order = 10)]
    pub verbose: bool,
}

/// CLI entry point
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Modbus TCP client and server",
    after_help = "EXAMPLES:\n    mb read holding --ip 127.0.0.1 --port 502 --addr 1\n    mb read coils --ip 192.168.1.100 --addr 0 --qty 8\n    mb write holding --ip 127.0.0.1 --addr 100 --value 42\n    mb write coils --ip 127.0.0.1 --addr 0 --value 1,0,1,1\n    mb server --ip 0.0.0.0 --port 502"
)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Read coils, discrete inputs, input or holding registers
    Read {
        #[command(subcommand)]
        area: ReadArea,
    },

    /// Write coils or holding registers
    Write {
        #[command(subcommand)]
        area: WriteArea,
    },

    /// Run a Modbus server
    Server {
        /// IP address to bind to (TCP only)
        #[arg(long, value_parser = clap::value_parser!(IpAddr), conflicts_with = "device", display_order = 1)]
        ip: Option<IpAddr>,

        /// Port to listen on (TCP only)
        #[arg(long, default_value_t = 502, display_order = 2)]
        port: u16,

        /// Serial device path (RTU only)
        #[arg(long, conflicts_with = "ip", display_order = 3)]
        device: Option<PathBuf>,

        /// Baud rate for serial communication (RTU only)
        #[arg(long, default_value_t = 9600, display_order = 4)]
        baud: u32,

        /// Parity for serial communication (RTU only)
        #[arg(long, value_enum, default_value = "none", display_order = 5)]
        parity: Parity,

        /// Stop bits for serial communication (RTU only)
        #[arg(long, value_enum, default_value = "1", display_order = 6)]
        stop_bits: StopBits,

        /// Data bits for serial communication (RTU only)
        #[arg(long, value_enum, default_value = "8", display_order = 7)]
        data_bits: DataBits,

        /// Unit/Slave ID
        #[arg(long, default_value_t = 1, display_order = 8)]
        unit: u8,

        /// Number of coils (0-65535)
        #[arg(long, default_value_t = 10000, display_order = 9)]
        num_coils: u16,

        /// Number of discrete inputs (0-65535)
        #[arg(long, default_value_t = 10000, display_order = 10)]
        num_discrete: u16,

        /// Number of holding registers (0-65535)
        #[arg(long, default_value_t = 10000, display_order = 11)]
        num_holding: u16,

        /// Number of input registers (0-65535)
        #[arg(long, default_value_t = 10000, display_order = 12)]
        num_input: u16,

        /// Verbose logging
        #[arg(long, display_order = 13)]
        verbose: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum ReadArea {
    /// Read Coils (FC 1)
    Coil {
        /// Starting address
        #[arg(long = "addr", value_name = "ADDRESS")]
        start: u16,
        /// Quantity (default 1, max 2000)
        #[arg(long = "qty", default_value_t = 1, value_parser = validate_coil_qty, display_order = 6)]
        qty: u16,
        #[command(flatten)]
        common: Common,
    },
    /// Read Discrete Inputs (FC 2)
    Discrete {
        /// Starting address
        #[arg(long = "addr", value_name = "ADDRESS")]
        start: u16,
        /// Quantity (default 1, max 2000)
        #[arg(long = "qty", default_value_t = 1, value_parser = validate_coil_qty, display_order = 6)]
        qty: u16,
        #[command(flatten)]
        common: Common,
    },
    /// Read Holding Registers (FC 3)
    Holding {
        /// Starting address
        #[arg(long = "addr", value_name = "ADDRESS")]
        start: u16,
        /// Quantity (default 1, max 125)
        #[arg(long = "qty", default_value_t = 1, value_parser = validate_register_qty, display_order = 6)]
        qty: u16,
        #[command(flatten)]
        common: Common,
    },
    /// Read Input Registers (FC 4)
    Input {
        /// Starting address
        #[arg(long = "addr", value_name = "ADDRESS")]
        start: u16,
        /// Quantity (default 1, max 125)
        #[arg(long = "qty", default_value_t = 1, value_parser = validate_register_qty, display_order = 6)]
        qty: u16,
        #[command(flatten)]
        common: Common,
    },
}

#[derive(Subcommand, Debug)]
pub enum WriteArea {
    /// Write Single/Multiple Coils (FC 5/15)
    Coil {
        /// Starting address
        #[arg(long = "addr", value_name = "ADDRESS")]
        start: u16,
        /// Value(s) to write (0=OFF, 1=ON; comma-separated for multiple)
        #[arg(
            long = "value",
            value_delimiter = ',',
            num_args = 1..,
            required = true,
            value_parser = clap::value_parser!(u16)
        )]
        values: Vec<u16>,
        #[command(flatten)]
        common: Common,
    },
    /// Write Single/Multiple Holding Registers (FC 6/16)
    Holding {
        /// Starting address
        #[arg(long = "addr", value_name = "ADDRESS")]
        start: u16,
        /// Value(s) to write (comma-separated for multiple)
        #[arg(
            long = "value",
            value_delimiter = ',',
            num_args = 1..,
            required = true,
            value_parser = clap::value_parser!(u16)
        )]
        values: Vec<u16>,
        #[command(flatten)]
        common: Common,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_coil_qty_valid() {
        assert_eq!(validate_coil_qty("1"), Ok(1));
        assert_eq!(validate_coil_qty("1000"), Ok(1000));
        assert_eq!(validate_coil_qty("2000"), Ok(2000));
    }

    #[test]
    fn test_validate_coil_qty_invalid() {
        assert!(validate_coil_qty("0").is_err());
        assert!(validate_coil_qty("2001").is_err());
        assert!(validate_coil_qty("abc").is_err());
        assert!(validate_coil_qty("").is_err());
    }

    #[test]
    fn test_validate_coil_qty_error_messages() {
        let result = validate_coil_qty("0");
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("1-2000"));
        assert!(error_msg.contains("FC 01/05/15"));

        let result = validate_coil_qty("abc");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be a number"));
    }

    #[test]
    fn test_validate_register_qty_valid() {
        assert_eq!(validate_register_qty("1"), Ok(1));
        assert_eq!(validate_register_qty("100"), Ok(100));
        assert_eq!(validate_register_qty("125"), Ok(125));
    }

    #[test]
    fn test_validate_register_qty_invalid() {
        assert!(validate_register_qty("0").is_err());
        assert!(validate_register_qty("126").is_err());
        assert!(validate_register_qty("xyz").is_err());
        assert!(validate_register_qty("").is_err());
    }

    #[test]
    fn test_validate_register_qty_error_messages() {
        let result = validate_register_qty("0");
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("1-125"));
        assert!(error_msg.contains("FC 03/04/06/16"));

        let result = validate_register_qty("xyz");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be a number"));
    }
}
