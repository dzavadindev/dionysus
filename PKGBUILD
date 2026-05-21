# Maintainer: Dan Zavadin <daniil.zavadin@gmail.com>
pkgname=dionysus
pkgver=1.0.0
pkgrel=1
pkgdesc="Lightweight GTK4 application launcher for Wayland"
arch=('x86_64')
url="https://github.com/dzavadindev/dionysus"
license=('GPL3')

depends=('gtk4' 'hicolor-icon-theme')
makedepends=('cargo' 'rust')

# TODO: Local-first source for distribution-readiness testing.
# switch to a release tarball or git source when publishing.
#
# source=()
# sha256sums=('SKIP')

build() {
  cd "$startdir"
  cargo build --release --locked
}

check() {
  cd "$startdir"
  cargo test --release --locked
}

package() {
  cd "$startdir"

  install -Dm755 target/release/dionysus "$pkgdir/usr/bin/dionysus"

  # TODO: enable once LICENSE file exists.
  # install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"

  # TODO: enable once desktop assets are added.
  # install -Dm644 assets/dionysus.desktop "$pkgdir/usr/share/applications/dionysus.desktop"
  # install -Dm644 assets/dionysus.svg "$pkgdir/usr/share/icons/hicolor/scalable/apps/dionysus.svg"
}
