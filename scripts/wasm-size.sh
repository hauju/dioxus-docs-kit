#!/usr/bin/env bash
#
# Report the on-the-wire size of a `dx bundle --web --release` output.
#
# Prints raw and brotli sizes for the wasm, the JS glue and the stylesheets.
# Appends a markdown table to $GITHUB_STEP_SUMMARY when that variable is set,
# and exits non-zero when the raw wasm exceeds $WASM_SIZE_LIMIT_BYTES (unset:
# report only). Brotli sizes come from the `.br` sidecars the server ships; if
# a sidecar is missing the file is compressed on the fly just to get a number.

set -euo pipefail

public_dir="${PUBLIC_DIR:-target/dx/dioxus-docs-kit-example/release/web/public}"
limit="${WASM_SIZE_LIMIT_BYTES:-}"

if [ ! -d "$public_dir" ]; then
    echo "no bundle at $public_dir - run 'dx bundle --web --release --debug-symbols false' first" >&2
    exit 1
fi

br_size() {
    if [ -f "$1.br" ]; then
        wc -c <"$1.br"
    elif command -v brotli >/dev/null 2>&1; then
        brotli -q 11 -c "$1" | wc -c
    else
        echo 0
    fi
}

human() {
    awk -v b="$1" 'BEGIN {
        if (b >= 1048576) printf "%.2f MB", b / 1048576
        else if (b >= 1024) printf "%.1f KB", b / 1024
        else printf "%d B", b
    }'
}

wasm_raw=0
table=""
for ext in wasm js css; do
    count=0
    raw=0
    br=0
    while IFS= read -r file; do
        count=$((count + 1))
        size=$(wc -c <"$file")
        raw=$((raw + size))
        br=$((br + $(br_size "$file")))
        if [ "$ext" = wasm ] && [ "$size" -gt "$wasm_raw" ]; then
            wasm_raw=$size
        fi
    done < <(find "$public_dir" -type f -name "*.$ext" | sort)
    table="$table| \`.$ext\` | $count | $(human "$raw") | $(human "$br") |
"
done

summary="### Web bundle size

| Asset | Files | Raw | Brotli |
| --- | ---: | ---: | ---: |
$table"

status=0
if [ -n "$limit" ]; then
    if [ "$wasm_raw" -gt "$limit" ]; then
        summary="$summary
Raw wasm **$(human "$wasm_raw")** exceeds the limit of $(human "$limit") (\`WASM_SIZE_LIMIT_BYTES=$limit\`)."
        status=1
    else
        summary="$summary
Raw wasm $(human "$wasm_raw") is within the limit of $(human "$limit")."
    fi
fi

echo "$summary"
if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
    echo "$summary" >>"$GITHUB_STEP_SUMMARY"
fi

exit "$status"
