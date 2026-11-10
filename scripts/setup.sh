#!/usr/bin/env bash
# Installs what a tessoa plugin project needs on this computer, and only what
# is missing: the tessoa-plugin command-line tool, and Node.js (--lang ts) or
# Rust with the wasm32-wasip2 target (--lang rust). For macOS (Apple silicon
# and Intel) and Linux on x86-64.
#
#   scripts/setup.sh --lang ts [--mirror auto|cn|official] [--no-modify-path]
#
# Every file it downloads must match the sha256 in scripts/cli.lock, whichever
# server it came from; anything else is refused.
#
# --mirror cn takes Node.js, Rust and their packages from mirrors in mainland
# China (npmmirror, rsproxy), --mirror official never does, and the default,
# auto, downloads the start of the same file from both and uses whichever was
# faster. With the mirrors, npm's registry and cargo's crates source are
# pointed at them too, unless they are already set to something else.
#
# --no-modify-path leaves the shell start-up files alone; the tools are then
# reachable by the paths printed at the end.
#
# A shell that was open before the script ran does not read the start-up files
# again. When the tools are not on its PATH, the script ends with a line that
# adds them to it.
#
# TESSOA_PLUGIN_HOME changes where tessoa-plugin and Node.js go (default
# ~/.tessoa-plugin); tessoa-plugin looks there too. TESSOA_PLUGIN_DOWNLOAD
# changes where tessoa-plugin is downloaded from; its sha256 still has to
# match.
set -euo pipefail

# The caller's PATH, before this script adds to its own.
start_path="$PATH"
lang=""
mirror="auto"
modify_path=1
while [ $# -gt 0 ]; do
    case "$1" in
    --lang) lang="${2:-}"; shift 2 ;;
    --mirror) mirror="${2:-}"; shift 2 ;;
    --no-modify-path) modify_path=0; shift ;;
    *) echo "usage: $0 [--lang ts|rust] [--mirror auto|cn|official] [--no-modify-path]" >&2; exit 2 ;;
    esac
done
case "$lang" in "" | ts | rust) ;; *) echo "setup: --lang is ts or rust" >&2; exit 2 ;; esac
case "$mirror" in auto | cn | official) ;; *) echo "setup: --mirror is auto, cn or official" >&2; exit 2 ;; esac

say() { echo "setup: $*"; }
die() { echo "setup: $*" >&2; exit 1; }

here="$(cd "$(dirname "$0")" && pwd)"
lock="$here/cli.lock"
root="${TESSOA_PLUGIN_HOME:-$HOME/.tessoa-plugin}"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
node_major=22

case "$(uname -s)/$(uname -m)" in
Darwin/arm64) target="aarch64-apple-darwin"; node_platform="darwin-arm64" ;;
Darwin/x86_64) target="x86_64-apple-darwin"; node_platform="darwin-x64" ;;
Linux/x86_64) target="x86_64-unknown-linux-gnu"; node_platform="linux-x64" ;;
*) die "tessoa-plugin is built for macOS (Apple silicon and Intel) and Linux on x86-64; this is $(uname -s) $(uname -m)" ;;
esac

# ---------------------------------------------------------------- cli.lock

pinned_version() { awk -v t="$1" '$1 == "version" && $2 == t { print $3 }' "$lock"; }
pinned_hash() { awk -v f="$1" '$1 == "file" && $2 == f { print tolower($3) }' "$lock"; }

# ---------------------------------------------------------------- helpers

sha256_of() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | cut -d' ' -f1
    else
        shasum -a 256 "$1" | cut -d' ' -f1
    fi
}

