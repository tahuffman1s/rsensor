# Maintainer: Travis Huffman <huffmantravis57@protonmail.com>
pkgname=rsensor
pkgver=0.1.0
pkgrel=1
pkgdesc="A Rust-based system monitoring tool inspired by psensor"
arch=('x86_64' 'i686' 'aarch64' 'armv7h')
url="https://github.com/tahuffman1s/rsensor"
license=('MIT')
depends=('gcc-libs')
makedepends=('cargo')
provides=('rsensor')
conflicts=('rsensor')
install="${pkgname}.install"

# Override default source handling completely
source=()
sha256sums=()
noextract=()

# Disable makepkg standard build environment
options=('!strip' '!debug' '!libtool' '!buildflags')

# Skip extraction phase completely 
pkgver() {
  echo "0.1.0"
}

build() {
  # Build in place without touching source
  cd "$startdir"
  RUSTFLAGS="" cargo build --release
  
  # Debug: Print working directory and build output location
  echo "Build directory: $(pwd)"
  echo "Binary should be at: $(pwd)/target/release/$pkgname"
  ls -la "$(pwd)/target/release/"
}

check() {
  # Skip checks
  :
}

package() {
  # Print diagnostic information
  echo "Package function working directory: $(pwd)"
  echo "Startdir: $startdir"
  
  # Use absolute path
  ABSOLUTE_PATH="$(readlink -f "$startdir/target/release/$pkgname")"
  echo "Absolute path to binary: $ABSOLUTE_PATH"
  
  if [ -f "$ABSOLUTE_PATH" ]; then
    echo "Binary exists at absolute path"
    install -Dm755 "$ABSOLUTE_PATH" "$pkgdir/usr/bin/$pkgname"
  elif [ -f "$startdir/target/release/$pkgname" ]; then
    echo "Binary exists at relative path"
    install -Dm755 "$startdir/target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
  else
    echo "ERROR: Binary not found at either path"
    echo "Contents of target/release directory:"
    ls -la "$startdir/target/release/"
    exit 1
  fi
  
  # Install docs if they exist
  if [ -f "$startdir/README.md" ]; then
    install -Dm644 "$startdir/README.md" "$pkgdir/usr/share/doc/$pkgname/README.md"
  fi
  
  # Install license if it exists
  if [ -f "$startdir/LICENSE" ]; then
    install -Dm644 "$startdir/LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
  fi
}
