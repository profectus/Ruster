cask "macscp" do
  version "0.3.0"
  sha256 "d08fdb0b3290ea15117093fe7cc71eca5f4fd57cad3e67a83cd607145cf82714"

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