# get URL NAME: download a file named in cli.lock and check its sha256 before
# anything uses it. Prints the path of the file.
get() {
    local url="$1" name="$2" want dest actual
    want="$(pinned_hash "$name")"
    [ -n "$want" ] || die "scripts/cli.lock has no sha256 for $name, so it is not downloaded"
    dest="$work/$name"
    say "downloading $url" >&2
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL --retry 2 -o "$dest" "$url" || die "cannot download $url"
    else
        wget -q -O "$dest" "$url" || die "cannot download $url"
    fi
    actual="$(sha256_of "$dest")"
    if [ "$actual" != "$want" ]; then
        rm -f "$dest"
        die "the sha256 of $name is $actual but scripts/cli.lock says $want; refused"
    fi
    echo "$dest"
}

unzip_to() {
    # $1 = zip, $2 = destination
    mkdir -p "$2"
    if command -v unzip >/dev/null 2>&1; then
        unzip -q -o "$1" -d "$2"
    elif [ "$(uname -s)" = Darwin ]; then
        ditto -x -k "$1" "$2"
    elif command -v python3 >/dev/null 2>&1; then
        python3 -m zipfile -e "$1" "$2"
    else
        die "need unzip or python3 to unpack $1"
    fi
}

# version_of TEXT: the first x.y.z in TEXT.
version_of() { printf '%s\n' "$1" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -n 1 || true; }
# at_least HAVE WANT: true when version HAVE >= WANT.
at_least() { [ "$(printf '%s\n%s\n' "$2" "$1" | sort -t. -k1,1n -k2,2n -k3,3n | head -n 1)" = "$2" ]; }
first_line() { "$@" 2>/dev/null | head -n 1 || true; }

# rate URL: bytes per second over the first MiB of URL, reading for at most
# four seconds; 0 when nothing arrives.
rate() {
    local r="" t0 t1
    if command -v curl >/dev/null 2>&1; then
        # -L: both mirrors answer with a redirect to their CDN.
        r="$(curl -sL -o /dev/null -r 0-1048575 --max-time 4 -w '%{speed_download}' "$1" 2>/dev/null || true)"
    elif command -v wget >/dev/null 2>&1 && command -v timeout >/dev/null 2>&1; then
        # Linux without curl; GNU date has %N.
        t0="$(date +%s.%N)"
        timeout 4 wget -q -O "$work/rate" --header='Range: bytes=0-1048575' "$1" 2>/dev/null || true
        t1="$(date +%s.%N)"
        r="$(awk -v s="$(wc -c <"$work/rate" 2>/dev/null || echo 0)" -v a="$t0" -v b="$t1" \
            'BEGIN { d = b - a; if (d <= 0) d = 0.001; printf "%.0f", s / d }')"
        rm -f "$work/rate"
    fi
    echo "${r:-0}"
}
kb() { awk -v r="$1" 'BEGIN { printf "%.0f", r / 1024 }'; }

# mirrored OFFICIAL MIRROR: true when the downloads come from the mirrors. The
# first call decides. With --mirror auto it fetches the start of the file this
# run is about to download from the official server and from the mirror (the
# two URLs) and takes the faster: how soon a server answers says little, since
# from mainland China the official servers answer a small request within a
# second or two and then send a few hundred KB a second or less.
use_mirror=""
mirrored() {
    local o m which
    if [ -z "$use_mirror" ]; then
        case "$mirror" in
        cn) use_mirror=1; say "using the mirrors (npmmirror, rsproxy)" ;;
        official) use_mirror=0; say "using the official servers" ;;
        *)
            o="$(rate "$1")"
            m="$(rate "$2")"
            if awk -v o="$o" -v m="$m" 'BEGIN { exit !(m > o) }'; then
                use_mirror=1
                which="the mirrors (npmmirror, rsproxy)"
            else
                use_mirror=0
                which="the official servers"
            fi
            say "a test download ran at $(kb "$o") KB/s from the official servers and $(kb "$m") KB/s from the mirrors; using $which"
            ;;
        esac
    fi
    [ "$use_mirror" = 1 ]
}

summary=""
report() { summary="$summary$(printf '%-14s %-10s %s' "$1" "$2" "$3")"$'\n'; }
path_dirs=()

