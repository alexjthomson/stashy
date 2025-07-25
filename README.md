# Stashy 🦀
A trait-based abstraction for stashing key-value data in Rust.
you to define how and where data gets stashed — whether locally in memory,
remotely, or anywhere else you can implement the `Stash` trait.

## 🚀 Features
- 🧩 **Trait-based**: Define your own stashing logic by implementing the `Stash`
  trait.
- 🔌 **Pluggable backends**: Redis, local in-memory, file-based — anything goes.
- 🧪 **Test-friendly**: Swap out stashes in tests or use a mock stash.
- ✅ **Minimal dependencies**: Lightweight by default.

## 🧰 Use Cases
- Session storage
- Caching layers
- Feature flag toggles

## 🔌 Planned Backend Implementations
- [x] In-memory
- [ ] Filesystem
- [x] Redis
- [ ] DynamoDB
- [ ] RocksDB