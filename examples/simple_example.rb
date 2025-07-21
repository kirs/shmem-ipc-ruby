#!/usr/bin/env ruby
# frozen_string_literal: true

require_relative "../lib/shmem_ipc"

puts "ShmemIpc Simple Example"
puts "=" * 30

# Example 1: Float data (default)
puts "\n1. Float Ring Buffer (default behavior)"
puts "Creating float ring buffer with capacity 1000..."
sender, receiver = ShmemIpc.create_ring(1000)  # Defaults to float

# Send some data
puts "Sending data: [1.0, 2.0, 3.14, 42.0]"
result = sender.send([1.0, 2.0, 3.14, 42.0])
puts "Send result: #{result}"

# Receive the data
puts "\nReceiving data..."
result = receiver.receive
puts "Received: #{result[:data]}"
puts "Remaining items: #{result[:remaining]}"
puts "Signal: #{result[:signal]}"

# Example 2: Integer data
puts "\n2. Integer Ring Buffer"
puts "Creating integer ring buffer with capacity 1000..."
int_sender, int_receiver = ShmemIpc.create_integer_ring(1000)

puts "Sending integers: [10, 20, 30, 42]"
result = int_sender.send([10, 20, 30, 42])
puts "Send result: #{result}"

result = int_receiver.receive
puts "Received: #{result[:data]}"

# Example 3: Byte data (strings)
puts "\n3. Byte Ring Buffer (strings)"
puts "Creating byte ring buffer with capacity 1000..."
byte_sender, byte_receiver = ShmemIpc.create_byte_ring(1000)

puts "Sending string: 'Hello, shared memory!'"
result = byte_sender.send("Hello, shared memory!")
puts "Send result: #{result}"

result = byte_receiver.receive
puts "Received: '#{result[:data]}'"

puts "\n" + "=" * 30
puts "All examples completed successfully!"