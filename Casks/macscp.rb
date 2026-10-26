cask "macscp" do
  version "0.2.9"
  sha256 "0f512fd51af5f1499f21d3663d38512a8919d27d46637930b39948cce747569e"

  url "https://github.com/profectus/macSCP/releases/download/v#{version}/MacSCP.zip"
  name "MacSCP"
  desc "Native dual-pane SFTP/SCP client for macOS built with Rust & GPUI"
  homepage "https://github.com/profectus/macSCP"

  depends_on macos: :monterey

  app "MacSCP.app"

  binary "#{appdir}/MacSCP.app/Contents/MacOS/macscp", target: "macscp"

  zap trash: [
    "~/.config/macscp",
  ]
end
