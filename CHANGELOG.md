# Changes for [`disk-spinner`](https://crates.io/crates/disk-spinner)

<!-- next-header -->

## [[0.3.0](https://docs.rs/disk-spinner/0.3.0/disk-spinner/)] - 2026-03-03

## Added

- Now comes with a Dockerfile.
- Customizable pseudorandom functions can be used: 
  - Shishua (requires the shishua CLI tool installed)
  - Blake3 (new)
  - AES (existing)

### Changed

- The default buffer size increased to 16MB (up from 8kB).
- Block devices are now opened with `O_EXCL`.
- The partition detection check can now deal with more than 26 physical disks.

## [[0.2.0](https://docs.rs/disk-spinner/0.2.0/disk-spinner/)] - 2025-08-06

- Initial release on crates.io.
