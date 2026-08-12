# frozen_string_literal: true

require "optparse"
require "yaml"

REUSABLE_WORKFLOW_COMMIT = "21145ce218263e3b30359bab0c748da4702f801b"
EXPECTED_CALLS = {
  ".github/workflows/quality.yml" => {
    "rust" => "Tinkora/.github/.github/workflows/reusable-rust-quality.yml@#{REUSABLE_WORKFLOW_COMMIT}",
    "wasm" => "Tinkora/.github/.github/workflows/reusable-wasm-quality.yml@#{REUSABLE_WORKFLOW_COMMIT}"
  },
  ".github/workflows/supply-chain.yml" => {
    "audit" => "Tinkora/.github/.github/workflows/reusable-supply-chain.yml@#{REUSABLE_WORKFLOW_COMMIT}"
  },
  ".github/workflows/pages.yml" => {
    "deploy" => "Tinkora/.github/.github/workflows/reusable-pages.yml@#{REUSABLE_WORKFLOW_COMMIT}"
  },
  ".github/workflows/release.yml" => {
    "prepare" => "Tinkora/.github/.github/workflows/reusable-release.yml@#{REUSABLE_WORKFLOW_COMMIT}"
  }
}.freeze

options = { root: Dir.pwd }
OptionParser.new do |parser|
  parser.on("--root PATH") { |path| options[:root] = path }
end.parse!

root = File.expand_path(options[:root])
errors = []

EXPECTED_CALLS.each do |relative_path, expected_jobs|
  workflow_path = File.join(root, relative_path)
  unless File.file?(workflow_path)
    errors << "Missing workflow: #{relative_path}"
    next
  end

  begin
    workflow = YAML.safe_load_file(workflow_path, aliases: false)
    jobs = workflow.fetch("jobs")
    expected_jobs.each do |job_name, expected_reference|
      actual_reference = jobs.dig(job_name, "uses")
      next if actual_reference == expected_reference

      errors << "#{relative_path} job #{job_name} must use #{expected_reference}"
    end
  rescue KeyError, Psych::Exception => error
    errors << "Invalid workflow #{relative_path}: #{error.message}"
  end
end

if errors.empty?
  puts "Reusable workflow contracts passed (commit #{REUSABLE_WORKFLOW_COMMIT})."
  exit 0
end

errors.each { |error| warn error }
exit 1
