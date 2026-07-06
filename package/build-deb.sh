#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"

package_name="qsp-agent"
version="$(sed -n 's/^version = "\(.*\)"/\1/p' "${repo_root}/qsp-agent/Cargo.toml" | head -n 1)"
architecture="$(dpkg --print-architecture)"
binary_path="${repo_root}/target/release/${package_name}"

if [[ -z "${version}" ]]; then
    echo "Failed to determine package version" >&2
    exit 1
fi

if [[ ! -x "${binary_path}" ]]; then
    echo "Missing release binary at ${binary_path}" >&2
    exit 1
fi

build_root="${repo_root}/target/package/${package_name}_${version}_${architecture}"
package_root="${build_root}/root"
debian_dir="${package_root}/DEBIAN"
output_dir="${repo_root}/dist"

join_dependencies() {
    awk 'BEGIN { first = 1 } { if (!first) printf ", "; printf "%s", $0; first = 0 } END { print "" }'
}

shared_library_packages() {
    local binary="$1"
    local library
    local package_line

    ldd "${binary}" \
        | awk '/=> \// { print $3 } /^[[:space:]]*\// { print $1 }' \
        | while read -r library; do
            if package_line="$(dpkg-query -S "${library}" 2>/dev/null | sed -n '1p')" \
                && [[ -n "${package_line}" ]]
            then
                printf '%s\n' "${package_line%%:*}"
            else
                printf 'warning: no package found for shared library %s\n' "${library}" >&2
            fi
        done
}

rm -rf "${build_root}"
mkdir -p \
    "${debian_dir}" \
    "${package_root}/usr/bin" \
    "${package_root}/etc/qsp-agent" \
    "${package_root}/lib/systemd/system" \
    "${package_root}/usr/share/man/man1" \
    "${package_root}/usr/share/doc/${package_name}"

install -m 0755 "${binary_path}" "${package_root}/usr/bin/${package_name}"
install -m 0644 "${repo_root}/package/config.toml" "${package_root}/etc/qsp-agent/config.toml"
install -m 0644 "${repo_root}/package/systemd/qsp-agent.service" \
    "${package_root}/lib/systemd/system/qsp-agent.service"
install -m 0644 "${repo_root}/package/man/qsp-agent.1" \
    "${package_root}/usr/share/man/man1/qsp-agent.1"
gzip -n -9 "${package_root}/usr/share/man/man1/qsp-agent.1"
install -m 0644 "${repo_root}/Readme.md" \
    "${package_root}/usr/share/doc/${package_name}/README.md"
install -m 0644 "${repo_root}/COPYING" \
    "${package_root}/usr/share/doc/${package_name}/copyright"
install -m 0644 "${repo_root}/package/README.Debian" \
    "${package_root}/usr/share/doc/${package_name}/README.Debian"
install -m 0755 "${repo_root}/package/debian/postinst" "${debian_dir}/postinst"
install -m 0755 "${repo_root}/package/debian/prerm" "${debian_dir}/prerm"
install -m 0755 "${repo_root}/package/debian/postrm" "${debian_dir}/postrm"

printf '/etc/qsp-agent/config.toml\n' > "${debian_dir}/conffiles"

dependencies="$(
    {
        printf 'adduser\n'
        shared_library_packages "${package_root}/usr/bin/${package_name}"
    } | sort -u | join_dependencies
)"

if [[ -z "${dependencies}" ]]; then
    echo "Failed to determine shared library dependencies" >&2
    exit 1
fi

sed \
    -e "s/@PACKAGE_NAME@/${package_name}/g" \
    -e "s/@VERSION@/${version}/g" \
    -e "s/@ARCHITECTURE@/${architecture}/g" \
    -e "s/@DEPENDENCIES@/${dependencies}/g" \
    "${repo_root}/package/debian/control.in" > "${debian_dir}/control"

(
    cd "${package_root}"
    find . -path './DEBIAN' -prune -o -type f -print0 \
        | xargs -0 md5sum > "${debian_dir}/md5sums"
)

mkdir -p "${output_dir}"
dpkg-deb --build --root-owner-group "${package_root}" \
    "${output_dir}/${package_name}_${version}_${architecture}.deb"
