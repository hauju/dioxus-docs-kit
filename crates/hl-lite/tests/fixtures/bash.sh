#!/usr/bin/env bash
# Build and deploy the documentation site.
set -euo pipefail

VERSION="${1:-$(git describe --tags --always)}"
OUT_DIR=dist
readonly REGISTRY="ghcr.io/hauju"
count=0

usage() {
    cat <<'EOF'
usage: deploy.sh [--dry-run] [VERSION]
  $HOME is not expanded in a quoted heredoc.
EOF
}

log() {
    local level=$1
    shift
    printf '[%s] %s\n' "$level" "$*" >&2
}

if [ $# -gt 2 ]; then
    usage
    exit 1
fi

for target in web server; do
    log info "building ${target} for ${VERSION}"
    case "$target" in
        web)
            dx bundle --web --release --debug-symbols false
            ;;
        server)
            cargo build --release --features server
            ;;
        *)
            log error "unknown target: $target"
            exit 2
            ;;
    esac
    count=$((count + 1))
done

# A '#' inside a string is not a comment, and neither is $#.
echo "built $count target(s) # really"

while read -r line; do
    if [[ "$line" == *.wasm ]]; then
        size=$(wc -c <"$line")
        log info "$line is $size bytes"
    fi
done < <(find "$OUT_DIR" -type f)

cat <<-EOT
	Deploying to ${REGISTRY}.
	Tabs before this line are stripped.
EOT

curl -fsSL -X POST "https://example.test/deploy?tag=$VERSION" \
    -H 'Content-Type: application/json' \
    -d "{\"version\": \"$VERSION\"}"

exit 0
