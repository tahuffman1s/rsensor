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

# Define source as local directory (using current directory)
source=("$pkgname::git+file://${startdir}")
sha256sums=('SKIP')

build() {
  cd "$srcdir/$pkgname"
  RUSTFLAGS="" cargo build --release
}

package() {
  cd "$srcdir/$pkgname"
  
  # Install binary
  install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
  
  # Install docs if they exist
  if [ -f "README.md" ]; then
    install -Dm644 "README.md" "$pkgdir/usr/share/doc/$pkgname/README.md"
  fi
  
  # Install license if it exists
  if [ -f "LICENSE" ]; then
    install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
  fi
  
  # Install the .install file
  install -Dm644 "${pkgname}.install" "$pkgdir/usr/share/${pkgname}/${pkgname}.install"
}