use clap::{Parser, Subcommand, arg};
use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};
use tokio_modbus::client::Reader;
use tokio_modbus::prelude::*;

/// Flags common to every subcommand
#[derive(Debug, clap::Args)]
struct Common {
    /// Modbus TCP server IP address
    #[arg(long, value_parser = clap::value_parser!(IpAddr))]
    ip: IpAddr,

    /// Modbus TCP server port
    #[arg(long, default_value_t = 502)]
    port: u16,

    /// Modbus slave / unit ID
    #[arg(long, default_value_t = 0)]
    unit: u8,

    /// Optional CSV scaling/metadata file
    #[arg(long)]
    format: Option<PathBuf>,
}

/// CLI entry point
#[derive(Parser, Debug)]
#[command(author, version, about = "Rust Modbus TCP client (single-shot)")]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
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
}

#[derive(Subcommand, Debug)]
enum ReadArea {
    /// Read Coils (FC 1)
    Coils {
        /// Starting address
        #[arg(long = "addr")]
        start: u16,
        /// Quantity (default 1)
        #[arg(long = "qty", default_value_t = 1)]
        qty: u16,
        #[command(flatten)]
        common: Common,
    },
    /// Read Discrete Inputs (FC 2)
    Discrete {
        /// Starting address
        #[arg(long = "addr")]
        start: u16,
        /// Quantity (default 1)
        #[arg(long = "qty", default_value_t = 1)]
        qty: u16,
        #[command(flatten)]
        common: Common,
    },
    /// Read Holding Registers (FC 3)
    Holding {
        /// Starting address
        #[arg(long = "addr")]
        start: u16,
        /// Quantity (default 1)
        #[arg(long = "qty", default_value_t = 1)]
        qty: u16,
        #[command(flatten)]
        common: Common,
    },
    /// Read Input Registers (FC 4)
    Input {
        /// Starting address
        #[arg(long = "addr")]
        start: u16,
        /// Quantity (default 1)
        #[arg(long = "qty", default_value_t = 1)]
        qty: u16,
        #[command(flatten)]
        common: Common,
    },
}

#[derive(Subcommand, Debug)]
enum WriteArea {
    /// Write Single/Multiple Coils (FC 5/15)
    Coils {
        /// Starting address
        #[arg(long = "addr")]
        start: u16,
        /// Values to write (repeat flag or comma-sep)
        #[arg(
            long = "values",
            value_delimiter = ',',
            num_args = 1..,
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
        /// Values to write (repeat flag or comma-sep)
        #[arg(
            long = "values",
            value_delimiter = ',',
            num_args = 1..,
            value_parser = clap::value_parser!(u16)
        )]
        values: Vec<u16>,
        #[command(flatten)]
        common: Common,
    },
}

