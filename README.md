# Ruyi Slab

[![release](https://github.com/ruyi-rs/slab/workflows/release/badge.svg)](https://github.com/ruyi-rs/slab/actions)
[![license](https://img.shields.io/crates/l/ruyi-slab)](https://github.com/ruyi-rs/slab)
[![crates.io](https://img.shields.io/crates/v/ruyi-slab)](https://crates.io/crates/ruyi-slab)
[![docs](https://docs.rs/ruyi-slab/badge.svg)](https://docs.rs/ruyi-slab)

An object based allocator backed by a contiguous growable array of slots.

The slab allocator pre-allocates memory for objects of same type so that
it reduces fragmentation caused by allocations and deallocations. When
allocating memory for an object, it just finds a free (unused) slot, marks
it as used, and returns the index of the slot for later access to the
object. When freeing an object, it just adds the slot holding the object
to the list of free (unused) slots after dropping the object.

## No-std Support

To use ruyi-slab without the Rust standard library but with a memory allocator:

```toml
[dependencies]
ruyi-slab = { version = "0.1", default-features = false }
```

## License

Licensed under either of

* Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
* MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
