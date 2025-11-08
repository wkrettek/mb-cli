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
                    eprintln!(
                        "Connection to {ip}:{} timed out after {} seconds",
                        common.port, common.timeout
                    );
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
            })
            .await
            {
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
                    eprintln!(
                        "Connection to {} timed out after {} seconds",
                        device.display(),
                        common.timeout
                    );
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_modbus::prelude::ExceptionCode;

    #[test]
    fn test_handle_modbus_response_with_timeout_success() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let success_result: Result<
                Result<Result<Vec<bool>, ExceptionCode>, tokio_modbus::Error>,
                tokio::time::error::Elapsed,
            > = Ok(Ok(Ok([true, false, true].to_vec())));

            let result =
                handle_modbus_response_with_timeout(success_result, "test operation", 5).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), [true, false, true]);
        });
    }

    #[test]
    fn test_handle_modbus_response_with_timeout_exception() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let exception_result: Result<
                Result<Result<Vec<bool>, ExceptionCode>, tokio_modbus::Error>,
                tokio::time::error::Elapsed,
            > = Ok(Ok(Err(ExceptionCode::IllegalDataAddress)));

            let result =
                handle_modbus_response_with_timeout(exception_result, "test operation", 5).await;
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("Modbus exception"));
        });
    }

    #[tokio::test]
    async fn test_handle_modbus_response_with_timeout_elapsed() {
        // Simulate a timeout by creating an Elapsed error
        use std::future::Future;
        use std::pin::Pin;
        use std::task::{Context, Poll};

        struct TimeoutFuture;
        impl Future for TimeoutFuture {
            type Output = Result<Result<Vec<bool>, ExceptionCode>, tokio_modbus::Error>;
            fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
                Poll::Pending // Never completes
            }
        }

        let result = tokio::time::timeout(Duration::from_millis(1), TimeoutFuture).await;

        let timeout_result = handle_modbus_response_with_timeout(result, "test operation", 5).await;
        assert!(timeout_result.is_err());
        assert!(timeout_result
            .unwrap_err()
            .to_string()
            .contains("Operation timeout"));
    }

    #[test]
    fn test_handle_modbus_response_with_timeout_modbus_error() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let io_error =
                std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "connection failed");
            let modbus_error_result: Result<
                Result<Result<Vec<bool>, ExceptionCode>, tokio_modbus::Error>,
                tokio::time::error::Elapsed,
            > = Ok(Err(tokio_modbus::Error::Transport(io_error)));

            let result =
                handle_modbus_response_with_timeout(modbus_error_result, "test operation", 5).await;
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("connection failed"));
        });
    }

    // Test timeout configuration ranges
    #[test]
    fn test_timeout_duration_creation() {
        let duration1 = Duration::from_secs(1);
        assert_eq!(duration1.as_secs(), 1);

        let duration30 = Duration::from_secs(30);
        assert_eq!(duration30.as_secs(), 30);

        let duration_max = Duration::from_secs(u64::MAX);
        assert_eq!(duration_max.as_secs(), u64::MAX);
    }
}
