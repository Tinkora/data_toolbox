# frozen_string_literal: true

require "open3"
require "optparse"
require "set"

REQUIRED_FILES = %w[
  README.md
  README.zh-CN.md
  LICENSE
  CONTRIBUTING.md
  CONTRIBUTING.zh-CN.md
  SECURITY.md
  SECURITY.zh-CN.md
  SUPPORT.md
  SUPPORT.zh-CN.md
  CHANGELOG.md
].freeze

BILINGUAL_PAIRS = %w[
  README
  CONTRIBUTING
  SECURITY
  SUPPORT
].map { |name| ["#{name}.md", "#{name}.zh-CN.md"] }.freeze

CONFIG_EXTENSIONS = %w[.json .jsonc .lock .toml .yaml .yml].freeze
CONFIG_FILENAMES = %w[.gitignore].freeze
UTF8_BOM = "\xEF\xBB\xBF".b.freeze

options = { root: Dir.pwd }
OptionParser.new do |parser|
  parser.on("--root PATH") { |path| options[:root] = path }
end.parse!

root = File.expand_path(options[:root])
errors = []

stdout, stderr, status = Open3.capture3("git", "-C", root, "ls-files", "-z")
unless status.success?
  warn "Unable to list tracked files: #{stderr.strip}"
  exit 1
end

tracked_files = stdout.split("\0").reject(&:empty?).to_set

REQUIRED_FILES.each do |path|
  errors << "Missing required file: #{path}" unless tracked_files.include?(path) && File.file?(File.join(root, path))
end

BILINGUAL_PAIRS.each do |english, chinese|
  english_exists = tracked_files.include?(english) && File.file?(File.join(root, english))
  chinese_exists = tracked_files.include?(chinese) && File.file?(File.join(root, chinese))
  next if english_exists && chinese_exists

  errors << "Missing bilingual pair: #{english}" unless english_exists
  errors << "Missing bilingual pair: #{chinese}" unless chinese_exists
end

text_files = tracked_files.select do |path|
  extension = File.extname(path).downcase
  config_filename = File.basename(path).include?(".config.")
  extension == ".md" || extension == ".markdown" ||
    CONFIG_EXTENSIONS.include?(extension) || config_filename ||
    CONFIG_FILENAMES.include?(File.basename(path)) ||
    REQUIRED_FILES.include?(path)
end

text_files.sort.each do |path|
  absolute_path = File.join(root, path)
  next unless File.file?(absolute_path)

  # Read explicitly as UTF-8 and report each file so one bad file cannot hide others.
  content = File.read(absolute_path, encoding: Encoding::UTF_8)
  errors << "UTF-8 BOM is not allowed: #{path}" if content.b.start_with?(UTF8_BOM)
  errors << "Invalid UTF-8: #{path}" unless content.valid_encoding?
rescue SystemCallError => error
  errors << "Unable to read #{path}: #{error.message}"
end

if errors.empty?
  puts "Documentation checks passed (#{text_files.length} tracked text files scanned)."
  exit 0
end

errors.each { |error| warn error }
exit 1
