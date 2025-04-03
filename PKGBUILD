# Maintainer: Your Name <you@example.com>

pkgname=rsensor
pkgver=0.1.0
pkgrel=1
pkgdesc="A Rust-based system monitoring tool inspired by psensor"
arch=('x86_64' 'i686' 'aarch64' 'armv7h')
url="https://github.com/tahuffman1s/rsensor"
license=('MIT')
depends=('gcc-libs')
makedepends=('rust' 'cargo' 'git')

build() {
  cd "${srcdir}/${pkgname}"
  cargo build --release
}

check() {
  cd "${srcdir}/${pkgname}"
  cargo test --release
}


package() {
  cd "${srcdir}/${pkgname}"
  # Install binary
  install -Dm755 "target/release/${pkgname}" "${pkgdir}/usr/bin/${pkgname}"
  # Install docs (if you have a README.md)
  install -Dm644 README.md "${pkgdir}/usr/share/doc/${pkgname}/README.md"

  # Install license if available
  install -Dm644 LICENSE "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE" || true
}
