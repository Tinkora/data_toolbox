# frozen_string_literal: true

require "fileutils"
require "minitest/autorun"
require "open3"
require "rbconfig"
require "tmpdir"
require "yaml"

class CheckWorkflowContractsTest < Minitest::Test
  CHECKER = File.expand_path("check_workflow_contracts.rb", __dir__)
  COMMIT = "21145ce218263e3b30359bab0c748da4702f801b"

  def test_valid_tinkora_references_pass
    with_fixture do |root|
      result = run_checker(root)

      assert result[:status].success?, result[:output]
      assert_includes result[:output], "Reusable workflow contracts passed"
    end
  end

  def test_retired_organization_reference_fails
    with_fixture(owner: "retired-org") do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "must use Tinkora/.github"
    end
  end

  def test_floating_reference_fails
    with_fixture(reference: "main") do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "@#{COMMIT}"
    end
  end

  def test_floating_pages_reference_fails
    with_fixture(pages_reference: "main") do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "reusable-pages.yml@#{COMMIT}"
    end
  end

  def test_floating_release_reference_fails
    with_fixture(release_reference: "main") do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "reusable-release.yml@#{COMMIT}"
    end
  end

  def test_missing_required_job_fails
    with_fixture(include_wasm: false) do |root|
      result = run_checker(root)

      refute result[:status].success?
      assert_includes result[:output], "job wasm must use"
    end
  end

  private

  def with_fixture(
    owner: "Tinkora",
    reference: COMMIT,
    pages_reference: COMMIT,
    release_reference: COMMIT,
    include_wasm: true
  )
    Dir.mktmpdir("workflow-contracts-") do |root|
      quality_jobs = {
        "rust" => "#{owner}/.github/.github/workflows/reusable-rust-quality.yml@#{reference}"
      }
      if include_wasm
        quality_jobs["wasm"] =
          "#{owner}/.github/.github/workflows/reusable-wasm-quality.yml@#{reference}"
      end
      write_workflow(root, ".github/workflows/quality.yml", quality_jobs)
      write_workflow(
        root,
        ".github/workflows/supply-chain.yml",
        "audit" => "#{owner}/.github/.github/workflows/reusable-supply-chain.yml@#{reference}"
      )
      write_workflow(
        root,
        ".github/workflows/pages.yml",
        "deploy" => "#{owner}/.github/.github/workflows/reusable-pages.yml@#{pages_reference}"
      )
      write_workflow(
        root,
        ".github/workflows/release.yml",
        "prepare" => "#{owner}/.github/.github/workflows/reusable-release.yml@#{release_reference}"
      )
      yield root
    end
  end

  def write_workflow(root, relative_path, jobs)
    absolute_path = File.join(root, relative_path)
    FileUtils.mkdir_p(File.dirname(absolute_path))
    document = {
      "name" => "Fixture",
      "jobs" => jobs.transform_values { |uses| { "uses" => uses } }
    }
    File.write(absolute_path, YAML.dump(document), encoding: "UTF-8")
  end

  def run_checker(root)
    stdout, stderr, status = Open3.capture3(
      RbConfig.ruby,
      CHECKER,
      "--root",
      root
    )
    { output: stdout + stderr, status: status }
  end
end
