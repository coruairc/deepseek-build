# Resolve one platform branch of the formula the way Homebrew does.
#
#   brew ruby docs/evidence/04-distribution-proof/homebrew-formula/resolve-branch.rb arm
#   brew ruby docs/evidence/04-distribution-proof/homebrew-formula/resolve-branch.rb intel
#
# `Homebrew::SimulateSystem` is Homebrew's own mechanism for evaluating a
# formula for another OS/arch. The formula class is defined inside the block,
# so the `on_macos` / `on_arm` / `on_intel` guards select the branch under test.
require "formula"
require "simulate_system"

arch = ARGV.fetch(0, "arm").to_sym
Homebrew::SimulateSystem.with(os: :macos, arch: arch) do
  formula = Formulary.factory(ARGV.fetch(1, "local/deepseek-test/deepseek-build"))
  puts "arch=#{arch} os=#{Homebrew::SimulateSystem.os}"
  puts "version=#{formula.version}"
  puts "url=#{formula.stable.url}"
  puts "sha256=#{formula.stable.checksum.hexdigest}"
end
