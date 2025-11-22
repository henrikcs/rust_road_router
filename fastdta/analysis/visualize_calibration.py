#!/usr/bin/env python3
"""
Visualize traffic model calibration progress.

This script reads traffic model data logged during calibration and generates
plots showing the traffic model curve with parameters and observed data points.
"""

import argparse
import os
import sys
from pathlib import Path
import matplotlib.pyplot as plt
import numpy as np


class TrafficModelInvocation:
    """Represents a single invocation of the calibration function."""

    def __init__(self, model_type, params, densities, speeds):
        self.model_type = model_type
        self.params = params
        self.densities = densities
        self.speeds = speeds

    def __repr__(self):
        return f"TrafficModelInvocation({self.model_type}, params={self.params}, n_obs={len(self.densities)})"


def parse_traffic_model_file(filepath):
    """Parse the traffic model data file and return a list of invocations."""
    invocations = []

    with open(filepath, 'r') as f:
        lines = f.readlines()

    i = 0
    while i < len(lines):
        line = lines[i].strip()

        if line.startswith("INVOCATION|"):
            model_type = line.split("|")[1]

            # Read params
            i += 1
            params_line = lines[i].strip()
            if not params_line.startswith("PARAMS|"):
                raise ValueError(f"Expected PARAMS line, got: {params_line}")
            params = [float(x) for x in params_line.split("|")[1].split(",")]

            # Read densities
            i += 1
            densities_line = lines[i].strip()
            if not densities_line.startswith("DENSITIES|"):
                raise ValueError(
                    f"Expected DENSITIES line, got: {densities_line}")
            densities_str = densities_line.split("|")[1]
            densities = [float(x) for x in densities_str.split(
                ",")] if densities_str else []

            # Read speeds
            i += 1
            speeds_line = lines[i].strip()
            if not speeds_line.startswith("SPEEDS|"):
                raise ValueError(f"Expected SPEEDS line, got: {speeds_line}")
            speeds_str = speeds_line.split("|")[1]
            speeds = [float(x)
                      for x in speeds_str.split(",")] if speeds_str else []

            invocations.append(TrafficModelInvocation(
                model_type, params, densities, speeds))

        i += 1

    return invocations


def modified_lee_speed(density, free_flow_speed, a, e, theta, jam_density):
    """Calculate speed using Modified Lee traffic model."""
    d = density / jam_density
    speed = free_flow_speed * (1.0 - d**a) / (1.0 + e * d**theta)
    return speed


def calculate_r_squared(observed_speeds, observed_densities, free_flow_speed, a, e, theta, jam_density):
    """Calculate R² (coefficient of determination) for the model fit."""
    if len(observed_speeds) == 0:
        return 0.0

    # Predicted speeds using the model
    predicted_speeds = [modified_lee_speed(d, free_flow_speed, a, e, theta, jam_density)
                        for d in observed_densities]

    # Mean of observed speeds
    mean_observed = np.mean(observed_speeds)

    # Total sum of squares
    ss_tot = sum((y - mean_observed)**2 for y in observed_speeds)

    # Residual sum of squares
    ss_res = sum((y - y_pred)**2 for y,
                 y_pred in zip(observed_speeds, predicted_speeds))

    # R²
    if ss_tot == 0:
        return 0.0
    r_squared = 1.0 - (ss_res / ss_tot)

    return r_squared


def compute_axis_limits(invocations):
    """Compute the maximum axis limits across all invocations."""
    max_density = 0
    max_speed = 0

    for invocation in invocations:
        if invocation.model_type == "ModifiedLee":
            # Use jam_density for x-axis limit
            jam_density = invocation.params[4]
            max_density = max(max_density, jam_density)

            # Free flow speed for y-axis limit
            free_flow_speed = invocation.params[0]
            max_speed = max(max_speed, free_flow_speed)

            # Also check observed data
            if invocation.densities:
                max_density = max(max_density, max(invocation.densities))
                max_speed = max(max_speed, max(invocation.speeds))

    # Add some padding (10%)
    max_density *= 1.1
    max_speed *= 1.1

    return max_density, max_speed


