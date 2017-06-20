
#!/bin/bash
# setup_precommit.sh
if [ -d "./.git/hooks/" ]; then
OUTFILE="./.git/hooks/pre-commit"         # Name of the file to generate.

# -----------------------------------------------------------
# 'Here document containing the body of the generated script.
(
cat <<'EOF'
#!/bin/bash

echo "This pre-commit hook was generated by entering '/bin/bash setup_precommit.sh' into the console at repo root."
echo "running rust fmt..."

check_char='\xE2\x9C\x93'
cross_char='\xE2\x9D\x8C'
green='\033[0;32m'
nc='\033[0m'
check="$green$check_char$nc"
errors=0

cargo fmt --write-mode=replace
fi

echo -n "Running tests... "
if result=$(cargo test --color always 2>&1); then
	echo -e "$check"
else
	echo "$result"
	errors=1
fi

if [ "$errors" != 0 ]; then
	echo "Failed"
	exit 1
else
	echo "OK"
fi

exit 0
EOF
) > $OUTFILE
# -----------------------------------------------------------

#  Quoting the 'limit string' prevents variable expansion
#+ within the body of the above 'here document.'
#  This permits outputting literal strings in the output file.

if [ -f "$OUTFILE" ]; then
chmod 755 $OUTFILE
echo "Done!"
fi
else
echo "Whoops... Are you running setup_precommit.sh from the repo root?"
fi