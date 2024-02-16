# MPC
This project is to perform secure computation for the maximum/k^{th} ranked element from a secret set where each element is submitted as an Boolean SS.

The are a few directories in the solution:
  - data: store the generated offline data
  - test: store auto-generated binary files for test purpose
  - libfss: implmentation for used fss primitives, including dpf, idpf, and ic (interval containment) gate.
  - frontend: A frontend for bencharmking all basic protocols in libmpc.

Open two terminals
    - cd frontend
    - In the 1st terminal: cargo run --release 0
    - In the 2nd terminal: cargo run --release 1
