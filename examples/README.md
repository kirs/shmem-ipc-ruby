# ShmemIpc Ruby Examples

This directory contains examples showing how to use the shmem_ipc Ruby gem.

## Simple Example

`simple_example.rb` - Shows basic usage within a single process:

```ruby
ruby examples/simple_example.rb
```

This creates a sender/receiver pair and demonstrates sending and receiving data.

## Client/Server Example

For inter-process communication, you need to run two separate processes:

### Step 1: Start the Server

```ruby
ruby examples/server.rb
```

This will create a receiver and print file descriptors that need to be passed to the client.

### Step 2: Start the Client

In another terminal, use the file descriptors from the server:

```ruby
ruby examples/client.rb <memfd> <empty_signal> <full_signal>
```

Replace the placeholders with the actual file descriptor numbers printed by the server.

## Real-World Usage

In production, you would typically:

1. Use a proper IPC mechanism (Unix domain sockets, D-Bus, etc.) to pass file descriptors between processes
2. Implement proper error handling and reconnection logic
3. Use event loops or threads to handle the blocking/non-blocking aspects
4. Consider using process supervision to restart failed processes

The shmem-ipc library is designed for high-performance scenarios where you need to minimize latency and maximize throughput between processes.