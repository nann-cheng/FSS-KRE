# MPC
This project is to perform secure computation for the maximum/k^{th} ranking element from a secret set.

The are a few directories in the solution:
  - data: store the generated offline data
  - test: store auto-generated binary files for test purpose
  - libfss: implmentation for those fss primitives 
  - offline: assisting in generating offline data used for online 2PC
  - frontend: A frontend for bencharmking all basic protocols in libmpc.

To generate offline data (for batch_max), Open one terminal:
    - cd frontend
    - cargo test batch_max_gen_offlinedata

Open two terminals
    - cd frontend
    - In the 1st terminal: cargo run 0
    - In the 2nd terminal: cargo run 1

The results are written in "test", in frontend repo then run "cargo test test_result" to verify the results.

