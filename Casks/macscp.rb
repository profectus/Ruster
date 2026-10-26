cask "macscp" do
  version "0.2.2"
  sha256 "88800c297621ffea41274040be48700f6fef5b386bf9b77aa26ebb1b68ee832e"

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