async fn connect_to_modbus(ip: IpAddr, port: u16, unit_id: u8) -> anyhow::Result<client::Context> {
    let socket_addr = SocketAddr::new(ip, port);
    println!("Connecting to Modbus server at {ip}:{port} (Unit ID: {unit_id})...");

    match client::tcp::connect(socket_addr).await {
        Ok(mut ctx) => {
            // Set the slave/unit ID
            ctx.set_slave(Slave(unit_id));
            println!("Successfully connected to Modbus server at {ip}:{port}");
            Ok(ctx)
        }
        Err(e) => {
            println!("Failed to connect to {ip}:{port} - Error: {e}");
            Err(e.into())
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Command::Read { area } => match area {
            ReadArea::Coils { start, qty, common } => {
                println!(
                    "Reading coil at address {} (Unit ID: {})",
                    start, common.unit
                );
                let mut client = connect_to_modbus(common.ip, common.port, common.unit).await?;

                match client.read_coils(start, qty).await {
                    Ok(response) => match response {
                        Ok(coils) => {
                            println!("Successfully read {} coil(s):", coils.len());
                            for (i, value) in coils.iter().enumerate() {
                                let addr = start + i as u16;
                                println!(
                                    "  Address {}: {}",
                                    addr,
                                    if *value { "ON" } else { "OFF" }
                                );
                            }
                        }
                        Err(exception) => {
                            eprintln!("Modbus exception response: {exception:?}");
                            return Err(anyhow::anyhow!("Modbus exception: {:?}", exception));
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to read coils: {e}");
                        return Err(e.into());
                    }
                }
            }
            ReadArea::Discrete { start, qty, common } => {
                println!(
                    "Reading discrete input at address {} (Unit ID: {})",
                    start, common.unit
                );
                let mut client = connect_to_modbus(common.ip, common.port, common.unit).await?;

                match client.read_discrete_inputs(start, qty).await {
                    Ok(response) => match response {
                        Ok(inputs) => {
                            println!("Successfully read {} discrete input(s):", inputs.len());
                            for (i, value) in inputs.iter().enumerate() {
                                let addr = start + i as u16;
                                println!(
                                    "  Address {}: {}",
                                    addr,
                                    if *value { "ON" } else { "OFF" }
                                );
                            }
                        }
                        Err(exception) => {
                            eprintln!("Modbus exception response: {exception:?}");
                            return Err(anyhow::anyhow!("Modbus exception: {:?}", exception));
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to read discrete inputs: {e}");
                        return Err(e.into());
                    }
                }
            }
            ReadArea::Holding { start, qty, common } => {
                println!(
                    "Reading holding register at address {} (Unit ID: {})",
                    start, common.unit
                );
                let mut client = connect_to_modbus(common.ip, common.port, common.unit).await?;

                match client.read_holding_registers(start, qty).await {
                    Ok(response) => match response {
                        Ok(registers) => {
                            println!("Successfully read {} holding register(s):", registers.len());
                            for (i, value) in registers.iter().enumerate() {
                                let addr = start + i as u16;
                                println!("  Address {addr}: {value} (0x{value:04X})");
                            }
                        }
                        Err(exception) => {
                            eprintln!("Modbus exception response: {exception:?}");
                            return Err(anyhow::anyhow!("Modbus exception: {:?}", exception));
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to read holding registers: {e}");
                        return Err(e.into());
                    }
                }
            }
            ReadArea::Input { start, qty, common } => {
                println!(
                    "Reading input register at address {} (Unit ID: {})",
                    start, common.unit
                );
                let mut client = connect_to_modbus(common.ip, common.port, common.unit).await?;

                match client.read_input_registers(start, qty).await {
                    Ok(response) => match response {
                        Ok(registers) => {
                            println!("Successfully read {} input register(s):", registers.len());
                            for (i, value) in registers.iter().enumerate() {
                                let addr = start + i as u16;
                                println!("  Address {addr}: {value} (0x{value:04X})");
                            }
                        }
                        Err(exception) => {
                            eprintln!("Modbus exception response: {exception:?}");
                            return Err(anyhow::anyhow!("Modbus exception: {:?}", exception));
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to read input registers: {e}");
                        return Err(e.into());
                    }
                }
            }
        },

        Command::Write { area } => {
            match area {
                WriteArea::Coils {
                    start,
                    values,
                    common,
                } => {
                    println!(
                        "Writing coil at address {} with value {:?} (Unit ID: {})",
                        start, values, common.unit
                    );
                    let _client = connect_to_modbus(common.ip, common.port, common.unit).await?;

                    // TODO: Perform actual write
                    println!("Would write coil value {values:?} at address {start}");
                }
                WriteArea::Holding {
                    start,
                    values,
                    common,
                } => {
                    println!(
                        "Writing holding register at address {} with value {:?} (Unit ID: {})",
                        start, values, common.unit
                    );
                    let _client = connect_to_modbus(common.ip, common.port, common.unit).await?;

                    // TODO: Perform actual write
                    println!("Would write register value {values:?} at address {start}");
                }
            }
        }
    }

    Ok(())
}
