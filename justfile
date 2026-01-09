manifest := "src/rust/Cargo.toml"

# Run any cargo subcommand against the workspace manifest.
cargo subcmd *args:
    cargo {{subcmd}} --manifest-path={{manifest}} {{args}}

# Sync or bump versions from DESCRIPTION.
# Examples:
#   just bump-version
#   just bump-version -- --bump=patch
#   just bump-version -- --bump=dev
#   just bump-version -- --bump=dev+
#   just bump-version -- --set=0.1.0.9000
#   just bump-version -- --mode=workspace
bump-version *args:
    Rscript tools/bump-version.R {{args}}