# Folders of the tools the project uses that the calling shell does not find by
# name. npm in particular starts node by name, so calling npm by its full path
# is not enough.
here_dirs=()
want_here() {
    local d
    case ":$start_path:" in *":$1:"*) return 0 ;; esac
    for d in ${here_dirs[@]+"${here_dirs[@]}"}; do [ "$d" = "$1" ] && return 0; done
    here_dirs+=("$1")
}

# ---------------------------------------------------------------- tessoa-plugin

want="$(pinned_version tessoa-plugin)"
tool_dir="$root/bin"
tool_exe="$tool_dir/tessoa-plugin"
found=""
for candidate in "$tool_exe" "$(command -v tessoa-plugin 2>/dev/null || true)"; do
    [ -n "$candidate" ] && [ -x "$candidate" ] || continue
    have="$(version_of "$(first_line "$candidate" --version)")"
    if [ -n "$have" ] && at_least "$have" "$want"; then
        found="$candidate"
        break
    fi
done
if [ -n "$found" ]; then
    say "tessoa-plugin $have found at $found"
    report tessoa-plugin "$have" "$found"
else
    name="tessoa-plugin-$want-$target.zip"
    [ -n "$(pinned_hash "$name")" ] ||
        die "this template does not pin a tessoa-plugin build for $target yet (scripts/cli.lock has no $name)"
    base="${TESSOA_PLUGIN_DOWNLOAD:-https://download.tessoa.com/plugins/}"
    case "$base" in */) ;; *) base="$base/" ;; esac
    zip="$(get "$base$name" "$name")"
    unzip_to "$zip" "$work/tessoa-plugin"
    mkdir -p "$tool_dir"
    cp -f "$work/tessoa-plugin/tessoa-plugin" "$tool_exe"
    chmod 755 "$tool_exe"
    [ -f "$work/tessoa-plugin/THIRD-PARTY-NOTICES.html" ] && cp -f "$work/tessoa-plugin/THIRD-PARTY-NOTICES.html" "$tool_dir/"
    say "installed tessoa-plugin $want"
    report tessoa-plugin "$want" "$tool_exe"
    path_dirs+=("$tool_dir")
fi

# ---------------------------------------------------------------- Node.js

if [ "$lang" = ts ]; then
    node_want="$(pinned_version node)"
    node_name="node-v$node_want-$node_platform.tar.gz"
    node_official="https://nodejs.org/dist/v$node_want/$node_name"
    node_mirror="https://npmmirror.com/mirrors/node/v$node_want/$node_name"
    our_node="$root/node/bin/node"
    system_node="$(command -v node 2>/dev/null || true)"
    system_version=""
    [ -n "$system_node" ] && system_version="$(version_of "$(first_line "$system_node" --version)")"
    our_version=""
    [ -x "$our_node" ] && our_version="$(version_of "$(first_line "$our_node" --version)")"

    if [ -n "$system_version" ] && [ "${system_version%%.*}" -ge "$node_major" ]; then
        node="$system_node"
        say "Node.js $system_version found at $node"
        report node "$system_version" "$node"
    elif [ -n "$our_version" ] && [ "${our_version%%.*}" -ge "$node_major" ]; then
        node="$our_node"
        say "Node.js $our_version found at $node"
        report node "$our_version" "$node"
    else
        [ -n "$system_version" ] &&
            say "Node.js $system_version at $system_node is older than $node_major; installing $node_want for tessoa-plugin, which prefers it"
        if mirrored "$node_official" "$node_mirror"; then url="$node_mirror"; else url="$node_official"; fi
        archive="$(get "$url" "$node_name")"
        mkdir -p "$work/node" "$root"
        tar -xzf "$archive" -C "$work/node"
        rm -rf "$root/node"
        mv "$work/node/node-v$node_want-$node_platform" "$root/node"
        node="$our_node"
        say "installed Node.js $node_want"
        report node "$node_want" "$node"
        # Only when there is no other Node.js: an older one stays the user's
        # default, and tessoa-plugin finds this one by itself.
        [ -z "$system_node" ] && path_dirs+=("$root/node/bin")
    fi

    npm="$(dirname "$node")/npm"
    [ -x "$npm" ] || npm="$(command -v npm 2>/dev/null || true)"
    [ -n "$npm" ] || die "npm was not found next to $node"
    # npm is a script that runs node from the PATH.
    export PATH="$(dirname "$node"):$PATH"
    report npm "$(first_line "$npm" --version)" "$npm"
    if mirrored "$node_official" "$node_mirror"; then
        registry="$(first_line "$npm" config get registry)"
        if [ "$registry" = "https://registry.npmjs.org/" ]; then
            "$npm" config set registry https://registry.npmmirror.com/ --location=user >/dev/null
            say "npm now uses https://registry.npmmirror.com/ (undo: npm config delete registry --location=user)"
        elif [ -n "$registry" ]; then
            say "npm already uses $registry; left as it is"
        fi
    fi
