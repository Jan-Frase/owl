#!/bin/bash
new="$1"

for skill in {0..8}; do
  echo ""
  echo "===== Testing against Stockfish Skill Level $skill ====="
  ./fastchess/fastchess \
    -engine cmd="./$new" name="$new" \
    -engine cmd="stockfish" name="stockfish" "option.Skill Level=$skill" \
    -each tc=10+0.1 \
    -openings file=./Openings-PGN/2moves_LT_1000.pgn format=pgn order=sequential \
    -rounds 20 -repeat -concurrency 6 \
    # -log file="log_$skill.log" level=info append=false engine=true -pgnout file="games_$skill.pgn" append=false
  echo ""
  echo ""
done
