# rigid-body
rapier f64 rigid body by ffm api or jni

The Rust crate exposes both Java access paths from the same `cdylib`:

- `src/abi/ffm.rs`: C ABI metadata for Java FFM and other C callers.
- `src/abi/jni.rs`: JNI-compatible `Java_*` wrappers that call the existing
  `rc_*` C ABI implementation.

Smoke test projects:

- `test21`: Gradle Java 21 JNI smoke test. Run with `gradle -p test21 check`.
- `test25`: Gradle Java 25 FFM smoke test. Run with `gradle -p test25 check`.
