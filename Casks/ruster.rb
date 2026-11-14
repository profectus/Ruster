cask "ruster" do
  version "0.4.9"
  sha256 "0f2f7b2db1b056770b660b3c32495a6139ca23711ec07920186cef51902be589"

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
