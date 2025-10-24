# YouRAM

Your RAM: An open-source static random access memory (SRAM) compiler.

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)



## What is YouRAM?

YouRAM is an open-source, cross-technology SRAM compiler written in Rust. It can automatically generate SPICE netlists for SRAM circuits and perform functional simulations.



## Quick start

Install [Cargo](https://cargo.site/). 

Clone the repository

```bash
git clone https://github.com/MoleSir/YouRAM
cd YouRAM
```

Cargo will automatically download all dependencies.

Run a simple example:

````
cargo run -- -c ./config/nangate45_4_4.json
````

This will use the Nangate45 process library to create an SRAM with 4-bit address width and 4-bit data width, perform a simple functional test, and finally output the SPICE netlist to the `output` directory.



## Document

For details on the design and usage of this project, please refer to the [document](./document).




## Referecens

- https://github.com/VLSIDA/OpenRAM