# opd_parser

 ![crates.io](https://img.shields.io/crates/v/opd-parser.svg)

Parser for the OPD point cloud animation format.

## CI

Prow presubmits and postsubmits, configured in
`infra/modules/ci/prow/terragrunt.hcl` under `local.repos_config.opd_parser`.

| Job | Trigger | What it runs |
|-----|---------|--------------|
| `cargo-tests` | every PR | `fslabscli rust-tests` (fmt, clippy, check, doc, test) |
| `bazel-tests` | `/test bazel-tests` | `bazel test --config=ci //...` on Buildbarn, optional |
| `publish-all` | push to `master` | `fslabscli publish --autopublish-cargo` to `fsl` and crates.io |

Bazel and cargo are both first-class here: `cargo test` and `bazel test //...`
build the same crate from the same `Cargo.toml`, and `Cargo.lock` is committed
because crate_universe resolves the Bazel crate graph from it.
