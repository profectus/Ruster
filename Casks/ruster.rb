cask "ruster" do
  version "0.3.8"
  sha256 :no_check

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
