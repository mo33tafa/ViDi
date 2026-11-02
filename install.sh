#!/usr/bin/env sh
set -eu

repo="mo33tafa/vidi"
version="${VIDI_VERSION:-latest}"
install_dir="${VIDI_INSTALL_DIR:-}"

os=$(uname -s)
arch=$(uname -m)
[ "$os" = "Linux" ] || { printf '%s\n' 'vidi installer supports Linux only.' >&2; exit 1; }

case "$arch" in
  x86_64|amd64) asset_arch=x86_64 ;;
  aarch64|arm64) asset_arch=aarch64 ;;
  *) printf 'Unsupported Linux architecture: %s\n' "$arch" >&2; exit 1 ;;
esac

if [ -z "$install_dir" ]; then
  if [ "$(id -u)" -eq 0 ] && [ -d /usr/local/bin ] && [ -w /usr/local/bin ]; then
    install_dir=/usr/local/bin
  else
    install_dir="${HOME}/.local/bin"
  fi
fi

mkdir -p "$install_dir"
tmp_dir=$(mktemp -d)
trap 'rm -rf "$tmp_dir"' EXIT

if [ "$version" = latest ]; then
  url="https://github.com/${repo}/releases/latest/download/vidi-linux-${asset_arch}.tar.gz"
else
  url="https://github.com/${repo}/releases/download/${version}/vidi-linux-${asset_arch}.tar.gz"
fi

printf 'Downloading vidi for Linux (%s) ...\n' "$asset_arch"
curl --fail --location --silent --show-error "$url" -o "$tmp_dir/vidi.tar.gz"
tar -xzf "$tmp_dir/vidi.tar.gz" -C "$tmp_dir"
install -m 755 "$tmp_dir/vidi" "$install_dir/vidi"

case ":${PATH}:" in
  *":${install_dir}:"*) ;;
  *) printf 'Installed to %s/vidi\nAdd this directory to PATH if needed:\n  export PATH="%s:$PATH"\n' "$install_dir" "$install_dir" ;;
esac

"$install_dir/vidi" --version
