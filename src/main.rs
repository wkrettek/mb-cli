use std::sync::Arc;
use tokio_modbus::client::{Reader, Writer};

mod cli;
mod client;
mod server;
mod table;

use cli::{Cli, Command, ReadArea, WriteArea};
use client::{connect_to_modbus, modbus_operation_with_timeout};
use server::{ModbusData, run_rtu_server, run_tcp_server};
use table::{print_coil_table, print_register_table};

use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Command::Read { area } => match area {
            ReadArea::Coil { start, qty, common } => {
                let mut client = connect_to_modbus(&common).await?;
                let coils = modbus_operation_with_timeout(
                    || client.read_coils(start, qty),
                    "read coils",
                    common.timeout,
                )
                .await?;
                println!("Read {} coil(s) (Unit ID: {}):", coils.len(), common.unit);
                print_coil_table(&coils, start);
            }
            ReadArea::Discrete { start, qty, common } => {
                let mut client = connect_to_modbus(&common).await?;
                let inputs = modbus_operation_with_timeout(
                    || client.read_discrete_inputs(start, qty),
                    "read discrete inputs",
                    common.timeout,
                )
                .await?;
                println!(
                    "Read {} discrete input(s) (Unit ID: {}):",
                    inputs.len(),
                    common.unit
                );
                print_coil_table(&inputs, start);
            }
            ReadArea::Holding { start, qty, common } => {
                let mut client = connect_to_modbus(&common).await?;
                let registers = modbus_operation_with_timeout(
                    || client.read_holding_registers(start, qty),
                    "read holding registers",
                    common.timeout,
                )
                .await?;
                println!(
                    "Read {} holding register(s) (Unit ID: {}):",
                    registers.len(),
                    common.unit
                );
                print_register_table(&registers, start, common.verbose);
            }
            ReadArea::Input { start, qty, common } => {
                let mut client = connect_to_modbus(&common).await?;
                let registers = modbus_operation_with_timeout(
                    || client.read_input_registers(start, qty),
                    "read input registers",
                    common.timeout,
                )
                .await?;
                println!(
                    "Read {} input register(s) (Unit ID: {}):",
                    registers.len(),
                    common.unit
                );
                print_register_table(&registers, start, common.verbose);
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
                    modbus_operation_with_timeout(
                        || client.write_single_coil(start, bool_values[0]),
                        "write coil",
                        common.timeout,
                    )
                    .await?;
                    println!(
                        "Wrote coil at address {start} with value {} (Unit ID: {})",
                        if bool_values[0] { "ON" } else { "OFF" },
                        common.unit
                    );
                } else {
                    // Multiple coils write (FC 15)
                    modbus_operation_with_timeout(
                        || client.write_multiple_coils(start, &bool_values),
                        "write coils",
                        common.timeout,
                    )
                    .await?;
                    println!(
                        "Wrote {} coil(s) starting at address {} (Unit ID: {})",
                        bool_values.len(),
                        start,
                        common.unit
                    );
                    print_coil_table(&bool_values, start);
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
                    modbus_operation_with_timeout(
                        || client.write_single_register(start, values[0]),
                        "write register",
                        common.timeout,
                    )
                    .await?;
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
                } else {
                    // Multiple registers write (FC 16)
                    modbus_operation_with_timeout(
                        || client.write_multiple_registers(start, &values),
                        "write registers",
                        common.timeout,
                    )
                    .await?;
                    println!(
                        "Wrote {} holding register(s) starting at address {} (Unit ID: {})",
                        values.len(),
                        start,
                        common.unit
                    );
                    print_register_table(&values, start, common.verbose);
                }
            }
        },

        Command::Server {
            ip,
            device,
            port,
            baud,
            parity,
            stop_bits,
            data_bits,
            unit: _,
            num_coils,
            num_discrete,
            num_holding,
            num_input,
            verbose: _,
        } => {
            // Auto-detect TCP vs RTU based on arguments
            // Create shared data storage
            let data = Arc::new(tokio::sync::RwLock::new(ModbusData::new(
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
                    run_tcp_server(ip_addr, port, data).await?;
                }
                (None, Some(device_path)) => {
                    // RTU Server
                    println!("Starting Modbus RTU server on {}", device_path.display());
                    print_config();
                    run_rtu_server(&device_path, baud, &parity, &stop_bits, &data_bits, data).await?;
                }
                (None, None) => {
                    // Default to TCP on 0.0.0.0:502
                    use std::net::{IpAddr, Ipv4Addr};
                    let ip_addr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
                    println!("Starting Modbus TCP server on {ip_addr}:{port} (default)");
                    print_config();
                    run_tcp_server(ip_addr, port, data).await?;
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
