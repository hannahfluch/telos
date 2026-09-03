# telos

Guide on writing a kernel starter in Rust.

This repository covers the guide of writing a basic bootloader with a custom boot protocol for x86-64.

The kernel is a minimal entry stub.

## Getting Started

### Prerequisites
- Nix package manager

### Cloning the Repository

Clone the repository to your local machine:

```bash
git clone https://github.com/hannahfluch/telos.git
cd telos
```

The nix flake provides several outputs to run telos.
The `kernel` and `loader` can be built using:
```bash
nix build .#kernel
```
> for loader: .#loader

### Running

#### QEMU

```bash
nix run .
```

#### Real Machine

```bash
nix run .#flash
```

## Development

In order to work on the project, the flake provides a dev shell, which can be invoked with:
```bash
nix develop
```

