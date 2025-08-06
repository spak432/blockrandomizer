This project provides a simple block randomizer written in Rust for clinical trial allocation.
It supports customizable block sizes, reproducible results with seed control, and optional stratification by variables such as age or gender.
The program generates a CSV file with randomized group assignments for each subject. Stratification variables (if used) are retained in the output.

### Installation
```
git clone https://github.com/spak432/blockrandomizer.git
cd blockrandomizer
cargo build --release
```
<img width="401" height="316" alt="screenshot" src="https://github.com/user-attachments/assets/0e1a60d5-0292-4e35-99ae-5b9d4731ef4c" />

### License

This tool is intended for research purposes. Please feel free to modify and use it under the terms of the MIT License.
