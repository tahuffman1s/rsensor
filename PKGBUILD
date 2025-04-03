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
}

check() {
  # Skip checks
  :
}

package() {
  # Install binary
  install -Dm755 "$startdir/target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
  
  # Install docs if they exist
  if [ -f "$startdir/README.md" ]; then
    install -Dm644 "$startdir/README.md" "$pkgdir/usr/share/doc/$pkgname/README.md"
  fi
  
  # Install license if it exists
  if [ -f "$startdir/LICENSE" ]; then
    install -Dm644 "$startdir/LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
  fi
}