cask "macscp" do
  version "0.2.0"
  sha256 "d9c4a40110f8f854679267c2304b71ac9fa8663339c7402ce076fa051a1734b5"

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
