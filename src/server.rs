use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio_modbus::server::{Service, rtu, tcp::Server};
use tokio_modbus::prelude::*;

#[derive(Debug)]
pub struct ModbusData {
    pub coils: Vec<bool>,
    pub discrete_inputs: Vec<bool>,
    pub holding_registers: Vec<u16>,
    pub input_registers: Vec<u16>,
}

impl ModbusData {
    pub fn new(num_coils: u16, num_discrete: u16, num_holding: u16, num_input: u16) -> Self {
        Self {
            coils: vec![false; num_coils as usize],
            discrete_inputs: vec![false; num_discrete as usize],
            holding_registers: (0..num_holding).collect(),
            input_registers: (0..num_input).collect(),
        }
    }
}

#[derive(Clone)]
pub struct ModbusService {
    data: Arc<tokio::sync::RwLock<ModbusData>>,
}

impl ModbusService {
    pub fn new(data: Arc<tokio::sync::RwLock<ModbusData>>) -> Self {
        Self { data }
    }
}

impl Service for ModbusService {
    type Request = Request<'static>;
    type Response = Response;
    type Exception = ExceptionCode;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Exception>> + Send>,
    >;

    fn call(&self, req: Self::Request) -> Self::Future {
        let data = self.data.clone();
        Box::pin(async move {
            let mut data = data.write().await;

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
                        return Err(ExceptionCode::IllegalDataAddress);
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
                        return Err(ExceptionCode::IllegalDataAddress);
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
                        return Err(ExceptionCode::IllegalDataAddress);
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
                        return Err(ExceptionCode::IllegalDataAddress);
                    }
                }
                Request::WriteSingleCoil(addr, value) => {
                    let addr = addr as usize;
                    if addr < data.coils.len() {
                        println!("Write coil {addr}: {value}");
                        data.coils[addr] = value;
                        Response::WriteSingleCoil(addr as u16, value)
                    } else {
                        return Err(ExceptionCode::IllegalDataAddress);
                    }
                }
                Request::WriteSingleRegister(addr, value) => {
                    let addr = addr as usize;
                    if addr < data.holding_registers.len() {
                        println!("Write register {addr}: {value}");
                        data.holding_registers[addr] = value;
                        Response::WriteSingleRegister(addr as u16, value)
                    } else {
                        return Err(ExceptionCode::IllegalDataAddress);
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
                        return Err(ExceptionCode::IllegalDataAddress);
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
                        return Err(ExceptionCode::IllegalDataAddress);
                    }
                }
                _ => {
                    return Err(ExceptionCode::IllegalFunction);
                }
            };
            Ok(response)
        })
    }
}

pub async fn run_tcp_server(
    ip_addr: IpAddr,
    port: u16,
    data: Arc<tokio::sync::RwLock<ModbusData>>,
) -> anyhow::Result<()> {
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
    Ok(())
}

pub async fn run_rtu_server(
    device_path: &std::path::Path,
    baud: u32,
    data: Arc<tokio::sync::RwLock<ModbusData>>,
) -> anyhow::Result<()> {
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

            let serve_task = tokio::spawn(async move { rtu_server.serve_forever(service).await });

            // Wait for Ctrl+C
            tokio::signal::ctrl_c().await?;
            println!("\nStopping RTU server...");

            // Abort the serve task
            serve_task.abort();
            println!("RTU server stopped");
            Ok(())
        }
        Err(e) => {
            eprintln!(
                "Failed to open serial device {}: {}",
                device_path.display(),
                e
            );
            Err(e.into())
        }
    }
}