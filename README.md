# Sorrow OS

## A little bit of an operating system

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
