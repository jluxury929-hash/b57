# We add 'cargo clean' to delete old v33 artifacts before building v25
RUN cargo clean && cargo build --release
