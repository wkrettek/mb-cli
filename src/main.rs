use clap::{Parser, Subcommand, arg};
use std::{
    future,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};
use tokio_modbus::client::{Reader, Writer};
use tokio_modbus::prelude::*;
use tokio_modbus::server::{Service, tcp::Server};

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
#[command(
    author,
    version,
    about = "Modbus TCP client and server",
    after_help = "EXAMPLES:\n    mb read holding --ip 127.0.0.1 --port 502 --addr 1\n    mb read coils --ip 192.168.1.100 --addr 0 --qty 8\n    mb write holding --ip 127.0.0.1 --addr 100 --value 42\n    mb write coils --ip 127.0.0.1 --addr 0 --value 1,0,1,1\n    mb server --bind 0.0.0.0 --port 502"
)]
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

    /// Run a Modbus server
    Server {
        /// IP address to bind to
        #[arg(long, default_value = "0.0.0.0", value_parser = clap::value_parser!(IpAddr))]
        bind: IpAddr,

        /// Port to listen on
        #[arg(long, default_value_t = 502)]
        port: u16,

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
        /// Value(s) to write (0=OFF, 1=ON; comma-separated for multiple)
        #[arg(
            long = "value",
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
        /// Value(s) to write (comma-separated for multiple)
        #[arg(
            long = "value",
            value_delimiter = ',',
            num_args = 1..,
            value_parser = clap::value_parser!(u16)
        )]
        values: Vec<u16>,
        #[command(flatten)]
        common: Common,
    },
}

#[derive(Clone)]
struct ModbusService;

