#!/usr/bin/env python3
import sys
import subprocess

def main():
    if len(sys.argv) < 2:
        print("Usage: python script.py <new_engine_name>")
        sys.exit(1)

    new = sys.argv[1]


    #
    versions = ["09", "11", "12", "13", "14", "17"]
    elos = [1271, 1686, 1880, 1965, 2054, 2295]
    for (version, elo) in zip(versions, elos):
        print()
        print(f"===== Testing against Stash {version} with elo {elo}") 

        # Build the command as a list of arguments
        cmd = [
            "../fastchess/fastchess",
            "-engine", f"cmd=../{new}", f"name={new}",
            "-engine", f"cmd=stash-{version}", f"name=stash-{version}",
            "-each", "tc=10+0.1",
            "-openings", "file=../Openings-PGN/2moves_LT_1000.pgn", "format=pgn", "order=sequential",
            "-rounds", "20",
            "-repeat",
            "-concurrency", "6",
            "-scoreinterval", "10"
        ]

        # Uncomment the following lines if you want to enable logging and PGN output
        # cmd.extend(["-log", f"file=log_{skill}.log", "level=info", "append=false", "engine=true",
        #             "-pgnout", f"file=games_{skill}.pgn", "append=false"])

        # Run the command
        result = subprocess.run(cmd, capture_output=False, text=True)

        # Optionally check for errors
        if result.returncode != 0:
            print(f"Warning: fastchess returned non-zero exit code {result.returncode} for version {version}")
            break

        print()
        print()

if __name__ == "__main__":
    main()
