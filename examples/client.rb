#!/usr/bin/env ruby
# frozen_string_literal: true

# Client example: connects to server using file descriptors
# Usage: ruby client.rb <memfd> <empty_signal> <full_signal>

require_relative "../lib/shmem_ipc"

if ARGV.length != 3
  puts "Usage: ruby client.rb <memfd> <empty_signal> <full_signal>"
  puts "Get these values from running server.rb first"
  exit 1
end

memfd = ARGV[0].to_i
empty_signal = ARGV[1].to_i
full_signal = ARGV[2].to_i

CAPACITY = 100_000

puts "Connecting to server with FDs: #{memfd}, #{empty_signal}, #{full_signal}"

begin
  sender = ShmemIpc::RingSender.open(CAPACITY, memfd, empty_signal, full_signal)
  puts "Connected successfully!"
  
  # Send data in batches
  batch_size = 1000
  total_sent = 0
  
  10.times do |batch|
    # Generate some data
    data = Array.new(batch_size) { |i| (batch * batch_size + i + 1).to_f / 100.0 }
    
    result = sender.send(data)
    total_sent += data.length
    
    puts "Sent batch #{batch + 1}: #{data.length} items, total sent: #{total_sent}"
    puts "  Remaining buffer space: #{result[:remaining]}"
    
    sleep(0.1) # Small delay between batches
  end
  
  puts "Client finished. Total items sent: #{total_sent}"
  
rescue => e
  puts "Error: #{e.message}"
  puts e.backtrace
  exit 1
end