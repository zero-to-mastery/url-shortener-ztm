@echo off
REM Script to run cargo audit while ignoring unmaintained unic crate warnings
REM These warnings are not security vulnerabilities, just unmaintained crates

cargo audit --deny warnings ^
  --ignore RUSTSEC-2025-0081 ^
  --ignore RUSTSEC-2025-0075 ^
  --ignore RUSTSEC-2025-0080 ^
  --ignore RUSTSEC-2025-0074 ^
  --ignore RUSTSEC-2025-0104 ^
  --ignore RUSTSEC-2025-0098
