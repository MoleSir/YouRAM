# YouRAM

Your RAM: An open-source static random access memory (SRAM) compiler.

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)



## What is YouRAM?

YouRAM is an open-source, cross-technology SRAM compiler written in Rust. It can automatically generate SPICE netlists for SRAM circuits and perform functional simulations.



## Quick start


1. Install [Cargo](https://doc.rust-lang.org/cargo/) and a SPICE simulator supported by YouRAM, such as [Ngspice](https://ngspice.sourceforge.io/).

2. Clone the repository:

   ```bash
   git clone https://github.com/MoleSir/YouRAM
   cd YouRAM
   ```

3. Cargo will automatically download all dependencies.
   
4. Run a simple example:
   
   ```bash
   cargo run -- -c ./config/nangate45_4_4.json
   ```

   This command uses the Nangate45 process library to create an SRAM with a 4-bit address width and a 4-bit data width.

   It performs a simple functional test and timing characterization (to obtain SRAM delay and transition data),

   then outputs the SPICE/Verilog netlists and Liberty model to the `output` directory.



## Document

For details on the design and usage of this project, please refer to the [document](./document).




## Referecens

- https://github.com/VLSIDA/OpenRAM