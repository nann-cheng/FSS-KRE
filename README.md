# MPC
This project aims to compute the maximum/k^{th} ranking element from a secret set.

The are a few directories in the solution:
  - data: store the generated offline data
  - test: store auto-generated binary files for test purpose
  - libidpf: some underlying structures and libraries used by the offline project and AsynParty project
  - offline: the project to generate offline data used the the two mpc parties
  - max: a project to  run the main function.
  
You can run the example by this way:
  Open two terminals
    - in the first terminal, enter the max directory, run the command: cargo run --example max_server
    - in the second terminal, enter the max directory, run the command: cargo run --example max_client