def plot_invocation(invocation, output_path, invocation_index, max_density, max_speed):
    """Create a plot for a single calibration invocation."""
    fig, ax = plt.subplots(figsize=(10, 6))

    if invocation.model_type == "ModifiedLee":
        # Extract parameters
        free_flow_speed, a, e, theta, jam_density = invocation.params

        # Generate density range for plotting the model curve (up to jam_density)
        density_range = np.linspace(0, jam_density, 500)
        speeds_model = [modified_lee_speed(d, free_flow_speed, a, e, theta, jam_density)
                        for d in density_range]

        # Plot the traffic model curve
        ax.plot(density_range, speeds_model, 'b-',
                linewidth=2, label='Traffic Model')

        # Plot observed data points
        if invocation.densities:
            ax.scatter(invocation.densities, invocation.speeds,
                       c='red', s=50, alpha=0.7, label='Observed Data', zorder=5)

            # Calculate R²
            r_squared = calculate_r_squared(invocation.speeds, invocation.densities,
                                            free_flow_speed, a, e, theta, jam_density)
        else:
            r_squared = 0.0

        # Add parameter text box
        param_text = (
            f"Model: Modified Lee\n"
            f"Free Flow Speed: {free_flow_speed:.2f} km/h\n"
            f"a: {a:.4f}\n"
            f"e: {e:.4f}\n"
            f"θ: {theta:.4f}\n"
            f"k_j (Jam Density): {jam_density:.2f} veh/km\n"
            f"Observations: {len(invocation.densities)}\n"
            f"R²: {r_squared:.4f}"
        )

        # Place text box in upper right
        ax.text(0.98, 0.97, param_text, transform=ax.transAxes,
                verticalalignment='top', horizontalalignment='right',
                bbox=dict(boxstyle='round', facecolor='wheat', alpha=0.8),
                fontsize=9, family='monospace')

    else:
        raise ValueError(
            f"Unknown traffic model type: {invocation.model_type}")

    # Set labels and title
    ax.set_xlabel('Density (veh/km)', fontsize=12)
    ax.set_ylabel('Speed (km/h)', fontsize=12)
    ax.set_title(
        f'Traffic Model Calibration - Invocation {invocation_index}', fontsize=14)
    ax.grid(True, alpha=0.3)
    ax.legend(loc='upper left')

    # Set consistent axis limits across all plots
    ax.set_xlim(0, max_density)
    ax.set_ylim(0, max_speed)

    plt.tight_layout()
    plt.savefig(output_path, dpi=150, bbox_inches='tight')
    plt.close()

    print(f"Created plot: {output_path}")


def main():
    parser = argparse.ArgumentParser(
        description='Visualize traffic model calibration progress'
    )
    parser.add_argument('input', type=str,
                        help='Path to traffic model data file (e.g., traffic_model_data_a2)')
    parser.add_argument('output', type=str,
                        help='Output directory for plots')

    args = parser.parse_args()

    # Check if input file exists
    if not os.path.isfile(args.input):
        print(f"Error: Input file '{args.input}' not found", file=sys.stderr)
        sys.exit(1)

    # Create output directory if it doesn't exist
    output_dir = Path(args.output)
    output_dir.mkdir(parents=True, exist_ok=True)

    # Parse the traffic model data file
    print(f"Reading traffic model data from: {args.input}")
    invocations = parse_traffic_model_file(args.input)
    print(f"Found {len(invocations)} calibration invocations")

    # Compute consistent axis limits across all invocations
    max_density, max_speed = compute_axis_limits(invocations)
    print(
        f"Using axis limits: density=[0, {max_density:.2f}], speed=[0, {max_speed:.2f}]")

    # Get base name from input file
    base_name = Path(args.input).stem

    # Generate plots for each invocation
    for i, invocation in enumerate(invocations):
        output_path = output_dir / f"{base_name}.{i}.png"
        plot_invocation(invocation, output_path, i, max_density, max_speed)

    print(f"\nSuccessfully created {len(invocations)} plots in {output_dir}")


if __name__ == "__main__":
    main()
