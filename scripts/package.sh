#!/usr/bin/env bash
set -euo pipefail

# Release artifact packaging script for ultra-metis.
# Packages cross-compiled binaries into distributable archives with checksums.
#
# Usage: scripts/package.sh <version-tag>
#   e.g.: scripts/package.sh v0.2.0
#
# Expects binary artifacts in binaries-{target}/ directories (as produced by
# the GitHub Actions release build matrix) or in target/release/ for local use.

VERSION="${1:?Usage: scripts/package.sh <version-tag>}"

TARGETS=(
  aarch64-apple-darwin
  x86_64-apple-darwin
  x86_64-unknown-linux-gnu
  aarch64-unknown-linux-gnu
  x86_64-pc-windows-msvc
)

BINARIES=(ultra-metis ultra-metis-mcp)

DIST_DIR="dist"
rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}"

# Portable SHA256 checksum function
sha256() {
  if command -v sha256sum &>/dev/null; then
    sha256sum "$@"
  elif command -v shasum &>/dev/null; then
    shasum -a 256 "$@"
  else
    echo "ERROR: No SHA256 utility found (need sha256sum or shasum)" >&2
    exit 1
  fi
}

packaged=0

for target in "${TARGETS[@]}"; do
  # Determine where binaries live
  bin_dir="binaries-${target}"

  # If CI artifact directory doesn't exist, try target/release for local builds
  if [[ ! -d "${bin_dir}" ]]; then
    # For local builds, only package the current platform
    if [[ -d "target/release" ]]; then
      bin_dir="target/release"
      # Skip targets that don't match the current platform for local builds
      current_target=""
      case "$(uname -s)-$(uname -m)" in
        Darwin-arm64)  current_target="aarch64-apple-darwin" ;;
        Darwin-x86_64) current_target="x86_64-apple-darwin" ;;
        Linux-x86_64)  current_target="x86_64-unknown-linux-gnu" ;;
        Linux-aarch64) current_target="aarch64-unknown-linux-gnu" ;;
      esac
      if [[ "${target}" != "${current_target}" ]]; then
        continue
      fi
    else
      echo "WARN: No binaries found for ${target} (no binaries-${target}/ or target/release/), skipping"
      continue
    fi
  fi

  # Determine binary suffix
  exe_suffix=""
  if [[ "${target}" == *"windows"* ]]; then
    exe_suffix=".exe"
  fi

  # Verify all expected binaries exist
  missing=0
  for bin in "${BINARIES[@]}"; do
    if [[ ! -f "${bin_dir}/${bin}${exe_suffix}" ]]; then
      echo "ERROR: Missing binary ${bin_dir}/${bin}${exe_suffix} for target ${target}" >&2
      missing=1
    fi
  done
  if [[ "${missing}" -eq 1 ]]; then
    echo "ERROR: Skipping ${target} due to missing binaries" >&2
    continue
  fi

  # Create staging directory
  stage_name="ultra-metis-${VERSION}-${target}"
  stage_dir="${DIST_DIR}/${stage_name}"
  mkdir -p "${stage_dir}"

  # Copy binaries
  for bin in "${BINARIES[@]}"; do
    cp "${bin_dir}/${bin}${exe_suffix}" "${stage_dir}/"
  done

  # Set executable permissions on non-Windows binaries
  if [[ "${target}" != *"windows"* ]]; then
    for bin in "${BINARIES[@]}"; do
      chmod +x "${stage_dir}/${bin}"
    done
  fi

  # Copy LICENSE and README if they exist
  [[ -f "LICENSE" ]] && cp LICENSE "${stage_dir}/" || true
  [[ -f "README.md" ]] && cp README.md "${stage_dir}/" || true

  # Create archive
  if [[ "${target}" == *"windows"* ]]; then
    archive="${stage_name}.zip"
    (cd "${DIST_DIR}" && zip -r "${archive}" "${stage_name}/")
  else
    archive="${stage_name}.tar.gz"
    tar -czf "${DIST_DIR}/${archive}" -C "${DIST_DIR}" "${stage_name}"
  fi

  echo "Packaged: ${DIST_DIR}/${archive}"
  packaged=$((packaged + 1))

  # Clean up staging directory
  rm -rf "${stage_dir}"
done

if [[ "${packaged}" -eq 0 ]]; then
  echo "ERROR: No platforms were packaged. Ensure binaries are available." >&2
  exit 1
fi

# Generate SHA256 checksums for all archives
echo ""
echo "Generating SHA256 checksums..."
(
  cd "${DIST_DIR}"
  sha256 *.tar.gz *.zip 2>/dev/null > SHA256SUMS.txt || true
  # If no archives matched the globs, generate from what we have
  if [[ ! -s SHA256SUMS.txt ]]; then
    for f in *; do
      [[ "${f}" == "SHA256SUMS.txt" ]] && continue
      sha256 "${f}" >> SHA256SUMS.txt
    done
  fi
)

echo ""
echo "=== Release artifacts ==="
ls -lh "${DIST_DIR}/"
echo ""
echo "=== SHA256 checksums ==="
cat "${DIST_DIR}/SHA256SUMS.txt"
echo ""
echo "Done! ${packaged} platform(s) packaged in ${DIST_DIR}/"
