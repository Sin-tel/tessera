#!/usr/bin/env bash

# Override the Love2D executable by prepending to PATH or setting $LOVE:
#   export LOVE="/custom/love/path"
#   export PATH="/custom/love/dir:$PATH"

# Try in order: 1. $LOVE, 2. $PATH with extras, 3. Common install locations
if [[ -z "$LOVE" ]]; then
    # Add common install directories to PATH (MacOS, Linux, Homebrew)
    PATH="$PATH:\
/Applications/love.app/Contents/MacOS:\
$HOME/Applications/love.app/Contents/MacOS:\
/usr/games:\
/opt/homebrew/bin"
    
    # Find first valid executable (love/love2d) in modified PATH
    LOVE=$(command -v love love2d 2>/dev/null | head -n1)
fi

# Verify executable exists and is runnable
if [[ ! -x "$LOVE" ]]; then
    cat <<EOF
ERROR: Love2D not found. Either:
1. Install love2d package
2. Set LOVE environment variable:
   export LOVE="/path/to/love"
3. Add love's directory to PATH:
   export PATH="\$PATH:/path/to/love/dir"
EOF
    exit 1
fi

# Switch to lua directory and run
pushd lua >/dev/null || exit 1
"$LOVE" .
exit_code=$?
popd >/dev/null

exit $exit_code
