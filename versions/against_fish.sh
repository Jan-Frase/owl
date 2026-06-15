new="$1"

for skill in {0..10}; do
  echo ""
  echo "===== Testing against Stockfish Skill Level $skill ====="
  ./fastchess/fastchess \
    -engine cmd="./$new" name="$new" \
    -engine cmd="stockfish" name="stockfish" option.UCI_LimitStrength=true option."Skill Level"=$skill \
    -each tc=10+0.1 \
    -openings file=./Openings-PGN/2moves_LT_1000.pgn format=pgn order=sequential \
    -rounds 20 -repeat -concurrency 6 \
    # -pgnout file=pgn append=false -log file=logs level=warn append=false engine=true \
  echo ""
  echo ""
done
