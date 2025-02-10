#!/usr/bin/env bash

# Override the LÖVE executable by setting:
#   export LOVE="/custom/path/to/love"
# before running this script

# Check in order: 1. $LOVE env, 2. PATH, 3. Common locations
LOVE_EXEC=""
if [[ -n "$LOVE" && -x "$LOVE" ]]; then
    LOVE_EXEC="$LOVE"
else
    # Check for standard executable names
    for cmd in love love2d; do
        path=$(command -v "$cmd" 2>/dev/null)
        [[ -x "$path" ]] && { LOVE_EXEC="$path"; break; }
    done
    
    # Fallback to common install paths
    [[ -z "$LOVE_EXEC" ]] && while IFS= read -r path; do
        [[ -x "$path" ]] && { LOVE_EXEC="$path"; break; }
    done <<< "$(printf '%s\n' \
        '/usr/bin/love' \
        '/usr/local/bin/love' \
        '/Applications/love.app/Contents/MacOS/love' \
        "$HOME/Applications/love.app/Contents/MacOS/love")"
fi

[[ -z "$LOVE_EXEC" ]] && {
    echo "ERROR: LÖVE not found. Try:"
    echo "1. Install love2d package"
    echo "2. export LOVE='/path/to/love'"
    exit 1
}

# Run in lua directory with directory stack
pushd lua >/dev/null || { echo "Missing 'lua' directory"; exit 1; }
"$LOVE_EXEC" . 
exit_code=$?
popd >/dev/null

exit $exit_code