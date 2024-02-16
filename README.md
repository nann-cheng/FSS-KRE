# MPC
This project is to perform secure computation for the maximum/k^{th} ranked element from a secret set where each element is submitted as an Boolean SS.

The are a few directories in the solution:
  - data: store the generated offline data
  - test: store auto-generated binary files for test purpose
  - libfss: implmentation for used fss primitives, including dpf, idpf, and ic (interval containment) gate.
  - offline: assisting in generating offline data used for online 2PC computation.
  - frontend: A frontend for bencharmking all basic protocols in libmpc.

To generate offline data (for batch_max), Open one terminal:
    - cd frontend
    - cargo test gen_offlinedata -- --nocapture (-- --nocapture is used to diplay standard I/O)
    - cargo test --release gen_offlinedata -- --nocapture (-- --nocapture is used to diplay standard I/O)

Open two terminals
    - cd frontend
    - In the 1st terminal: cargo run --release 0
    - In the 2nd terminal: cargo run --release 1

The results are written in "test", in frontend repo then run "cargo test test_result" to verify the results.


<!-- 
How to run the whole benchmarking from scratch?

1. git clone https://github.com/nann-cheng/FSS-KRE.git

git checkout -b latest
git branch --set-upstream-to=origin/latest


2.(for rustc) apt update
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

3.firewall

apt install ufw
ufw enable

ufw allow 8088/tcp
ufw disable
ufw enable

 -->
