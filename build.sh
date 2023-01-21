!#/bin/bash

set -e

echo "- Building sconectl"

cargo build

echo "- Building README.md"

cp README.template README.md
./target/debug/sconectl  --help 2>> README.md
echo "\`\`\`" >> README.md

echo "DONE"
