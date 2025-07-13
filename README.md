# mb-cli

A fast and reliable Modbus TCP and RTU client/server command-line tool. Built on top of [tokio-modbus](https://github.com/slowtec/tokio-modbus).

## Features

- **Modbus TCP** and **RTU** support with automatic protocol detection
- **Client operations**: Read and write coils, discrete inputs, holding registers, and input registers
- **Server mode**: Run a Modbus TCP or RTU server
- **Table output**: Clean, formatted output

## Installation

```bash
cargo install --locked mb-cli
```

### From Source

```bash
# Clone the repository
git clone <repository-url>
cd mb-cli

# Build and install
cargo install --path .
```

### Prerequisites

- Rust 1.76+ with Cargo
- For RTU: Access to serial ports (may require permissions)

## Usage

### Basic Examples

#### Read Operations

```bash
# Read holding registers from TCP server
mb read holding --ip 192.168.1.100 --addr 0 --qty 10

# Read coils from RTU device
mb read coil --device /dev/ttyUSB0 --addr 0 --qty 8 --baud 9600

# Read with verbose output
mb read holding --ip 127.0.0.1 --addr 100 --qty 5 --verbose
```

#### Write Operations

```bash
# Write single coil
mb write coil --ip 192.168.1.100 --addr 0 --value 1

# Write multiple coils
mb write coil --ip 192.168.1.100 --addr 0 --value 1,0,1,1

# Write holding register
mb write holding --ip 192.168.1.100 --addr 100 --value 42

# Write multiple registers
mb write holding --ip 192.168.1.100 --addr 100 --value 42,43,44
```

#### Server Mode

```bash
# Start TCP server
mb server --ip 0.0.0.0 --port 502

# Start RTU server
mb server --device /dev/ttyUSB0 --baud 9600

# Server with custom memory layout
mb server --ip 0.0.0.0 --num-coils 1000 --num-holding 500
```

### Command Reference

#### Global Options

- `--timeout <seconds>` - Timeout for connections and operations (default: 5)
- `--verbose` / `-v` - Enable verbose output
- `--unit <id>` - Modbus unit/slave ID (default: 0 for client, 1 for server)

#### TCP Options

- `--ip <address>` - IP address to connect to or bind to
- `--port <port>` - TCP port (default: 502)

#### RTU Options  

- `--device <path>` - Serial device path (e.g., /dev/ttyUSB0, COM1)
- `--baud <rate>` - Baud rate (default: 9600)

#### Read Commands

```bash
mb read <area> --addr <address> [--qty <quantity>] [connection options]
```

Areas: `coil`, `discrete`, `holding`, `input`

- Coils and discrete inputs: 1-2000 per request
- Registers: 1-125 per request

#### Write Commands

```bash
mb write <area> --addr <address> --value <values> [connection options]
```

Areas: `coil`, `holding`

- Values: Comma-separated for multiple writes
- Coils: 0=OFF, 1=ON (or any non-zero=ON)

#### Server Command

```bash
mb server [connection options] [memory options]
```

Memory options:
- `--num-coils <count>` - Number of coils (default: 10000)
- `--num-discrete <count>` - Number of discrete inputs (default: 10000)
- `--num-holding <count>` - Number of holding registers (default: 10000)
- `--num-input <count>` - Number of input registers (default: 10000)

### Protocol Detection

The tool automatically detects the protocol based on arguments:

- `--ip` specified → Modbus TCP
- `--device` specified → Modbus RTU
- Both specified → Error (mutually exclusive)
- Neither specified → Error (must specify one)

### Examples by Use Case

#### Industrial Automation

```bash
# Read PLC outputs
mb read coil --ip 192.168.1.10 --addr 0 --qty 16

# Read sensor values  
mb read input --ip 192.168.1.10 --addr 1000 --qty 4 --verbose

# Write setpoint
mb write holding --ip 192.168.1.10 --addr 2000 --value 1500
```

#### Serial Device Communication

```bash
# Read from RTU device with custom baud rate
mb read holding --device /dev/ttyUSB0 --baud 19200 --unit 1 --addr 0 --qty 10

# Write to RTU device with timeout
mb write coil --device /dev/ttyUSB0 --unit 2 --addr 0 --value 1 --timeout 10
```

#### Testing and Development

```bash
# Start test server
mb server --ip 127.0.0.1 --port 5020 &

# Test client against local server
mb read holding --ip 127.0.0.1 --port 5020 --addr 0 --qty 5

# Test with verbose logging
mb read coil --ip 127.0.0.1 --port 5020 --addr 0 --qty 8 --verbose
```

## Error Handling

The tool provides detailed error messages for common issues:

- **Connection timeouts**: Clear timeout messages with duration
- **Modbus exceptions**: Detailed exception codes (IllegalDataAddress, etc.)
- **Validation errors**: Specification-compliant range checking with explanations
- **Connection failures**: Network and serial port error details
