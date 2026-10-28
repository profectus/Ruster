cask "ruster" do
  version "0.3.8"
  sha256 "ce891ba127178893abcce824dbac1ed6b1d94d99c50ddc3ddaa2fdeabea6497c"

  url "https://github.com/profectus/Ruster/releases/download/v#{version}/Ruster.zip"
  name "Ruster"
  desc "Native dual-pane SFTP/SCP client for macOS built with Rust & GPUI"
  homepage "https://github.com/profectus/Ruster"

  depends_on macos: :monterey

  app "Ruster.app"

  binary "#{appdir}/Ruster.app/Contents/MacOS/ruster", target: "ruster"

  zap trash: [
    "~/.config/ruster",
  ]
end
