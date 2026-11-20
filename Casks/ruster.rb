cask "ruster" do
  version "0.4.14"
  sha256 "fb54178cc9acfe0a90ea02dc9762aab8e5754ef5c952a48df525781b96ea60c4"

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
