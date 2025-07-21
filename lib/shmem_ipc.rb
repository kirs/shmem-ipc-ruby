# frozen_string_literal: true

require_relative "shmem_ipc/version"

begin
  # Try to load the extension
  ruby_version = /(\d+\.\d+)/.match(RUBY_VERSION)
  require_relative "shmem_ipc/#{ruby_version[1]}/shmem_ipc"
rescue LoadError
  require_relative "shmem_ipc/shmem_ipc"
end

module ShmemIpc
  class Error < StandardError; end

  # Float ring buffer classes
  class FloatRingSender
    def initialize(capacity)
      @sender = ShmemIpc::FloatSender.new(capacity)
    end

    def self.open(capacity, memfd, empty_signal, full_signal)
      sender = allocate
      sender.instance_variable_set(:@sender, ShmemIpc::FloatSender.open(capacity, memfd, empty_signal, full_signal))
      sender
    end

    def get_file_descriptors
      @sender.get_fds
    end

    def send(data)
      data = [data] unless data.is_a?(Array)
      @sender.send_data(data)
    end
  end

  class FloatRingReceiver
    def initialize(capacity)
      @receiver = ShmemIpc::FloatReceiver.new(capacity)
    end

    def self.open(capacity, memfd, empty_signal, full_signal)
      receiver = allocate
      receiver.instance_variable_set(:@receiver, ShmemIpc::FloatReceiver.open(capacity, memfd, empty_signal, full_signal))
      receiver
    end

    def get_file_descriptors
      @receiver.get_fds
    end

    def receive
      @receiver.receive_data
    end
  end

  # Integer ring buffer classes
  class IntegerRingSender
    def initialize(capacity)
      @sender = ShmemIpc::IntegerSender.new(capacity)
    end

    def self.open(capacity, memfd, empty_signal, full_signal)
      sender = allocate
      sender.instance_variable_set(:@sender, ShmemIpc::IntegerSender.open(capacity, memfd, empty_signal, full_signal))
      sender
    end

    def get_file_descriptors
      @sender.get_fds
    end

    def send(data)
      data = [data] unless data.is_a?(Array)
      @sender.send_data(data)
    end
  end

  class IntegerRingReceiver
    def initialize(capacity)
      @receiver = ShmemIpc::IntegerReceiver.new(capacity)
    end

    def self.open(capacity, memfd, empty_signal, full_signal)
      receiver = allocate
      receiver.instance_variable_set(:@receiver, ShmemIpc::IntegerReceiver.open(capacity, memfd, empty_signal, full_signal))
      receiver
    end

    def get_file_descriptors
      @receiver.get_fds
    end

    def receive
      @receiver.receive_data
    end
  end

  # Byte ring buffer classes
  class ByteRingSender
    def initialize(capacity)
      @sender = ShmemIpc::ByteSender.new(capacity)
    end

    def self.open(capacity, memfd, empty_signal, full_signal)
      sender = allocate
      sender.instance_variable_set(:@sender, ShmemIpc::ByteSender.open(capacity, memfd, empty_signal, full_signal))
      sender
    end

    def get_file_descriptors
      @sender.get_fds
    end

    def send(data)
      @sender.send_data(data)
    end
  end

  class ByteRingReceiver
    def initialize(capacity)
      @receiver = ShmemIpc::ByteReceiver.new(capacity)
    end

    def self.open(capacity, memfd, empty_signal, full_signal)
      receiver = allocate
      receiver.instance_variable_set(:@receiver, ShmemIpc::ByteReceiver.open(capacity, memfd, empty_signal, full_signal))
      receiver
    end

    def get_file_descriptors
      @receiver.get_fds
    end

    def receive
      @receiver.receive_data
    end
  end

  # Convenience methods for creating sender/receiver pairs
  def self.create_float_ring(capacity)
    receiver = FloatRingReceiver.new(capacity)
    fds = receiver.get_file_descriptors
    sender = FloatRingSender.open(capacity, *fds)
    [sender, receiver]
  end

  def self.create_integer_ring(capacity)
    receiver = IntegerRingReceiver.new(capacity)
    fds = receiver.get_file_descriptors
    sender = IntegerRingSender.open(capacity, *fds)
    [sender, receiver]
  end

  def self.create_byte_ring(capacity)
    receiver = ByteRingReceiver.new(capacity)
    fds = receiver.get_file_descriptors
    sender = ByteRingSender.open(capacity, *fds)
    [sender, receiver]
  end

  # Backward compatibility - defaults to float
  def self.create_ring(capacity)
    create_float_ring(capacity)
  end

  # Legacy aliases for backward compatibility
  RingSender = FloatRingSender
  RingReceiver = FloatRingReceiver
end