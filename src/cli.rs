use clap::{Parser, Subcommand};
use std::{net::IpAddr, path::PathBuf};

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
    /// Modbus TCP server IP address (for TCP client)
    #[arg(long, value_parser = clap::value_parser!(IpAddr), conflicts_with = "device")]
    pub ip: Option<IpAddr>,

    /// Serial device path (for RTU client)
    #[arg(long, conflicts_with = "ip")]
    pub device: Option<PathBuf>,

    /// Modbus TCP server port (TCP only)
    #[arg(long, default_value_t = 502)]
    pub port: u16,

    /// Baud rate for serial communication (RTU only)
    #[arg(long, default_value_t = 9600)]
    pub baud: u32,

    /// Modbus slave / unit ID
    #[arg(long, default_value_t = 0)]
    pub unit: u8,

    /// Verbose output
    #[arg(long, short)]
    pub verbose: bool,

    /// Timeout for connections and operations in seconds
    #[arg(long, default_value_t = 5)]
    pub timeout: u64,
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
        /// IP address to bind to (for TCP server)
        #[arg(long, value_parser = clap::value_parser!(IpAddr), conflicts_with = "device")]
        ip: Option<IpAddr>,

        /// Serial device path (for RTU server)
        #[arg(long, conflicts_with = "ip")]
        device: Option<PathBuf>,

        /// Port to listen on (TCP only)
        #[arg(long, default_value_t = 502)]
        port: u16,

        /// Baud rate for serial communication (RTU only)
        #[arg(long, default_value_t = 9600)]
        baud: u32,

        /// Unit/Slave ID
        #[arg(long, default_value_t = 1)]
        unit: u8,

        /// Number of coils (0-65535)
        #[arg(long, default_value_t = 10000)]
        num_coils: u16,

        /// Number of discrete inputs (0-65535)
        #[arg(long, default_value_t = 10000)]
        num_discrete: u16,

        /// Number of holding registers (0-65535)
        #[arg(long, default_value_t = 10000)]
        num_holding: u16,

        /// Number of input registers (0-65535)
        #[arg(long, default_value_t = 10000)]
        num_input: u16,

        /// Verbose logging
        #[arg(long)]
        verbose: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum ReadArea {
    /// Read Coils (FC 1)
    Coil {
        /// Starting address
        #[arg(long = "addr")]
        start: u16,
        /// Quantity (default 1, max 2000)
        #[arg(long = "qty", default_value_t = 1, value_parser = validate_coil_qty)]
        qty: u16,
        #[command(flatten)]
        common: Common,
    },
    /// Read Discrete Inputs (FC 2)
    Discrete {
        /// Starting address
        #[arg(long = "addr")]
        start: u16,
        /// Quantity (default 1, max 2000)
        #[arg(long = "qty", default_value_t = 1, value_parser = validate_coil_qty)]
        qty: u16,
        #[command(flatten)]
        common: Common,
    },
    /// Read Holding Registers (FC 3)
    Holding {
        /// Starting address
        #[arg(long = "addr")]
        start: u16,
        /// Quantity (default 1, max 125)
        #[arg(long = "qty", default_value_t = 1, value_parser = validate_register_qty)]
        qty: u16,
        #[command(flatten)]
        common: Common,
    },
    /// Read Input Registers (FC 4)
    Input {
        /// Starting address
        #[arg(long = "addr")]
        start: u16,
        /// Quantity (default 1, max 125)
        #[arg(long = "qty", default_value_t = 1, value_parser = validate_register_qty)]
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
        #[arg(long = "addr")]
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
        #[arg(long = "addr")]
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