use clap::{Parser, Subcommand};
use std::{
    future,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio_modbus::client::{Reader, Writer};
use tokio_modbus::prelude::*;
use tokio_modbus::server::{Service, rtu, tcp::Server};

/// Flags common to every subcommand
#[derive(Debug, clap::Args)]
struct Common {
    /// Modbus TCP server IP address (for TCP client)
    #[arg(long, value_parser = clap::value_parser!(IpAddr), conflicts_with = "device")]
    ip: Option<IpAddr>,

    /// Serial device path (for RTU client)
    #[arg(long, conflicts_with = "ip")]
    device: Option<PathBuf>,

    /// Modbus TCP server port (TCP only)
    #[arg(long, default_value_t = 502)]
    port: u16,

    /// Baud rate for serial communication (RTU only)
    #[arg(long, default_value_t = 9600)]
    baud: u32,

    /// Modbus slave / unit ID
    #[arg(long, default_value_t = 0)]
    unit: u8,


    /// Verbose output
    #[arg(long, short)]
    verbose: bool,
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
enum ReadArea {
    /// Read Coils (FC 1)
    Coil {
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

#[derive(Debug)]
struct ModbusData {
    coils: Vec<bool>,
    discrete_inputs: Vec<bool>,
    holding_registers: Vec<u16>,
    input_registers: Vec<u16>,
}

impl ModbusData {
    fn new(num_coils: u16, num_discrete: u16, num_holding: u16, num_input: u16) -> Self {
        Self {
            coils: vec![false; num_coils as usize],
            discrete_inputs: vec![false; num_discrete as usize],
            holding_registers: (0..num_holding).collect(),
            input_registers: (0..num_input).collect(),
        }
    }
}

#[derive(Clone)]
struct ModbusService {
    data: Arc<Mutex<ModbusData>>,
}

impl ModbusService {
    fn new(data: Arc<Mutex<ModbusData>>) -> Self {
        Self { data }
    }
}

impl Service for ModbusService {
    type Request = Request<'static>;
    type Response = Response;
    type Exception = ExceptionCode;
    type Future = future::Ready<Result<Self::Response, Self::Exception>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let mut data = match self.data.lock() {
            Ok(data) => data,
            Err(_) => return future::ready(Err(ExceptionCode::ServerDeviceFailure)),
        };

        let response = match req {
            Request::ReadCoils(addr, qty) => {
                // Note: We don't have access to client IP in the service layer
                println!("Read {qty} coil(s) starting at {addr}");
                let start = addr as usize;
                let end = start + qty as usize;
                if end <= data.coils.len() {
                    let coils = data.coils[start..end].to_vec();
                    Response::ReadCoils(coils)
                } else {
                    return future::ready(Err(ExceptionCode::IllegalDataAddress));
                }
            }
            Request::ReadDiscreteInputs(addr, qty) => {
                println!("Read {qty} discrete input(s) starting at {addr}");
                let start = addr as usize;
                let end = start + qty as usize;
                if end <= data.discrete_inputs.len() {
                    let inputs = data.discrete_inputs[start..end].to_vec();
                    Response::ReadDiscreteInputs(inputs)
                } else {
                    return future::ready(Err(ExceptionCode::IllegalDataAddress));
                }
            }
            Request::ReadHoldingRegisters(addr, qty) => {
                println!("Read {qty} holding register(s) starting at {addr}");
                let start = addr as usize;
                let end = start + qty as usize;
                if end <= data.holding_registers.len() {
                    let registers = data.holding_registers[start..end].to_vec();
                    Response::ReadHoldingRegisters(registers)
                } else {
                    return future::ready(Err(ExceptionCode::IllegalDataAddress));
                }
            }
            Request::ReadInputRegisters(addr, qty) => {
                println!("Read {qty} input register(s) starting at {addr}");
                let start = addr as usize;
                let end = start + qty as usize;
                if end <= data.input_registers.len() {
                    let registers = data.input_registers[start..end].to_vec();
                    Response::ReadInputRegisters(registers)
                } else {
                    return future::ready(Err(ExceptionCode::IllegalDataAddress));
                }
            }
            Request::WriteSingleCoil(addr, value) => {
                let addr = addr as usize;
                if addr < data.coils.len() {
                    println!("Write coil {addr}: {value}");
                    data.coils[addr] = value;
                    Response::WriteSingleCoil(addr as u16, value)
                } else {
                    return future::ready(Err(ExceptionCode::IllegalDataAddress));
                }
            }
            Request::WriteSingleRegister(addr, value) => {
                let addr = addr as usize;
                if addr < data.holding_registers.len() {
                    println!("Write register {addr}: {value}");
                    data.holding_registers[addr] = value;
                    Response::WriteSingleRegister(addr as u16, value)
                } else {
                    return future::ready(Err(ExceptionCode::IllegalDataAddress));
                }
            }
            Request::WriteMultipleCoils(addr, values) => {
                let start = addr as usize;
                let end = start + values.len();
                if end <= data.coils.len() {
                    println!("Write {} coils starting at {addr}", values.len());
                    for (i, &value) in values.iter().enumerate() {
                        data.coils[start + i] = value;
                    }
                    Response::WriteMultipleCoils(addr, values.len() as u16)
                } else {
                    return future::ready(Err(ExceptionCode::IllegalDataAddress));
                }
            }
            Request::WriteMultipleRegisters(addr, values) => {
                let start = addr as usize;
                let end = start + values.len();
                if end <= data.holding_registers.len() {
                    println!("Write {} registers starting at {addr}", values.len());
                    for (i, &value) in values.iter().enumerate() {
                        data.holding_registers[start + i] = value;
                    }
                    Response::WriteMultipleRegisters(addr, values.len() as u16)
                } else {
                    return future::ready(Err(ExceptionCode::IllegalDataAddress));
                }
            }
            _ => {
                return future::ready(Err(ExceptionCode::IllegalFunction));
            }
        };
        future::ready(Ok(response))
    }
}

fn print_register_table(registers: &[u16], start_addr: u16, verbose: bool) {
    if registers.is_empty() {
        return;
    }

    // Print header
    if verbose {
        println!("{:<8} {:<6} {:<8}", "Address", "Value", "Hex");
        println!("{:─<8} {:─<6} {:─<8}", "", "", "");
    } else {
        println!("{:<8} {:<6}", "Address", "Value");
        println!("{:─<8} {:─<6}", "", "");
    }

    // Print data rows
    for (i, &value) in registers.iter().enumerate() {
        let addr = start_addr + i as u16;
        if verbose {
            println!("{addr:<8} {value:<6} 0x{value:04X}");
        } else {
            println!("{addr:<8} {value:<6}");
        }
    }
}

fn print_coil_table(coils: &[bool], start_addr: u16) {
    if coils.is_empty() {
        return;
    }

    // Print header
    println!("{:<8} {:<6}", "Address", "Value");
    println!("{:─<8} {:─<6}", "", "");

    // Print data rows
    for (i, &value) in coils.iter().enumerate() {
        let addr = start_addr + i as u16;
        println!("{:<8} {:<6}", addr, if value { "ON" } else { "OFF" });
    }
}

async fn connect_to_modbus(common: &Common) -> anyhow::Result<client::Context> {
    match (&common.ip, &common.device) {
        (Some(ip), None) => {
            // TCP connection
            let socket_addr = SocketAddr::new(*ip, common.port);
            if common.verbose {
                println!(
                    "Connecting to Modbus TCP server at {ip}:{} (Unit ID: {})...",
                    common.port, common.unit
                );
            }

            match client::tcp::connect(socket_addr).await {
                Ok(mut ctx) => {
                    ctx.set_slave(Slave(common.unit));
                    if common.verbose {
                        println!(
                            "Successfully connected to Modbus TCP server at {ip}:{}",
                            common.port
                        );
                    }
                    Ok(ctx)
                }
                Err(e) => {
                    eprintln!("Failed to connect to {ip}:{} - Error: {e}", common.port);
                    Err(e.into())
                }
            }
        }
        (None, Some(device)) => {
            // RTU connection
            if common.verbose {
                println!(
                    "Connecting to Modbus RTU device at {} (Baud: {}, Unit ID: {})...",
                    device.display(),
                    common.baud,
                    common.unit
                );
            }

            match tokio_serial::SerialStream::open(&tokio_serial::new(
                device.to_string_lossy(),
                common.baud,
            )) {
                Ok(mut serial) => {
                    // Disable exclusive access for virtual ports
                    if let Err(e) = serial.set_exclusive(false) {
                        if common.verbose {
                            println!("Warning: Could not disable exclusive access: {e}");
                        }
                    }
                    let ctx = client::rtu::attach_slave(serial, Slave(common.unit));
                    if common.verbose {
                        println!(
                            "Successfully connected to Modbus RTU device at {}",
                            device.display()
                        );
                    }
                    Ok(ctx)
                }
                Err(e) => {
                    eprintln!("Failed to connect to {} - Error: {e}", device.display());
                    Err(e.into())
                }
            }
        }
        (None, None) => Err(anyhow::anyhow!(
            "Must specify either --ip for TCP or --device for RTU"
        )),
        (Some(_), Some(_)) => Err(anyhow::anyhow!("Cannot specify both --ip and --device")),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Command::Read { area } => match area {
            ReadArea::Coil { start, qty, common } => {
                let mut client = connect_to_modbus(&common).await?;

                match client.read_coils(start, qty).await {
                    Ok(response) => match response {
                        Ok(coils) => {
                            println!("Read {} coil(s) (Unit ID: {}):", coils.len(), common.unit);
                            print_coil_table(&coils, start);
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
                let mut client = connect_to_modbus(&common).await?;

                match client.read_discrete_inputs(start, qty).await {
                    Ok(response) => match response {
                        Ok(inputs) => {
                            println!(
                                "Read {} discrete input(s) (Unit ID: {}):",
                                inputs.len(),
                                common.unit
                            );
                            print_coil_table(&inputs, start);
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
                let mut client = connect_to_modbus(&common).await?;

                match client.read_holding_registers(start, qty).await {
                    Ok(response) => match response {
                        Ok(registers) => {
                            println!(
                                "Read {} holding register(s) (Unit ID: {}):",
                                registers.len(),
                                common.unit
                            );
                            print_register_table(&registers, start, common.verbose);
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
                let mut client = connect_to_modbus(&common).await?;

                match client.read_input_registers(start, qty).await {
                    Ok(response) => match response {
                        Ok(registers) => {
                            println!(
                                "Read {} input register(s) (Unit ID: {}):",
                                registers.len(),
                                common.unit
                            );
                            print_register_table(&registers, start, common.verbose);
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
            WriteArea::Coil {
                start,
                values,
                common,
            } => {
                let mut client = connect_to_modbus(&common).await?;

                // Convert u16 values to bool values (0 = false, non-zero = true)
                let bool_values: Vec<bool> = values.iter().map(|&v| v != 0).collect();

                if bool_values.len() == 1 {
                    // Single coil write (FC 5)
                    match client.write_single_coil(start, bool_values[0]).await {
                        Ok(response) => match response {
                            Ok(_) => {
                                println!(
                                    "Wrote coil at address {start} with value {} (Unit ID: {})",
                                    if bool_values[0] { "ON" } else { "OFF" },
                                    common.unit
                                );
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
                    match client.write_multiple_coils(start, &bool_values).await {
                        Ok(response) => match response {
                            Ok(_) => {
                                println!(
                                    "Wrote {} coil(s) starting at address {} (Unit ID: {})",
                                    bool_values.len(),
                                    start,
                                    common.unit
                                );
                                print_coil_table(&bool_values, start);
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
                let mut client = connect_to_modbus(&common).await?;

                if values.len() == 1 {
                    // Single register write (FC 6)
                    match client.write_single_register(start, values[0]).await {
                        Ok(response) => match response {
                            Ok(_) => {
                                if common.verbose {
                                    println!(
                                        "Wrote holding register at address {} with value {} (0x{:04X}) (Unit ID: {})",
                                        start, values[0], values[0], common.unit
                                    );
                                } else {
                                    println!(
                                        "Wrote holding register at address {} with value {} (Unit ID: {})",
                                        start, values[0], common.unit
                                    );
                                }
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
                    match client.write_multiple_registers(start, &values).await {
                        Ok(response) => match response {
                            Ok(_) => {
                                println!(
                                    "Wrote {} holding register(s) starting at address {} (Unit ID: {})",
                                    values.len(),
                                    start,
                                    common.unit
                                );
                                print_register_table(&values, start, common.verbose);
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
            ip,
            device,
            port,
            baud,
            unit,
            num_coils,
            num_discrete,
            num_holding,
            num_input,
            verbose,
        } => {
            // Auto-detect TCP vs RTU based on arguments
            // Create shared data storage
            let data = Arc::new(Mutex::new(ModbusData::new(
                num_coils,
                num_discrete,
                num_holding,
                num_input,
            )));

            // Print common configuration
            let print_config = || {
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
            };

            match (ip, device) {
                (Some(ip_addr), None) => {
                    // TCP Server
                    println!("Starting Modbus TCP server on {ip_addr}:{port}");
                    print_config();

                    let socket_addr = SocketAddr::new(ip_addr, port);
                    let listener = tokio::net::TcpListener::bind(socket_addr).await?;
                    println!("Modbus TCP server listening on {ip_addr}:{port}");
                    println!("Press Ctrl+C to stop the server");

                    let server = Server::new(listener);
                    let service = ModbusService::new(data);

                    let on_connected = move |stream, socket_addr| {
                        let service = service.clone();
                        async move {
                            println!("Client connected: {socket_addr}");
                            tokio_modbus::server::tcp::accept_tcp_connection(
                                stream,
                                socket_addr,
                                |_| Ok(Some(service.clone())),
                            )
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
                (None, Some(device_path)) => {
                    // RTU Server
                    println!("Starting Modbus RTU server on {}", device_path.display());
                    print_config();

                    println!("Using baud rate: {baud}");

                    match tokio_serial::SerialStream::open(&tokio_serial::new(
                        device_path.to_string_lossy(),
                        baud,
                    )) {
                        Ok(mut serial) => {
                            // Disable exclusive access for virtual ports
                            if let Err(e) = serial.set_exclusive(false) {
                                println!("Warning: Could not disable exclusive access: {e}");
                            }

                            let rtu_server = rtu::Server::new(serial);
                            let service = ModbusService::new(data);
                            println!("Modbus RTU server listening on {}", device_path.display());
                            println!("Press Ctrl+C to stop the server");

                            let serve_task =
                                tokio::spawn(
                                    async move { rtu_server.serve_forever(service).await },
                                );

                            // Wait for Ctrl+C
                            tokio::signal::ctrl_c().await?;
                            println!("\nStopping RTU server...");

                            // Abort the serve task
                            serve_task.abort();
                            println!("RTU server stopped");
                        }
                        Err(e) => {
                            eprintln!(
                                "Failed to open serial device {}: {}",
                                device_path.display(),
                                e
                            );
                            return Err(e.into());
                        }
                    }
                }
                (None, None) => {
                    // Default to TCP on 0.0.0.0:502
                    let ip_addr = IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0));
                    println!("Starting Modbus TCP server on {ip_addr}:{port} (default)");
                    print_config();

                    let socket_addr = SocketAddr::new(ip_addr, port);
                    let listener = tokio::net::TcpListener::bind(socket_addr).await?;
                    println!("Modbus TCP server listening on {ip_addr}:{port}");
                    println!("Press Ctrl+C to stop the server");

                    let server = Server::new(listener);
                    let service = ModbusService::new(data);

                    let on_connected = move |stream, socket_addr| {
                        let service = service.clone();
                        async move {
                            println!("Client connected: {socket_addr}");
                            tokio_modbus::server::tcp::accept_tcp_connection(
                                stream,
                                socket_addr,
                                |_| Ok(Some(service.clone())),
                            )
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
                (Some(_), Some(_)) => {
                    // This should be prevented by clap conflicts
                    return Err(anyhow::anyhow!("Cannot specify both --ip and --device"));
                }
            }
        }
    }

    Ok(())
}