fi

# ---------------------------------------------------------------- Rust

if [ "$lang" = rust ]; then
    min_rust="1.88.0"
    sdk_cargo="$here/../sdk/rust/Cargo.toml"
    if [ -f "$sdk_cargo" ]; then
        v="$(sed -n 's/^rust-version *= *"\([0-9.]*\)".*/\1/p' "$sdk_cargo" | head -n 1)"
        case "$v" in *.*.*) min_rust="$v" ;; *.*) min_rust="$v.0" ;; esac
    fi
    cargo_home="${CARGO_HOME:-$HOME/.cargo}"
    cargo="$(command -v cargo 2>/dev/null || true)"
    [ -z "$cargo" ] && [ -x "$cargo_home/bin/cargo" ] && cargo="$cargo_home/bin/cargo"
    rustup="$(command -v rustup 2>/dev/null || true)"
    [ -z "$rustup" ] && [ -x "$cargo_home/bin/rustup" ] && rustup="$cargo_home/bin/rustup"
    rustup_v="$(pinned_version rustup)"
    init_official="https://static.rust-lang.org/rustup/archive/$rustup_v/$target/rustup-init"
    init_mirror="https://rsproxy.cn/rustup/archive/$rustup_v/$target/rustup-init"
    if mirrored "$init_official" "$init_mirror"; then
        export RUSTUP_DIST_SERVER=https://rsproxy.cn
        export RUSTUP_UPDATE_ROOT=https://rsproxy.cn/rustup
    fi

    if [ -z "$cargo" ]; then
        name="rustup-init-$rustup_v-$target"
        if mirrored "$init_official" "$init_mirror"; then url="$init_mirror"; else url="$init_official"; fi
        init="$(get "$url" "$name")"
        chmod 755 "$init"
        args=(-y --profile minimal --default-toolchain stable --target wasm32-wasip2)
        [ "$modify_path" = 0 ] && args+=(--no-modify-path)
        say "installing Rust (stable) with rustup $rustup_v"
        "$init" "${args[@]}" || die "rustup-init failed"
        cargo="$cargo_home/bin/cargo"
        rustup="$cargo_home/bin/rustup"
    else
        rustc="$(dirname "$cargo")/rustc"
        [ -x "$rustc" ] || rustc="$(command -v rustc 2>/dev/null || true)"
        have="$(version_of "$(first_line "$rustc" --version)")"
        if [ -z "$have" ] || ! at_least "$have" "$min_rust"; then
            [ -n "$rustup" ] || die "Rust $have at $cargo is older than $min_rust and there is no rustup to update it"
            say "Rust $have is older than $min_rust; updating the stable toolchain"
            "$rustup" update stable || die "rustup update stable failed"
        fi
        if [ -n "$rustup" ]; then
            if ! "$rustup" target list --installed | grep -qx wasm32-wasip2; then
                say "adding the wasm32-wasip2 target"
                "$rustup" target add wasm32-wasip2 || die "rustup target add wasm32-wasip2 failed"
            fi
        else
            say "no rustup found; make sure the wasm32-wasip2 target is installed"
        fi
    fi
    report cargo "$(version_of "$(first_line "$cargo" --version)")" "$cargo"
    report rustc "$(version_of "$(first_line "$(dirname "$cargo")/rustc" --version)")" "$(dirname "$cargo")/rustc"

    # Procedural macros are built for this computer, which takes a C linker.
    if ! command -v cc >/dev/null 2>&1; then
        if [ "$(uname -s)" = Darwin ]; then
            say "no C compiler found; Rust needs one to link. Install it with: xcode-select --install"
        else
            say "no C compiler found; Rust needs one to link. Install gcc (Debian, Ubuntu: sudo apt install build-essential)"
        fi
    fi

    if mirrored "$init_official" "$init_mirror"; then
        config="$cargo_home/config.toml"
        if [ -f "$config" ] && grep -q '^\[source\.crates-io\]' "$config"; then
            say "cargo's crates source is already set in $config; left as it is"
        else
            mkdir -p "$cargo_home"
            printf '\n[source.crates-io]\nreplace-with = "rsproxy-sparse"\n\n[source.rsproxy-sparse]\nregistry = "sparse+https://rsproxy.cn/index/"\n' >>"$config"
            say "cargo now takes crates from rsproxy (undo: remove the [source.*] lines from $config)"
        fi
    fi
