# Chip-8 Interpreter in Rust

This project is a Chip-8 interpreter implemented in Rust. It is designed as a learning exercise to understand Rust programming concepts while building a functional emulator for the Chip-8 virtual machine.

## Project Structure

The project is divided into two main components:
1. **`chip8_core`**: Contains the core logic for the Chip-8 interpreter, including memory management, registers, and instruction execution.
2. **`chip8_desktop`**: Provides a desktop interface for running the interpreter, including rendering and input handling.

## Goals

- Learn Rust by implementing a Chip-8 interpreter.
- Understand low-level emulation concepts such as memory management, registers, and instruction decoding.
- Build a functional emulator capable of running classic Chip-8 games.

## Getting Started

1. **Prerequisites**:
   - Rust and Cargo installed on your system.
   - Basic familiarity with Rust programming.

2. **Build and Run**:
   ```sh
   cargo build
   cargo run
   ```

## Next Steps

- Implement the remaining Chip-8 instructions.
- Add support for rendering and keyboard input.
- Test the interpreter with sample ROMs.

## Resources

- [Chip-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
- [Rust Documentation](https://doc.rust-lang.org/book/)

Happy coding!