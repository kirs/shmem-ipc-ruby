# ShmemIpc Ruby Gem

A Ruby binding for the [shmem-ipc](https://github.com/diwic/shmem-ipc) Rust crate, providing high-performance shared memory inter-process communication using lock-free ring buffers.

## Features

- **Zero-copy IPC**: Data is shared directly between processes without copying
- **Lock-free**: Uses atomic operations for maximum performance
- **Memory safe**: Leverages Linux memfd sealing for safety between untrusted processes
- **High throughput**: Optimized for streaming large amounts of data
- **Multi-type support**: Separate optimized classes for floats, integers, and bytes
- **Ruby-friendly**: Clean Ruby API wrapping the Rust implementation

## Installation

Add this line to your application's Gemfile:

```ruby
gem 'shmem_ipc'
```

And then execute:

    $ bundle install

Or install it yourself as:

    $ gem install shmem_ipc

## Requirements

- **Linux only**: This library uses Linux-specific features (memfd, eventfd)
- **Ruby 2.7+**: Required for the native extension
- **Rust toolchain**: Required for compilation

## Quick Start

### Different Data Types

The library supports three optimized data types with separate classes:

```ruby
require 'shmem_ipc'

# Float data (64-bit floats)
float_sender, float_receiver = ShmemIpc.create_float_ring(1000)
float_sender.send([1.0, 2.0, 3.14, 42.0])
result = float_receiver.receive
puts "Floats: #{result[:data]}"

# Integer data (64-bit signed integers)
int_sender, int_receiver = ShmemIpc.create_integer_ring(1000)  
int_sender.send([1, 2, 42, -10])
result = int_receiver.receive
puts "Integers: #{result[:data]}"

# Byte data (strings and binary data)
byte_sender, byte_receiver = ShmemIpc.create_byte_ring(1000)
byte_sender.send("Hello, World!")
result = byte_receiver.receive
puts "String: #{result[:data]}"
```

### Simple In-Process Example

```ruby
require 'shmem_ipc'

# Create a float ring buffer (default)
sender, receiver = ShmemIpc.create_ring(1000)

# Send some data
result = sender.send([1.0, 2.0, 3.14, 42.0])
puts "Sent data, remaining space: #{result[:remaining]}"

# Receive the data
result = receiver.receive
puts "Received: #{result[:data]}"
```

### Inter-Process Communication

For communication between separate processes, you need to pass file descriptors:

**Server process:**
```ruby
require 'shmem_ipc'

# Create receiver for specific data type
receiver = ShmemIpc::FloatRingReceiver.new(10000)

# Get file descriptors to pass to client
memfd, empty_signal, full_signal = receiver.get_file_descriptors

# ... pass these FDs to client process via your IPC mechanism ...
# (Unix domain sockets, D-Bus, pipes, etc.)

# Receive data
loop do
  result = receiver.receive
  unless result[:data].empty?
    puts "Received #{result[:data].length} items"
  end
end
```

**Client process:**
```ruby
require 'shmem_ipc'

# Connect using FDs received from server
sender = ShmemIpc::FloatRingSender.open(10000, memfd, empty_signal, full_signal)

# Send data
data = Array.new(1000) { |i| i.to_f }
result = sender.send(data)
puts "Sent #{data.length} items"
```

## API Reference

### Convenience Methods

- `ShmemIpc.create_float_ring(capacity)` - Creates float sender/receiver pair
- `ShmemIpc.create_integer_ring(capacity)` - Creates integer sender/receiver pair  
- `ShmemIpc.create_byte_ring(capacity)` - Creates byte sender/receiver pair
- `ShmemIpc.create_ring(capacity)` - Creates float pair (backward compatibility)

### Float Ring Buffers

#### ShmemIpc::FloatRingSender

**Class Methods:**
- `new(capacity)` - Creates a new sender with specified capacity
- `open(capacity, memfd, empty_signal, full_signal)` - Opens sender using file descriptors

**Instance Methods:**
- `send(data)` - Sends float data (single value or array)
  - Returns: `{remaining: int, signal: bool}` hash
- `get_file_descriptors()` - Returns array of file descriptors

#### ShmemIpc::FloatRingReceiver

**Class Methods:**
- `new(capacity)` - Creates a new receiver with specified capacity
- `open(capacity, memfd, empty_signal, full_signal)` - Opens receiver using file descriptors

**Instance Methods:**
- `receive()` - Receives available float data
  - Returns: `{data: array, remaining: int, signal: bool}` hash
- `get_file_descriptors()` - Returns array of file descriptors

### Integer Ring Buffers

#### ShmemIpc::IntegerRingSender / ShmemIpc::IntegerRingReceiver

Same API as float classes but for 64-bit signed integers.

### Byte Ring Buffers

#### ShmemIpc::ByteRingSender

**Instance Methods:**
- `send(data)` - Sends byte data (string or array of integers 0-255)

#### ShmemIpc::ByteRingReceiver

**Instance Methods:**
- `receive()` - Receives byte data
  - Returns: `{data: string, remaining: int, signal: bool}` hash

## Data Type Details

### Floats (`f64`)
- 64-bit IEEE 754 floating point numbers
- Ruby `Float` values
- Use for: sensor data, audio samples, mathematical computations

### Integers (`i64`)  
- 64-bit signed integers (-9,223,372,036,854,775,808 to 9,223,372,036,854,775,807)
- Ruby `Integer` values  
- Use for: counters, IDs, timestamps, discrete measurements

### Bytes (`u8`)
- Raw byte data
- Input: Ruby `String` or `Array` of integers (0-255)
- Output: Ruby `String` (binary-safe)
- Use for: text messages, binary protocols, file data, images

## Performance Characteristics

- **Lock-free**: No mutexes or semaphores, uses atomic operations only
- **Zero-copy**: Data is written once by sender, read directly by receiver
- **Batch operations**: Single atomic operation per batch of items
- **Cache-friendly**: Sequential memory layout optimized for CPU caches
- **Type-optimized**: Each data type uses optimal memory layout

## Use Cases

### Audio Processing
```ruby
# Audio samples as floats
audio_sender, audio_receiver = ShmemIpc.create_float_ring(4096)
samples = Array.new(1024) { |i| Math.sin(2 * Math::PI * 440 * i / 44100) }
audio_sender.send(samples)
```

### Telemetry Data
```ruby
# Sensor readings as integers
sensor_sender, sensor_receiver = ShmemIpc.create_integer_ring(1000)
sensor_sender.send([temperature_celsius * 100, humidity_percent, pressure_pa])
```

### Message Passing
```ruby
# JSON messages as bytes
msg_sender, msg_receiver = ShmemIpc.create_byte_ring(8192)
msg_sender.send(JSON.generate({action: "update", data: values}))
```

## Examples

See the `examples/` directory for complete working examples:

- `simple_example.rb` - Basic usage of all data types
- `typed_data_example.rb` - Comprehensive multi-type demonstration
- `server.rb` / `client.rb` - Inter-process communication
- `README.md` - Detailed example documentation

## Limitations

- **Linux only**: Uses Linux-specific kernel features
- **Single producer/consumer**: Each ring buffer supports one sender and one receiver
- **Type separation**: Cannot mix data types in the same ring buffer
- **Untrusted processes**: Designed for communication between potentially untrusted processes

## Development

After checking out the repo, run:

```bash
bundle install
rake compile  # Build the native extension
rake test     # Run tests
```

## Contributing

Bug reports and pull requests are welcome on GitHub at https://github.com/diwic/shmem-ipc.

## License

The gem is available as open source under the terms of the [Apache 2.0 License](https://opensource.org/licenses/Apache-2.0).