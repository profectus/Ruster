cask "macscp" do
  version "0.1.0"
  sha256 "c2244ac307ad0a445bd0e5921d6ddb6fdbcfd33e334ca5f0a437a96b9c1f676b"

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
