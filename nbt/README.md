# nbt

NBT parsing crate.

Assumes a 64-bit platform.

TODO: Write a better description.

## Design

- Lazy: only parse what you need
- No unsafe for performance sake

## Todo

- [x] Cache parsed results
- [ ] Write more tests
- [x] Ser/de from/into structs
- [ ] Switch to byteorder
- [x] Format NBT
- [x] Visitor API