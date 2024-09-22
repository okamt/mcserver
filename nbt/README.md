# nbt

NBT parsing crate.

## Design

- Lazy: only parse what you need
- No unsafe for performance sake

## Todo

- [x] Cache parsed results
- [ ] Write more tests
- [ ] Ser/de from/into structs
- [ ] Switch to byteorder maybe
- [x] Format NBT
- [x] Visitor API