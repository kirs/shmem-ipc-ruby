# frozen_string_literal: true

require_relative "lib/shmem_ipc/version"

Gem::Specification.new do |spec|
  spec.name = "shmem_ipc"
  spec.version = ShmemIpc::VERSION
  spec.authors = ["Kir Shatrov"]
  spec.email = ["kir@shatrov.io"]

  spec.summary = "High-performance shared memory IPC for Ruby"
  spec.description = "A Ruby binding for the shmem-ipc Rust crate, providing lock-free ring buffers for inter-process communication"
  spec.homepage = "https://github.com/kirs/shmem-ipc-ruby"
  spec.license = "Apache-2.0"
  spec.required_ruby_version = ">= 3.2.0"

  spec.metadata["homepage_uri"] = spec.homepage
  spec.metadata["source_code_uri"] = "https://github.com/kirs/shmem-ipc-ruby"
  spec.metadata["changelog_uri"] = "https://github.com/kirs/shmem-ipc-ruby/blob/main/CHANGELOG.md"

  # Specify which files should be added to the gem when it is released.
  spec.files = Dir.chdir(__dir__) do
    `git ls-files -z`.split("\x0").reject do |f|
      (f == __FILE__) || f.match(%r{\A(?:(?:bin|test|spec|features)/|\.(?:git|travis|circleci)|appveyor)})
    end
  end
  spec.bindir = "exe"
  spec.executables = spec.files.grep(%r{\Aexe/}) { |f| File.basename(f) }
  spec.require_paths = ["lib"]

  # Native extension
  spec.extensions = ["ext/shmem_ipc/extconf.rb"]

  # Runtime dependencies
  spec.add_dependency "rb_sys", "~> 0.9"

  # Development dependencies
  spec.add_development_dependency "rake", "~> 13.0"
  spec.add_development_dependency "rake-compiler", "~> 1.2"
  spec.add_development_dependency "minitest", "~> 5.0"
  spec.add_development_dependency "rubocop", "~> 1.21"
end