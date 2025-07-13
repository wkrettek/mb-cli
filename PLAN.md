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

## Next: High Priority
- [ ] Add support for Modbus RTU (serial) in addition to TCP
    - [ ] Add serial port dependencies (tokio-serial)
    - [ ] Add --device flag for serial port selection
    - [ ] Implement RTU connection logic
- [ ] Implement auto-detection for TCP vs RTU based on --ip or --device flags
    - [ ] Make --ip and --device mutually exclusive
    - [ ] Auto-select TCP when --ip is provided
    - [ ] Auto-select RTU when --device is provided

## Before Publishing: Medium Priority
- [ ] Clean up cli output
    - [ ] cli
    - [ ] Hide more detail behind --verbose/-v
    - Maybe print a table of results?
    - [ ] server
        - [ ] print the action and from where
- [ ] Lint and cleanup
    - [ ] Use DRY where possible
    - [ ] Break into multiple files
    - [ ] Get multiple AI reviews
        - [ ] o3
        - [ ] gemini
- [ ] Add comprehensive error handling and retry logic
    - [ ] Implement connection retry with backoff
    - [ ] Very good error messages for common issues
    - [ ] Timeout configuration
- [ ] Add tests for all functionality
    - [ ] Unit tests for parsing
    - [ ] Integration tests for client operations
    - [ ] Mock server tests
- [ ] Create comprehensive documentation and examples
    - [ ] README with usage examples
    - [ ] Man page generation
    - [ ] Example scripts
- [ ] Package and publish
    - [ ] Cargo.toml metadata for publishing
    - [ ] Release preparation

## Future Enhancements
- [ ] Server configuration files (holding file, etc.)
- [ ] Probing functionality (removed from current scope)