fi

# ---------------------------------------------------------------- PATH

changed=0
if [ "$modify_path" = 1 ] && [ "${#path_dirs[@]}" -gt 0 ]; then
    env_file="$root/env"
    {
        echo "# Written by the tessoa plugin setup script."
        for d in "${path_dirs[@]}"; do
            echo "case \":\$PATH:\" in *\":$d:\"*) ;; *) export PATH=\"$d:\$PATH\" ;; esac"
        done
    } >"$env_file"
    line=". \"$env_file\""
    for rc in "$HOME/.profile" "$HOME/.bash_profile" "$HOME/.bash_login" "$HOME/.bashrc" "$HOME/.zshrc"; do
        # A login bash reads only the first of .bash_profile, .bash_login and
        # .profile that exists, and a new terminal on macOS starts a login
        # shell: the first two get the line too, when they exist.
        case "$rc" in */.bash_profile | */.bash_login) [ -f "$rc" ] || continue ;; esac
        # .zshrc only where zsh is the shell or the file exists already.
        if [ "$rc" = "$HOME/.zshrc" ] && [ ! -f "$rc" ] && [ "$(basename "${SHELL:-}")" != zsh ]; then
            continue
        fi
        if ! grep -qxF "$line" "$rc" 2>/dev/null; then
            printf '\n%s\n' "$line" >>"$rc"
            changed=1
            say "added the tools to PATH in $rc"
        fi
    done
fi

want_here "$(dirname "${found:-$tool_exe}")"
# Our Node.js only where there is no other: an older one stays the default.
if [ "$lang" = ts ] && [ -z "$system_node" ]; then want_here "$(dirname "$node")"; fi
if [ "$lang" = rust ]; then want_here "$(dirname "$cargo")"; fi

echo
printf '%s' "$summary"
echo
if [ "$modify_path" = 0 ] && [ "${#path_dirs[@]}" -gt 0 ]; then
    echo "PATH was left alone (--no-modify-path)."
elif [ "$changed" = 1 ]; then
    echo "New shells find these tools by name."
fi
echo "Next: tessoa-plugin init (see sdk/AGENTS.md)."
if [ "${#here_dirs[@]}" -gt 0 ]; then
    joined="$(IFS=:; echo "${here_dirs[*]}")"
    echo
    echo "This shell does not find these tools by name yet. Run this line in it first"
    echo "(where every command starts a new shell, start each command with it):"
    echo "  export PATH=\"$joined:\$PATH\""
fi
