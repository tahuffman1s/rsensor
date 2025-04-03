# Maintainer: Your Name <your.email@example.com>
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

# We're building from local sources, so no external files/URLs:
source=()
sha256sums=()

prepare() {
  # Copy all local files (Cargo.toml, src/, etc.) into $srcdir/rsensor
  mkdir -p "$srcdir/$pkgname"
  cp -r . "$srcdir/$pkgname"

  cd "$srcdir/$pkgname"
  # Fetch dependencies, using Cargo.lock if present
  cargo fetch --locked
}

build() {
  cd "$srcdir/$pkgname"
  # Build in release mode using the stable Rust toolchain
  cargo build --frozen --release
}

check() {
  cd "$srcdir/$pkgname"
  # Run tests in frozen mode (ensures Cargo.lock is used)
  cargo test --frozen
}

package() {
  cd "$srcdir/$pkgname"
  # Install the compiled binary to /usr/bin
  install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"

  # If you have a README.md
  install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"

  # If you have a LICENSE file
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE" || true
}
