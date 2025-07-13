# Modbus CLI Tool Development Plan

## Completed
- [x] Implement write operations for coils and holding registers
- [x] Add server subcommand framework
- [x] Implement the server functionality
    - [x] Create in-memory storage for all register types
    - [x] Implement Modbus TCP server using tokio-modbus
    - [x] Initialize values where each address equals its value
    - [x] Handle read/write requests from clients
    - [x] Manual verification of all read/write TCP functionality

- [x] Add support for Modbus RTU (serial) in addition to TCP
    - [x] Add serial port dependencies (tokio-serial)
    - [x] Add --device flag for serial port selection
    - [x] Implement RTU connection logic
    - [x] Add --baud flag for RTU baud rate configuration
    - [x] Handle exclusive access for virtual serial ports
- [x] Implement auto-detection for TCP vs RTU based on --ip or --device flags
    - [x] Make --ip and --device mutually exclusive
    - [x] Auto-select TCP when --ip is provided
    - [x] Auto-select RTU when --device is provided
    - [x] Manual verification of RTU read/write functionality

## Next: High Priority

## Critical Correctness Fixes: High Priority
- [x] **RTU serial port opening**: ~~Replace blocking `SerialStream::open()` with async version~~ (Won't do - current API is correct)
- [x] **RTU server unit ID**: ~~Fix hard-coded slave=1 in RTU server~~ (Won't do - no hard-coded slave found)
- [x] **Async-safe Mutex**: Replace `std::sync::Mutex` with `tokio::sync::RwLock` to prevent blocking async runtime
- [x] **Coil value parsing**: ~~Change from accepting any `u16` to proper `bool` parsing~~ (Won't do - being permissive is better)

## Usability Improvements: Medium Priority  
- [x] **Range validation**: Add Clap range validation for qty (coils: 1-2000, registers: 1-125) with detailed Modbus spec error messages
- [x] **Verbose server logging**: ~~Gate server read/write logs behind `--verbose` flag~~ (Won't do - current output is fine)
- [ ] **Serial port options**: Add optional `--parity`, `--stop-bits`, `--data-bits` flags for RS-485 compatibility

## Polish Improvements: Low Priority
- [x] **Table helper DRY**: Extract common header logic from print_register_table and print_coil_table
- [x] **Transport enum**: ~~Replace nested match with `enum Transport`~~ (Won't do - current tuple matching is clear and idiomatic)

## Before Publishing: Medium Priority
- [ ] Clean up cli output
    - [x] cli
    - [x] Hide more detail behind --verbose/-v
    - Maybe print a table of results?
    - [x] server
        - [ ] print the action and from where (do later)
- [ ] Lint and cleanup
    - [x] DRY violations:
        - [x] Extract common read operation error handling (4x duplication)
        - [x] Extract common write operation error handling (2x duplication) 
        - [x] Extract TCP server setup (2x duplication)
        - [x] Create error handling utilities (handle_modbus_response helper)
    - [ ] Function length:
        - [ ] Break down 411-line main() function
        - [ ] Extract server command handling
    - [ ] Break into multiple files:
        - [ ] cli.rs - CLI structs and parsing
        - [ ] client.rs - Client connection and operations
        - [ ] server.rs - Server implementation  
        - [ ] table.rs - Table formatting
        - [ ] main.rs - Just orchestration
    - [x] Remove dead code:
        - [x] Remove unused `format` field from Common struct
        - [x] Remove unused server variables (_unit, _verbose)
        - [x] Remove unused `arg` import from clap
        - [x] Fix help text examples (--bind → --ip)
    - [ ] Add constants for magic numbers (ports, baud rates)
    - [ ] Get multiple AI reviews
        - [x] o3
        - [ ] gemini
- [ ] Add comprehensive error handling and retry logic
    - [ ] Very good error messages for common issues
        - [ ] inputs and discretes are not writeable
    - [ ] Timeout configuration
- [ ] Add tests for all functionality
    - [ ] Basic integration tests (server on port 5020):
        - [ ] Server startup and shutdown
        - [ ] Read operations (coils, discrete, holding, input registers)
        - [ ] Single writes (coil, holding register)
        - [ ] Multiple writes (coils, holding registers)
        - [ ] Table output formatting
        - [ ] Verbose mode behavior
        - [ ] Failure cases:
            - [ ] Address out of range (IllegalDataAddress exception)
            - [ ] Write to read-only areas (IllegalFunction exception)
            - [ ] Connection to non-existent server
            - [ ] CLI validation (missing args, conflicting args)
    - [ ] Unit tests for parsing
    - [ ] Mock server tests
- [ ] Create comprehensive documentation and examples
    - [ ] README with usage examples
        - Installation
            - cargo install
    - [ ] Example scripts
- [ ] Package and publish
    - [ ] Cargo.toml metadata for publishing
    - [ ] Release preparation

## Future Enhancements
- [ ] Server configuration files (holding file, etc.)
- [ ] Probing functionality (removed from current scope)
