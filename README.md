# MPC
This project is to perform secure computation for the maximum/k^{th} ranked element from a secret set where each element is submitted as an Boolean SS.

The are a few directories in the solution:
  - data: store the generated offline data
  - test: store auto-generated binary files for test purpose
  - libfss: implmentation for used fss primitives, including dpf, idpf, and ic (interval containment) gate.
  - frontend: A frontend for bencharmking all basic protocols in libmpc.

<!-- RUSTFLAGS="-A warnings" -->

RUSTFLAGS="-A warnings" cargo build --release

Open two terminals
    - cd frontend/target/release/
    - In the 1st terminal: sudo ip netns exec ns1 ./target/release/frontend 0
    - In the 2nd terminal: sudo ip netns exec ns2 ./target/release/frontend 1
