# GLOC Documentation

> **G**lobal · **L**ogic · **O**rchestration · **C**omponent
>
> A universal business logic architecture for Rust.

---

## Latest — v0.2

**[→ Go to v0.2 documentation](./v0.2/index.md)**

What's in v0.2:
- `#[reactor]` macro — Mode A and Mode B, zero boilerplate
- `#[reactor_state]` — automatic derive injection
- `GlocStream<S>` — reactive state stream, pure `std`, zero deps
- `GlocConsumer<R>` — shared mutable handle to a reactor
- `GlocListener<R>` — trait-based transition observer
- `GlocObserver` — global transition interceptor

---

## Version History

| Version | Status | Docs |
|---|---|---|
| **v0.2** | ✅ Latest | [→ v0.2 docs](./v0.2/index.md) |
| v0.1 | ✅ Released | [→ v0.1 docs](./v0.1/index.md) |

---

[CHANGELOG](./CHANGELOG.md) · [crates.io](https://crates.io/crates/gloc) · [docs.rs](https://docs.rs/gloc) · [GitHub](https://github.com/godwinjk/gloc)