impl Service for ModbusService {
    type Request = Request<'static>;
    type Response = Response;
    type Exception = ExceptionCode;
    type Future = future::Ready<Result<Self::Response, Self::Exception>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let response = match req {
            Request::ReadCoils(addr, qty) => {
                // Return dummy coils - all OFF for now
                let coils = vec![false; qty as usize];
                Response::ReadCoils(coils)
            }
            Request::ReadDiscreteInputs(addr, qty) => {
                // Return dummy discrete inputs - all OFF for now
                let inputs = vec![false; qty as usize];
                Response::ReadDiscreteInputs(inputs)
            }
            Request::ReadHoldingRegisters(addr, qty) => {
                // Return registers where each address equals its value
                let registers: Vec<u16> = (addr..addr + qty).collect();
                Response::ReadHoldingRegisters(registers)
            }
            Request::ReadInputRegisters(addr, qty) => {
                // Return registers where each address equals its value
                let registers: Vec<u16> = (addr..addr + qty).collect();
                Response::ReadInputRegisters(registers)
            }
            Request::WriteSingleCoil(addr, value) => {
                println!("Write coil {addr}: {value}");
                Response::WriteSingleCoil(addr, value)
            }
            Request::WriteSingleRegister(addr, value) => {
                println!("Write register {addr}: {value}");
                Response::WriteSingleRegister(addr, value)
            }
            Request::WriteMultipleCoils(addr, values) => {
                println!("Write {} coils starting at {addr}", values.len());
                Response::WriteMultipleCoils(addr, values.len() as u16)
            }
            Request::WriteMultipleRegisters(addr, values) => {
                println!("Write {} registers starting at {addr}", values.len());
                Response::WriteMultipleRegisters(addr, values.len() as u16)
            }
            _ => {
                return future::ready(Err(ExceptionCode::IllegalFunction));
            }
        };
        future::ready(Ok(response))
    }
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

        Command::Write { area } => match area {
            WriteArea::Coils {
                start,
                values,
                common,
            } => {
                let mut client = connect_to_modbus(common.ip, common.port, common.unit).await?;

                // Convert u16 values to bool values (0 = false, non-zero = true)
                let bool_values: Vec<bool> = values.iter().map(|&v| v != 0).collect();

                if bool_values.len() == 1 {
                    // Single coil write (FC 5)
                    println!(
                        "Writing single coil at address {} with value {} (Unit ID: {})",
                        start,
                        if bool_values[0] { "ON" } else { "OFF" },
                        common.unit
                    );
                    match client.write_single_coil(start, bool_values[0]).await {
                        Ok(response) => match response {
                            Ok(_) => {
                                println!("Successfully wrote coil at address {start}");
                            }
                            Err(exception) => {
                                eprintln!("Modbus exception response: {exception:?}");
                                return Err(anyhow::anyhow!("Modbus exception: {:?}", exception));
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to write coil: {e}");
                            return Err(e.into());
                        }
                    }
                } else {
                    // Multiple coils write (FC 15)
                    println!(
                        "Writing {} coils starting at address {} (Unit ID: {})",
                        bool_values.len(),
                        start,
                        common.unit
                    );
                    match client.write_multiple_coils(start, &bool_values).await {
                        Ok(response) => match response {
                            Ok(_) => {
                                println!(
                                    "Successfully wrote {} coils starting at address {}",
                                    bool_values.len(),
                                    start
                                );
                                for (i, value) in bool_values.iter().enumerate() {
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
                            eprintln!("Failed to write coils: {e}");
                            return Err(e.into());
                        }
                    }
                }
            }
            WriteArea::Holding {
                start,
                values,
                common,
            } => {
                let mut client = connect_to_modbus(common.ip, common.port, common.unit).await?;

                if values.len() == 1 {
                    // Single register write (FC 6)
                    println!(
                        "Writing single holding register at address {} with value {} (0x{:04X}) (Unit ID: {})",
                        start, values[0], values[0], common.unit
                    );
                    match client.write_single_register(start, values[0]).await {
                        Ok(response) => match response {
                            Ok(_) => {
                                println!(
                                    "Successfully wrote holding register at address {} with value {} (0x{:04X})",
                                    start, values[0], values[0]
                                );
                            }
                            Err(exception) => {
                                eprintln!("Modbus exception response: {exception:?}");
                                return Err(anyhow::anyhow!("Modbus exception: {:?}", exception));
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to write register: {e}");
                            return Err(e.into());
                        }
                    }
                } else {
                    // Multiple registers write (FC 16)
                    println!(
                        "Writing {} holding registers starting at address {} (Unit ID: {})",
                        values.len(),
                        start,
                        common.unit
                    );
                    match client.write_multiple_registers(start, &values).await {
                        Ok(response) => match response {
                            Ok(_) => {
                                println!(
                                    "Successfully wrote {} holding registers starting at address {}",
                                    values.len(),
                                    start
                                );
                                for (i, value) in values.iter().enumerate() {
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
                            eprintln!("Failed to write registers: {e}");
                            return Err(e.into());
                        }
                    }
                }
            }
        },

        Command::Server {
            bind,
            port,
            unit: _unit,
            num_coils,
            num_discrete,
            num_holding,
            num_input,
            verbose: _verbose,
        } => {
            println!("Starting Modbus TCP server on {bind}:{port}");
            println!("Configuration:");
            println!(
                "  Coils: {} (addresses 0-{})",
                num_coils,
                num_coils.saturating_sub(1)
            );
            println!(
                "  Discrete Inputs: {} (addresses 0-{})",
                num_discrete,
                num_discrete.saturating_sub(1)
            );
            println!(
                "  Holding Registers: {} (addresses 0-{})",
                num_holding,
                num_holding.saturating_sub(1)
            );
            println!(
                "  Input Registers: {} (addresses 0-{})",
                num_input,
                num_input.saturating_sub(1)
            );
            println!("  Initialization: Each address value equals its address");
            println!();

            let socket_addr = SocketAddr::new(bind, port);
            let listener = tokio::net::TcpListener::bind(socket_addr).await?;
            println!("Modbus TCP server listening on {bind}:{port}");
            println!("Press Ctrl+C to stop the server");

            let server = Server::new(listener);
            let service = ModbusService;

            let on_connected = move |stream, socket_addr| {
                let service = service.clone();
                async move {
                    println!("Client connected: {socket_addr}");
                    tokio_modbus::server::tcp::accept_tcp_connection(stream, socket_addr, |_| {
                        Ok(Some(service.clone()))
                    })
                }
            };

            let on_process_error = |err| {
                eprintln!("Server error: {err}");
            };

            let ctrl_c = Box::pin(async {
                tokio::signal::ctrl_c().await.ok();
            });

            match server
                .serve_until(&on_connected, on_process_error, ctrl_c)
                .await?
            {
                tokio_modbus::server::Terminated::Finished => {
                    println!("\nServer finished");
                }
                tokio_modbus::server::Terminated::Aborted => {
                    println!("\nServer stopped");
                }
            }
        }
    }

    Ok(())
}
