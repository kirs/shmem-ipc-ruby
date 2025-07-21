#!/usr/bin/env ruby
# frozen_string_literal: true

# Server example: creates a receiver and prints file descriptors
# Run this first, then use the FDs in client.rb

require_relative "../lib/shmem_ipc"

CAPACITY = 100_000

puts "Creating receiver with capacity #{CAPACITY}..."
receiver = ShmemIpc::RingReceiver.new(CAPACITY)

# Get file descriptors
fds = receiver.get_file_descriptors
memfd, empty_signal, full_signal = fds

puts "Server created successfully!"
puts "File descriptors:"
puts "  memfd: #{memfd}"
puts "  empty_signal: #{empty_signal}"
puts "  full_signal: #{full_signal}"
puts
puts "Pass these FDs to the client process:"
puts "ruby client.rb #{memfd} #{empty_signal} #{full_signal}"
puts
puts "Waiting for data..."

# In a real implementation, you'd use these FDs with a process communication
# mechanism like Unix domain sockets, pipes, or D-Bus to pass them to another process.
# For this example, we'll just simulate receiving data.

sum = 0.0
loop do
  result = receiver.receive
  
  if result[:data].empty?
    sleep(0.01) # Wait a bit if no data
    next
  end
  
  batch_sum = result[:data].sum
  sum += batch_sum
  
  puts "Received #{result[:data].length} items, batch sum: #{batch_sum.round(4)}, total sum: #{sum.round(4)}"
  
  break if sum > 1000 # Stop after receiving enough data
end

puts "Server finished. Total sum: #{sum}"