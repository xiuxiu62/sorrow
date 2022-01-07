# Sorrow OS

A sad little operating system.

## Prerequisites
- Latest nightly build of rustc
- Rustup llvm-preview-tools
- Preferably a linux dev environment
- Qemu
- Kvm (optional, but recommended for linux users)

## Build and create image
`cargo kbuild`
`cargo kimage`

## Run
`cargo krun`

## Test 
`cargo ktest`

## Todo
- [x] GOP interface 
- [ ] Primitive shell
- [ ] Primitive disk location and sector reading
  - [x] Read sectors from lba 0
  - [ ] Search and index lba labels
  - [ ] Read sectors
  - [ ] Write sectors
- [ ] Basic ext2 filesystem
- [ ] Path parsing
