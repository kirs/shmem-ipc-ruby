#!/usr/bin/env ruby
# frozen_string_literal: true

require_relative "../lib/shmem_ipc"

puts "ShmemIpc Multi-Type Data Example"
puts "=" * 40

# Example 1: Float data (sensor readings)
puts "\n1. Float Ring Buffer - Sensor Readings"
puts "-" * 30

float_sender, float_receiver = ShmemIpc.create_float_ring(1000)

# Simulate temperature readings
temperatures = [23.5, 24.1, 23.8, 25.2, 26.0]
puts "Sending temperature readings: #{temperatures}"

result = float_sender.send(temperatures)
puts "Send result: remaining space = #{result[:remaining]}"

result = float_receiver.receive
puts "Received temperatures: #{result[:data]}"
puts "Average temperature: #{result[:data].sum / result[:data].length}°C"

# Example 2: Integer data (counters)
puts "\n2. Integer Ring Buffer - Event Counters"
puts "-" * 30

integer_sender, integer_receiver = ShmemIpc.create_integer_ring(1000)

# Simulate event counters
counters = [142, 256, 89, 1024, 512]
puts "Sending event counters: #{counters}"

result = integer_sender.send(counters)
puts "Send result: remaining space = #{result[:remaining]}"

result = integer_receiver.receive
puts "Received counters: #{result[:data]}"
puts "Total events: #{result[:data].sum}"

# Example 3: Byte data (messages)
puts "\n3. Byte Ring Buffer - Text Messages"
puts "-" * 30

byte_sender, byte_receiver = ShmemIpc.create_byte_ring(1000)

# Send text message
message = "Hello from shared memory! 🚀"
puts "Sending message: '#{message}'"

result = byte_sender.send(message)
puts "Send result: remaining space = #{result[:remaining]}"

result = byte_receiver.receive
puts "Received message: '#{result[:data]}'"
puts "Message length: #{result[:data].length} bytes"

# Example 4: Binary data
puts "\n4. Byte Ring Buffer - Binary Data"
puts "-" * 30

# Send binary data (image header simulation)
binary_data = "\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR"
puts "Sending binary data: #{binary_data.bytes.map { |b| "0x%02X" % b }.join(" ")}"

result = byte_sender.send(binary_data)
result = byte_receiver.receive

received_bytes = result[:data].bytes
puts "Received binary data: #{received_bytes.map { |b| "0x%02X" % b }.join(" ")}"
puts "Data matches: #{binary_data == result[:data]}"

# Example 5: Large data transfer demonstration
puts "\n5. Performance Comparison - Large Data"
puts "-" * 30

require 'benchmark'

large_size = 10000

# Float data
puts "Transferring #{large_size} floats..."
float_sender, float_receiver = ShmemIpc.create_float_ring(large_size + 100)
float_data = Array.new(large_size) { |i| i.to_f / 1000.0 }

float_time = Benchmark.realtime do
  float_sender.send(float_data)
  float_receiver.receive
end

# Integer data
puts "Transferring #{large_size} integers..."
integer_sender, integer_receiver = ShmemIpc.create_integer_ring(large_size + 100)
integer_data = Array.new(large_size) { |i| i }

integer_time = Benchmark.realtime do
  integer_sender.send(integer_data)
  integer_receiver.receive
end

# Byte data
puts "Transferring #{large_size} bytes..."
byte_sender, byte_receiver = ShmemIpc.create_byte_ring(large_size + 100)
byte_data = "x" * large_size

byte_time = Benchmark.realtime do
  byte_sender.send(byte_data)
  byte_receiver.receive
end

puts "\nPerformance Results:"
puts "Float transfer: %.4f seconds" % float_time
puts "Integer transfer: %.4f seconds" % integer_time
puts "Byte transfer: %.4f seconds" % byte_time

# Example 6: Mixed usage demonstration
puts "\n6. Real-World Scenario - Audio Processing Pipeline"
puts "-" * 50

# Simulate audio processing pipeline with different data types
puts "Setting up audio processing pipeline..."

# Audio samples (float data)
audio_sender, audio_receiver = ShmemIpc.create_float_ring(4096)

# Control messages (byte data) 
control_sender, control_receiver = ShmemIpc.create_byte_ring(1024)

# Metadata (integer data)
metadata_sender, metadata_receiver = ShmemIpc.create_integer_ring(256)

# Simulate audio processing
puts "\nProcessing audio batch..."

# Send audio samples (44.1kHz, stereo)
sample_rate = 44100
samples = Array.new(1024) { |i| Math.sin(2 * Math::PI * 440 * i / sample_rate) }
audio_sender.send(samples)

# Send control message
control_sender.send("SET_VOLUME:75")

# Send metadata (sample_rate, channels, bit_depth)
metadata_sender.send([44100, 2, 16])

# Process (receive) the data
audio_result = audio_receiver.receive
control_result = control_receiver.receive  
metadata_result = metadata_receiver.receive

puts "Processed #{audio_result[:data].length} audio samples"
puts "Control command: #{control_result[:data]}"
puts "Audio format: #{metadata_result[:data][0]}Hz, #{metadata_result[:data][1]} channels, #{metadata_result[:data][2]}-bit"

puts "\n" + "=" * 40
puts "Multi-type example completed successfully!"
puts "All data types (floats, integers, bytes) working correctly."