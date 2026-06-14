#!/usr/bin/env python3
"""
Plot version performance as horizontal bar charts.

Reads a CSV file with columns: version, update, wins, losses, draws.
For each version, creates a horizontal bar chart with green (wins), orange (draws), red (losses).
The 'update' text is shown above the respective bar chart.
"""

import sys
import pandas as pd
import matplotlib.pyplot as plt


def plot_version_performance(csv_file: str, output_image: str = "versions_plot.png"):
    """Create horizontal bar charts for each version from the CSV file."""
    # Read the CSV file
    try:
        df = pd.read_csv(csv_file)
    except FileNotFoundError:
        print(f"Error: File '{csv_file}' not found.")
        sys.exit(1)
    except Exception as e:
        print(f"Error reading CSV: {e}")
        sys.exit(1)

    # Strip any leading/trailing spaces from column names
    df.columns = df.columns.str.strip()

    required_columns = {"version", "update", "wins", "losses", "draws"}
    if not required_columns.issubset(df.columns):
        print("Error: CSV must contain columns: version, update, wins, losses, draws")
        sys.exit(1)

    # Ensure numeric columns are integers
    for col in ["wins", "losses", "draws"]:
        df[col] = pd.to_numeric(df[col], errors="coerce").fillna(0).astype(int)

    num_versions = len(df)
    if num_versions == 0:
        print("No data found in CSV.")
        sys.exit(0)

    # Create subplots (one row per version)
    fig, axes = plt.subplots(
        nrows=num_versions, ncols=1, figsize=(8, 2 * num_versions), sharex=True
    )
    # Ensure axes is always a list (even for a single subplot)
    if num_versions == 1:
        axes = [axes]

    categories = ["Wins", "Draws", "Losses"]
    colors = ["green", "orange", "red"]

    for idx, row in df.iterrows():
        ax = axes[idx]
        values = [row["wins"], row["draws"], row["losses"]]

        # Horizontal bar chart
        bars = ax.barh(categories, values, color=colors, edgecolor="black", linewidth=0.5, height=1)

        # Add value labels on the bars (optional)
        for bar, val in zip(bars, values):
            if val > 0:
                ax.text(
                    val + max(1, max(values) * 0.02),  # small offset
                    bar.get_y() + bar.get_height() / 2,
                    str(val),
                    va="center",
                    ha="left",
                    fontsize=9,
                )

        # Set title with version and change note
        ax.set_title(f"Version {row['version']}: {row['update']}", fontweight="bold")

        # Remove y-axis label (categories are self-explanatory)
        ax.set_ylabel("")
        # Add grid lines on x-axis for readability
        ax.xaxis.grid(True, linestyle="--", alpha=0.7)
        ax.set_axisbelow(True)  # grid behind bars

    # Common x-axis label
    axes[-1].set_xlabel("Number of games")

    # Overall figure title
    fig.suptitle("Performance per version", fontsize=14, fontweight="bold", y=0.98)

    # Adjust layout to prevent title overlapping
    plt.tight_layout(rect=[0, 0, 1, 0.96])

    # Save and show
    plt.savefig(output_image, dpi=150, bbox_inches="tight")
    print(f"Plot saved as '{output_image}'")
    plt.show()


if __name__ == "__main__":
    # Use the provided CSV filename or default to 'changelog.csv'
    csv_filename = sys.argv[1] if len(sys.argv) > 1 else "changelog.csv"
    plot_version_performance(csv_filename)
