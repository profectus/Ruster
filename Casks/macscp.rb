cask "macscp" do
  version "0.2.0"
  sha256 "7bd2f1292185ab9e38751ee33886eb04cfbfbe0fd8115e1be245bfed8bcbd8f1"

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
