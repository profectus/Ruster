cask "ruster" do
  version "0.4.12"
  sha256 "b1a7462835316d16222b6bdbcc5dfaadbb3499567a0901cee712ff755f0c7a78"

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
