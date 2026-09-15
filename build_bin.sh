#!/bin/bash

espflash save-image --merge --chip esp32c3 target/riscv32imc-unknown-none-elf/release/cube target/riscv32imc-unknown-none-elf/release/cube.bin
