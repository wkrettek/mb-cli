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

## Before Publishing: Medium Priority
- [ ] Clean up cli output
    - [x] cli
    - [x] Hide more detail behind --verbose/-v
    - Maybe print a table of results?
    - [x] server
        - [ ] print the action and from where (do later)
- [ ] Lint and cleanup
    - [ ] DRY violations:
        - [ ] Extract common read operation error handling (4x duplication)
        - [ ] Extract common write operation error handling (2x duplication) 
        - [ ] Extract TCP server setup (2x duplication)
        - [ ] Create error handling utilities
    - [ ] Function length:
        - [ ] Break down 411-line main() function
        - [ ] Extract server command handling
    - [ ] Break into multiple files:
        - [ ] cli.rs - CLI structs and parsing
        - [ ] client.rs - Client connection and operations
        - [ ] server.rs - Server implementation  
        - [ ] table.rs - Table formatting
        - [ ] main.rs - Just orchestration
    - [ ] Remove dead code:
        - [ ] Remove unused `format` field from Common struct
        - [ ] Remove unused server variables (_unit, _verbose)
    - [ ] Add constants for magic numbers (ports, baud rates)
    - [ ] Get multiple AI reviews
        - [ ] o3
        - [ ] gemini
- [ ] Add comprehensive error handling and retry logic
    - [ ] Implement connection retry with backoff
    - [ ] Very good error messages for common issues
        - inputs and discretes are not writeable
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
