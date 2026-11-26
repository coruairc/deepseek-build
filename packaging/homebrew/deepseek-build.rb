# Pinned to the throwaway proof pre-release, `v0.1.0-proof1`: both `url`s and
# both `sha256`s are the two macOS tarballs exactly as the release publishes
# them. When `v0.1.0` is published, take the two new darwin hashes from that
# release's `SHA256SUMS`, bump `version`, both `url`s and both `sha256`s, then
# re-run `scripts/check-homebrew-formula.py` against the new sums before
# updating any tap. The formula installs the prebuilt, ad-hoc-signed tarball --
# there is no source build, no `head` and no `patch`.
class DeepseekBuild < Formula
  desc "DeepSeek-first terminal coding agent"
  homepage "https://github.com/coruairc/deepseek-build"
  version "0.1.0-proof1"
  license "Apache-2.0"

  livecheck do
    skip "Prerelease-only proof tag; repoint manually when v0.1.0 is published."
  end

  on_macos do
    on_arm do
      url "https://github.com/coruairc/deepseek-build/releases/download/v0.1.0-proof1/deepseek-build-0.1.0-proof1-aarch64-apple-darwin.tar.gz"
      sha256 "2b61dfab870b397b49ed810f53e073714582e01dbf5c6f9d166656367c51324b"
    end

    on_intel do
      url "https://github.com/coruairc/deepseek-build/releases/download/v0.1.0-proof1/deepseek-build-0.1.0-proof1-x86_64-apple-darwin.tar.gz"
      sha256 "ffe0f3c5f76be9ee6fa2dff1d53c34de6dce66d3553a094ee62524f58f57b3a7"
    end
  end

  def install
    bin.install "deepseek-build"
    bin.install_symlink "deepseek-build" => "dsb"
    doc.install "LICENSE", "NOTICE", "THIRD-PARTY-NOTICES", "README.txt"
  end

  def caveats
    <<~EOS
      This build is ad-hoc signed and not notarized: it carries no Apple
      Developer ID signature and has not been through Apple's notarization
      service. A tarball fetched by `brew` (or `curl`) carries no quarantine
      attribute, so no Gatekeeper prompt is expected. A copy downloaded
      through a browser may be quarantined; clear it with:
        xattr -d com.apple.quarantine /path/to/deepseek-build
    EOS
  end

  test do
    assert_match "deepseek-build #{version}", shell_output("#{bin}/deepseek-build --version")
    assert_match "deepseek-build #{version}", shell_output("#{bin}/dsb --version")
  end
end
