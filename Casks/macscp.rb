cask "macscp" do
  version "0.2.2"
  sha256 "a3ee15c13afd7e72a192525e5e20a57ec480d0feae2846ae49af5dc88c695c36"

  url "https://github.com/profectus/macSCP/releases/download/v#{version}/MacSCP.zip"
  name "MacSCP"
  desc "Native dual-pane SFTP/SCP client for macOS built with Rust & GPUI"
  homepage "https://github.com/profectus/macSCP"

  auto_updates true
  depends_on macos: ">= :monterey"

  app "MacSCP.app"

  binary "#{appdir}/MacSCP.app/Contents/MacOS/macscp", target: "macscp"

  zap trash: [
    "~/.config/macscp",
  ]
end
