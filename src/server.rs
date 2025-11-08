use crate::cli::{DataBits, Parity, StopBits};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio_modbus::prelude::*;
use tokio_modbus::server::{Service, rtu, tcp::Server};

// Convert CLI enums to tokio_serial types
fn convert_parity(parity: &Parity) -> tokio_serial::Parity {
    match parity {
        Parity::None => tokio_serial::Parity::None,
        Parity::Even => tokio_serial::Parity::Even,
        Parity::Odd => tokio_serial::Parity::Odd,
    }
}

fn convert_stop_bits(stop_bits: &StopBits) -> tokio_serial::StopBits {
    match stop_bits {
        StopBits::One => tokio_serial::StopBits::One,
        StopBits::Two => tokio_serial::StopBits::Two,
    }
}

fn convert_data_bits(data_bits: &DataBits) -> tokio_serial::DataBits {
    match data_bits {
        DataBits::Five => tokio_serial::DataBits::Five,
        DataBits::Six => tokio_serial::DataBits::Six,
        DataBits::Seven => tokio_serial::DataBits::Seven,
        DataBits::Eight => tokio_serial::DataBits::Eight,
    }
}

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
    parity: &Parity,
    stop_bits: &StopBits,
    data_bits: &DataBits,
    data: Arc<tokio::sync::RwLock<ModbusData>>,
) -> anyhow::Result<()> {
    println!("Serial Configuration:");
    println!("  Baud Rate: {baud}");
    println!("  Parity: {:?}", parity);
    println!("  Stop Bits: {:?}", stop_bits);
    println!("  Data Bits: {:?}", data_bits);

    let builder = tokio_serial::new(device_path.to_string_lossy(), baud)
        .parity(convert_parity(parity))
        .stop_bits(convert_stop_bits(stop_bits))
        .data_bits(convert_data_bits(data_bits));

    match tokio_serial::SerialStream::open(&builder)
    {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_modbus::prelude::{ExceptionCode, Request, Response};

    #[test]
    fn test_modbus_data_new() {
        let data = ModbusData::new(100, 200, 300, 400);

        assert_eq!(data.coils.len(), 100);
        assert_eq!(data.discrete_inputs.len(), 200);
        assert_eq!(data.holding_registers.len(), 300);
        assert_eq!(data.input_registers.len(), 400);

        // All coils and discrete inputs should be false initially
        assert!(data.coils.iter().all(|&x| !x));
        assert!(data.discrete_inputs.iter().all(|&x| !x));

        // Holding and input registers should be initialized with address = value
        for (addr, &value) in data.holding_registers.iter().enumerate() {
            assert_eq!(addr as u16, value);
        }
        for (addr, &value) in data.input_registers.iter().enumerate() {
            assert_eq!(addr as u16, value);
        }
    }

    #[test]
    fn test_modbus_data_small_sizes() {
        let data = ModbusData::new(1, 2, 3, 4);

        assert_eq!(data.coils.len(), 1);
        assert_eq!(data.discrete_inputs.len(), 2);
        assert_eq!(data.holding_registers.len(), 3);
        assert_eq!(data.input_registers.len(), 4);

        assert_eq!(data.holding_registers, [0, 1, 2]);
        assert_eq!(data.input_registers, [0, 1, 2, 3]);
    }

    #[test]
    fn test_modbus_data_zero_sizes() {
        let data = ModbusData::new(0, 0, 0, 0);

        assert!(data.coils.is_empty());
        assert!(data.discrete_inputs.is_empty());
        assert!(data.holding_registers.is_empty());
        assert!(data.input_registers.is_empty());
    }

    #[tokio::test]
    async fn test_modbus_service_read_coils_valid() {
        let data = Arc::new(tokio::sync::RwLock::new(ModbusData::new(10, 10, 10, 10)));

        // Set some coil values for testing
        {
            let mut data_lock = data.write().await;
            data_lock.coils[0] = true;
            data_lock.coils[2] = true;
        }

        let service = ModbusService::new(data);
        let request = Request::ReadCoils(0, 3);

        let result = service.call(request).await;
        assert!(result.is_ok());

        if let Ok(Response::ReadCoils(coils)) = result {
            assert_eq!(coils.len(), 3);
            assert!(coils[0]);
            assert!(!coils[1]);
            assert!(coils[2]);
        } else {
            panic!("Expected ReadCoils response");
        }
    }

    #[tokio::test]
    async fn test_modbus_service_read_coils_out_of_bounds() {
        let data = Arc::new(tokio::sync::RwLock::new(ModbusData::new(5, 5, 5, 5)));
        let service = ModbusService::new(data);

        // Try to read beyond available coils
        let request = Request::ReadCoils(3, 5); // starts at 3, wants 5 coils = addresses 3,4,5,6,7 but only 0-4 exist

        let result = service.call(request).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ExceptionCode::IllegalDataAddress);
    }

    #[tokio::test]
    async fn test_modbus_service_read_holding_registers() {
        let data = Arc::new(tokio::sync::RwLock::new(ModbusData::new(10, 10, 10, 10)));
        let service = ModbusService::new(data);

        let request = Request::ReadHoldingRegisters(5, 3);

        let result = service.call(request).await;
        assert!(result.is_ok());

        if let Ok(Response::ReadHoldingRegisters(registers)) = result {
            assert_eq!(registers.len(), 3);
            assert_eq!(registers[0], 5); // address 5 = value 5
            assert_eq!(registers[1], 6); // address 6 = value 6  
            assert_eq!(registers[2], 7); // address 7 = value 7
        } else {
            panic!("Expected ReadHoldingRegisters response");
        }
    }

    #[tokio::test]
    async fn test_modbus_service_write_single_coil() {
        let data = Arc::new(tokio::sync::RwLock::new(ModbusData::new(10, 10, 10, 10)));
        let service = ModbusService::new(data.clone());

        let request = Request::WriteSingleCoil(3, true);

        let result = service.call(request).await;
        assert!(result.is_ok());

        if let Ok(Response::WriteSingleCoil(addr, value)) = result {
            assert_eq!(addr, 3);
            assert!(value);
        } else {
            panic!("Expected WriteSingleCoil response");
        }

        // Verify the coil was actually set
        let data_lock = data.read().await;
        assert!(data_lock.coils[3]);
    }

    #[tokio::test]
    async fn test_modbus_service_write_single_register() {
        let data = Arc::new(tokio::sync::RwLock::new(ModbusData::new(10, 10, 10, 10)));
        let service = ModbusService::new(data.clone());

        let request = Request::WriteSingleRegister(2, 12345);

        let result = service.call(request).await;
        assert!(result.is_ok());

        if let Ok(Response::WriteSingleRegister(addr, value)) = result {
            assert_eq!(addr, 2);
            assert_eq!(value, 12345);
        } else {
            panic!("Expected WriteSingleRegister response");
        }

        // Verify the register was actually set
        let data_lock = data.read().await;
        assert_eq!(data_lock.holding_registers[2], 12345);
    }

    #[tokio::test]
    async fn test_modbus_service_write_multiple_coils() {
        let data = Arc::new(tokio::sync::RwLock::new(ModbusData::new(10, 10, 10, 10)));
        let service = ModbusService::new(data.clone());

        let values = [true, false, true];
        let request = Request::WriteMultipleCoils(1, values.to_vec().into());

        let result = service.call(request).await;
        assert!(result.is_ok());

        if let Ok(Response::WriteMultipleCoils(addr, qty)) = result {
            assert_eq!(addr, 1);
            assert_eq!(qty, 3);
        } else {
            panic!("Expected WriteMultipleCoils response");
        }

        // Verify the coils were actually set
        let data_lock = data.read().await;
        assert!(data_lock.coils[1]);
        assert!(!data_lock.coils[2]);
        assert!(data_lock.coils[3]);
    }
}
