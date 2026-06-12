
#!/bin/bash
set -e

usage()
{
  echo "Usage: ./run_fastchess.sh name_new name_old [rounds]"
  echo ""
  echo "This runs fastchess with the new engine vs the old engine."
  echo "If rounds is not given, 250 rounds are played."

  exit 1
}

# Check arguments: need at least 2, at most 3
if [ $# -lt 2 ] || [ $# -gt 3 ]; then
  usage
fi

new="$1"
old="$2"

# Set rounds: use $3 if provided, otherwise default to 250
if [ -n "$3" ]; then
  rounds="$3"
else
  rounds=250
fi


./fastchess/fastchess \
  -engine cmd="./$new" name="$new" \
  -engine cmd="./$old" name="$old" \
  -each tc=10+0.1 \
  -openings file=./Openings-PGN/2moves_LT_1000.pgn format=pgn order=sequential \
  -pgnout file=pgn append=false \
  -rounds "$rounds" -repeat -concurrency 6
