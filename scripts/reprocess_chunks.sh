#!/usr/bin/env bash
# reprocess_chunks.sh — POST every saved audio chunk to speech-swift and log results.
#
# Mirrors exactly what the app does when sending live chunks:
#   POST http://localhost:<PORT>/registry/sessions
#   multipart/form-data, field name "file", content-type audio/wav
#
# Usage:
#   ./scripts/reprocess_chunks.sh [--port 8080] [--session <id>] [--dry-run]
#
# Options:
#   --port      speech-swift port (default: 8080)
#   --session   only reprocess chunks for this session id
#   --dry-run   print what would be sent without actually calling the API

set -euo pipefail

PORT=8080
SESSION_FILTER=""
DRY_RUN=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --port)    PORT="$2";            shift 2 ;;
        --session) SESSION_FILTER="$2";  shift 2 ;;
        --dry-run) DRY_RUN=true;         shift   ;;
        *) echo "Unknown option: $1" >&2; exit 1 ;;
    esac
done

BASE_URL="http://localhost:${PORT}"
CHUNKS_DIR="${HOME}/Library/Application Support/com.tinkermonkey.minutes/audio_chunks"

if [[ ! -d "$CHUNKS_DIR" ]]; then
    echo "No chunks directory found at: $CHUNKS_DIR"
    exit 1
fi

# Collect WAV files, sorted by session then start_ms (numeric order).
# Use a temp file instead of mapfile/process substitution for bash 3 compatibility.
_TMP_LIST=$(mktemp)
trap 'rm -f "$_TMP_LIST" /tmp/reprocess_response.json' EXIT

if [[ -n "$SESSION_FILTER" ]]; then
    SESSION_DIR="${CHUNKS_DIR}/${SESSION_FILTER}"
    if [[ ! -d "$SESSION_DIR" ]]; then
        echo "Session directory not found: $SESSION_DIR" >&2
        exit 1
    fi
    find "$SESSION_DIR" -name "*.wav" | sort -t/ -k1,1 > "$_TMP_LIST"
else
    find "$CHUNKS_DIR" -name "*.wav" | sort -t/ -k1,1 > "$_TMP_LIST"
fi

TOTAL=$(wc -l < "$_TMP_LIST" | tr -d ' ')
if [[ $TOTAL -eq 0 ]]; then
    echo "No WAV chunks found."
    exit 0
fi

echo "Found $TOTAL chunk(s) under: $CHUNKS_DIR"
[[ -n "$SESSION_FILTER" ]] && echo "Filtering to session: $SESSION_FILTER"
$DRY_RUN && echo "(dry-run — no requests will be made)"
echo ""

OK=0
FAIL=0

while IFS= read -r WAV; do
    # Extract session_id and start_ms from the path.
    SESSION_ID=$(basename "$(dirname "$WAV")")
    START_MS=$(basename "$WAV" .wav)
    SIZE=$(wc -c < "$WAV" | tr -d ' ')
    DURATION_MS=""

    # Compute duration from WAV header: bytes 28-31 = byte rate (little-endian int32).
    # duration_s = (file_size - 44) / byte_rate
    if command -v python3 &>/dev/null; then
        DURATION_MS=$(python3 - "$WAV" <<'EOF'
import sys, struct
path = sys.argv[1]
with open(path, 'rb') as f:
    f.seek(28)
    byte_rate = struct.unpack('<I', f.read(4))[0]
    f.seek(0, 2)
    total = f.tell()
    data_bytes = max(0, total - 44)
    ms = int(data_bytes / byte_rate * 1000) if byte_rate else 0
    print(ms)
EOF
)
    fi

    LABEL="session=${SESSION_ID} start=${START_MS}ms"
    [[ -n "$DURATION_MS" ]] && LABEL="${LABEL} duration=${DURATION_MS}ms"
    LABEL="${LABEL} size=${SIZE}B"

    if $DRY_RUN; then
        echo "[dry-run] $LABEL"
        continue
    fi

    printf "%-60s  " "$LABEL"

    HTTP_CODE=$(curl -s -o /tmp/reprocess_response.json -w "%{http_code}" \
        -X POST "${BASE_URL}/registry/sessions" \
        -F "file=@${WAV};type=audio/wav")

    if [[ "$HTTP_CODE" == "200" ]]; then
        # Extract summary fields from JSON response.
        if command -v python3 &>/dev/null; then
            SUMMARY=$(python3 - /tmp/reprocess_response.json <<'EOF'
import sys, json
try:
    with open(sys.argv[1]) as f:
        d = json.load(f)
    segs = d.get("segments", [])
    speakers = sorted({s.get("speaker_label", "?") for s in segs})
    words = sum(len(s.get("transcript", "").split()) for s in segs)
    print(f"{len(segs)} segment(s)  {words} word(s)  speakers: {', '.join(speakers)}")
except Exception as e:
    print(f"(could not parse response: {e})")
EOF
)
        else
            SUMMARY="HTTP 200"
        fi
        echo "OK  $SUMMARY"
        (( OK++ )) || true
    else
        ERROR=$(python3 - /tmp/reprocess_response.json 2>/dev/null <<'EOF' || cat /tmp/reprocess_response.json
import sys, json
with open(sys.argv[1]) as f:
    d = json.load(f)
print(d.get("error", json.dumps(d)))
EOF
)
        echo "FAIL (HTTP ${HTTP_CODE})  ${ERROR}"
        (( FAIL++ )) || true
    fi
done < "$_TMP_LIST"

echo ""
echo "Done. OK=${OK}  FAIL=${FAIL}  TOTAL=${TOTAL}"
