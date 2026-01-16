# 1. We use the latest Rust version to ensure Alloy 1.4 works
FROM rust:latest

# 2. Set the working directory inside the container
WORKDIR /app

# 3. Copy all your files into the container
COPY . .

# 4. THE NUCLEAR BUILD COMMAND
# We run 'cargo clean' first to banish any old v33/v21 conflict artifacts.
# Then we build the release binary.
RUN cargo clean && cargo build --release

# 5. The command to start your bot
CMD ["./target/release/apex-singularity"]
