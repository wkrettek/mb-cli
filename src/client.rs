use crate::cli::Common;
use std::net::SocketAddr;
use tokio::time::{timeout, Duration};
use tokio_modbus::client;
use tokio_modbus::prelude::*;

pub async fn connect_to_modbus(common: &Common) -> anyhow::Result<client::Context> {
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

            let connect_timeout = Duration::from_secs(common.timeout);
            match timeout(connect_timeout, client::tcp::connect(socket_addr)).await {
                Ok(connect_result) => match connect_result {
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
                },
                Err(_) => {
                    eprintln!("Connection to {ip}:{} timed out after {} seconds", common.port, common.timeout);
                    Err(anyhow::anyhow!("Connection timeout"))
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

            let connect_timeout = Duration::from_secs(common.timeout);
            match timeout(connect_timeout, async {
                tokio_serial::SerialStream::open(&tokio_serial::new(
                    device.to_string_lossy(),
                    common.baud,
                ))
            }).await {
                Ok(serial_result) => match serial_result {
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
                },
                Err(_) => {
                    eprintln!("Connection to {} timed out after {} seconds", device.display(), common.timeout);
                    Err(anyhow::anyhow!("Connection timeout"))
                }
            }
        }
        (None, None) => Err(anyhow::anyhow!(
            "Must specify either --ip for TCP or --device for RTU"
        )),
        (Some(_), Some(_)) => Err(anyhow::anyhow!("Cannot specify both --ip and --device")),
    }
}

// Generic helper for handling Modbus response errors with timeout
pub async fn handle_modbus_response_with_timeout<T, E>(
    result: Result<Result<Result<T, E>, tokio_modbus::Error>, tokio::time::error::Elapsed>,
    operation: &str,
    timeout_secs: u64,
) -> anyhow::Result<T>
where
    E: std::fmt::Debug,
{
    match result {
        Ok(modbus_result) => match modbus_result {
            Ok(response) => match response {
                Ok(data) => Ok(data),
                Err(exception) => {
                    eprintln!("Modbus exception response: {exception:?}");
                    Err(anyhow::anyhow!("Modbus exception: {:?}", exception))
                }
            },
            Err(e) => {
                eprintln!("Failed to {operation}: {e}");
                Err(e.into())
            }
        },
        Err(_) => {
            eprintln!("Operation '{operation}' timed out after {timeout_secs} seconds");
            Err(anyhow::anyhow!("Operation timeout"))
        }
    }
}

// Helper function to perform Modbus operations with timeout
pub async fn modbus_operation_with_timeout<T, E, F, Fut>(
    operation: F,
    operation_name: &str,
    timeout_secs: u64,
) -> anyhow::Result<T>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<Result<T, E>, tokio_modbus::Error>>,
    E: std::fmt::Debug,
{
    let op_timeout = Duration::from_secs(timeout_secs);
    let result = timeout(op_timeout, operation()).await;
    handle_modbus_response_with_timeout(result, operation_name, timeout_secs).await
}