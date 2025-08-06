This project provides a fast and reliable block randomizer written in Rust for clinical trial allocation.
It supports customizable block sizes, reproducible results with seed control, and optional stratification by variables such as age or gender.
The program generates a CSV file with randomized group assignments for each subject. Stratification variables (if used) are retained in the output.

Installation

git clone https://github.com/spak432/blockrandomizer.git
cd blockrandomizer
cargo build --release

Usage example (not yet implemented)

blockrandomizer --input strata.csv --block-size 4 --output allocation.csv

Options (not yet implemented)

--block-size: Specify the block size for randomization
--seed: Fix the random seed for reproducibility
--strata: (Optional) Specify stratification variables (e.g., age, gender)

License

This tool is intended for educational or research purposes. Please feel free to modify and use it under the terms of the MIT License.
