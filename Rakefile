# frozen_string_literal: true

require "bundler/gem_tasks"
require "rake/testtask"
require "rb_sys/extensiontask"

task build: :compile

RbSys::ExtensionTask.new("shmem_ipc") do |ext|
  ext.lib_dir = "lib/shmem_ipc"
end

Rake::TestTask.new(:test) do |t|
  t.libs << "test"
  t.libs << "lib"
  t.test_files = FileList["test/**/test_*.rb"]
end

require "rubocop/rake_task"

RuboCop::RakeTask.new

task default: %i[clobber compile test rubocop]