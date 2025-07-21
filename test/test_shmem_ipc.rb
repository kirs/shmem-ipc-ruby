# frozen_string_literal: true

require "test_helper"

class TestShmemIpc < Minitest::Test
  def test_that_it_has_a_version_number
    refute_nil ::ShmemIpc::VERSION
  end

  # Float ring buffer tests
  def test_create_float_ring_buffer
    sender, receiver = ShmemIpc.create_float_ring(1000)
    assert_instance_of ShmemIpc::FloatRingSender, sender
    assert_instance_of ShmemIpc::FloatRingReceiver, receiver
  end

  def test_float_send_and_receive_single_value
    sender, receiver = ShmemIpc.create_float_ring(1000)
    
    # Send a single value
    result = sender.send(42.5)
    assert_equal 999, result[:remaining] # 1000 - 1 = 999
    
    # Receive the value
    result = receiver.receive
    assert_equal [42.5], result[:data]
    assert_equal 0, result[:remaining]
  end

  def test_float_send_and_receive_array
    sender, receiver = ShmemIpc.create_float_ring(1000)
    
    data = [1.0, 2.0, 3.14, 42.0, -10.5]
    
    # Send array
    result = sender.send(data)
    assert_equal 995, result[:remaining] # 1000 - 5 = 995
    
    # Receive array
    result = receiver.receive
    assert_equal data, result[:data]
    assert_equal 0, result[:remaining]
  end

  def test_float_large_data_transfer
    sender, receiver = ShmemIpc.create_float_ring(10000)
    
    # Send a large array
    large_data = Array.new(5000) { |i| i.to_f / 100.0 }
    
    result = sender.send(large_data)
    assert_equal 5000, result[:remaining] # 10000 - 5000 = 5000
    
    result = receiver.receive
    assert_equal large_data, result[:data]
  end

  # Integer ring buffer tests
  def test_create_integer_ring_buffer
    sender, receiver = ShmemIpc.create_integer_ring(1000)
    assert_instance_of ShmemIpc::IntegerRingSender, sender
    assert_instance_of ShmemIpc::IntegerRingReceiver, receiver
  end

  def test_integer_send_and_receive_single_value
    sender, receiver = ShmemIpc.create_integer_ring(1000)
    
    # Send a single integer
    result = sender.send(42)
    assert_equal 999, result[:remaining]
    
    # Receive the value
    result = receiver.receive
    assert_equal [42], result[:data]
    assert_equal 0, result[:remaining]
  end

  def test_integer_send_and_receive_array
    sender, receiver = ShmemIpc.create_integer_ring(1000)
    
    data = [1, 2, 42, -10, 0]
    
    # Send array
    result = sender.send(data)
    assert_equal 995, result[:remaining]
    
    # Receive array
    result = receiver.receive
    assert_equal data, result[:data]
    assert_equal 0, result[:remaining]
  end

  def test_integer_large_numbers
    sender, receiver = ShmemIpc.create_integer_ring(1000)
    
    # Test with large integers
    data = [9223372036854775807, -9223372036854775808, 0]  # i64 max/min
    
    result = sender.send(data)
    result = receiver.receive
    assert_equal data, result[:data]
  end

  # Byte ring buffer tests
  def test_create_byte_ring_buffer
    sender, receiver = ShmemIpc.create_byte_ring(1000)
    assert_instance_of ShmemIpc::ByteRingSender, sender
    assert_instance_of ShmemIpc::ByteRingReceiver, receiver
  end

  def test_byte_send_and_receive_string
    sender, receiver = ShmemIpc.create_byte_ring(1000)
    
    data = "Hello, World!"
    
    # Send string
    result = sender.send(data)
    assert_equal 987, result[:remaining] # 1000 - 13 = 987
    
    # Receive string
    result = receiver.receive
    assert_equal data, result[:data]
    assert_equal 0, result[:remaining]
  end

  def test_byte_send_and_receive_binary_data
    sender, receiver = ShmemIpc.create_byte_ring(1000)
    
    # Binary data with various byte values
    data = "\x00\x01\x02\xFF\xFE\xFD"
    
    result = sender.send(data)
    result = receiver.receive
    assert_equal data, result[:data]
  end

  def test_byte_send_array_of_integers
    sender, receiver = ShmemIpc.create_byte_ring(1000)
    
    # Send array of byte values
    byte_array = [72, 101, 108, 108, 111]  # "Hello" in ASCII
    
    result = sender.send(byte_array)
    result = receiver.receive
    assert_equal "Hello", result[:data]
  end

  def test_byte_large_data_transfer
    sender, receiver = ShmemIpc.create_byte_ring(10000)
    
    # Send a large string
    large_data = "x" * 5000
    
    result = sender.send(large_data)
    assert_equal 5000, result[:remaining]
    
    result = receiver.receive
    assert_equal large_data, result[:data]
  end

  # General tests
  def test_empty_receive_all_types
    # Float
    sender, receiver = ShmemIpc.create_float_ring(1000)
    result = receiver.receive
    assert_equal [], result[:data]
    assert_equal 0, result[:remaining]

    # Integer
    sender, receiver = ShmemIpc.create_integer_ring(1000)
    result = receiver.receive
    assert_equal [], result[:data]
    assert_equal 0, result[:remaining]

    # Byte
    sender, receiver = ShmemIpc.create_byte_ring(1000)
    result = receiver.receive
    assert_equal "", result[:data]
    assert_equal 0, result[:remaining]
  end

  def test_multiple_send_receive_cycles_all_types
    # Float
    sender, receiver = ShmemIpc.create_float_ring(100)
    10.times do |i|
      data = [i.to_f, (i + 1).to_f, (i + 2).to_f]
      sender.send(data)
      result = receiver.receive
      assert_equal data, result[:data]
    end

    # Integer
    sender, receiver = ShmemIpc.create_integer_ring(100)
    10.times do |i|
      data = [i, i + 1, i + 2]
      sender.send(data)
      result = receiver.receive
      assert_equal data, result[:data]
    end

    # Byte
    sender, receiver = ShmemIpc.create_byte_ring(100)
    10.times do |i|
      data = "data#{i}"
      sender.send(data)
      result = receiver.receive
      assert_equal data, result[:data]
    end
  end

  def test_get_file_descriptors_all_types
    # Float
    receiver = ShmemIpc::FloatRingReceiver.new(1000)
    fds = receiver.get_file_descriptors
    assert_instance_of Array, fds
    assert_equal 3, fds.length
    fds.each { |fd| assert fd >= 0 }

    # Integer
    receiver = ShmemIpc::IntegerRingReceiver.new(1000)
    fds = receiver.get_file_descriptors
    assert_instance_of Array, fds
    assert_equal 3, fds.length
    fds.each { |fd| assert fd >= 0 }

    # Byte
    receiver = ShmemIpc::ByteRingReceiver.new(1000)
    fds = receiver.get_file_descriptors
    assert_instance_of Array, fds
    assert_equal 3, fds.length
    fds.each { |fd| assert fd >= 0 }
  end

  def test_error_handling_invalid_capacity
    # Test with invalid capacity (should raise an error)
    assert_raises(RuntimeError) do
      ShmemIpc::FloatRingSender.new(0)
    end
    
    assert_raises(RuntimeError) do
      ShmemIpc::IntegerRingSender.new(0)
    end
    
    assert_raises(RuntimeError) do
      ShmemIpc::ByteRingSender.new(0)
    end
  end

  # Backward compatibility tests
  def test_backward_compatibility
    # Default create_ring should work with floats
    sender, receiver = ShmemIpc.create_ring(1000)
    assert_instance_of ShmemIpc::FloatRingSender, sender
    assert_instance_of ShmemIpc::FloatRingReceiver, receiver
    
    # Legacy aliases should work
    assert_equal ShmemIpc::RingSender, ShmemIpc::FloatRingSender
    assert_equal ShmemIpc::RingReceiver, ShmemIpc::FloatRingReceiver
  end
end