# alani-sdk

Developer CLI, repository templates, sysroot management, code generation, local build helpers, and compatibility checks.

| Field | Value |
|---|---|
| Status | Experimental Rust skeleton |
| Tier | MVK required |
| Owner | Developer experience team |
| Aliases | None |
| Architectural dependencies | `alani-config`, `alani-protocol`, `alani-docs` |

## Quick Start

```bash
cargo fmt -- --check
cargo test --all-features
cargo test --no-default-features
cargo check --no-default-features
cargo clippy --all-features -- -D warnings
python3 tools/validate_sdk_examples.py
```

## Repository Layout

- `src/cli.rs` owns side-effect-free SDK CLI descriptors, invocations, plans, and command registries.
- `src/codegen.rs` owns schema-to-artifact job contracts and generated artifact metadata.
- `src/templates.rs` owns repository template descriptors, template file records, catalogs, and render plans.
- `src/sysroot.rs` owns sysroot layouts, component plans, host target metadata, and compatibility checks.
- `schemas/sdk-manifest.schema.json` publishes SDK manifest metadata and stable enum labels.
- `examples/sdk-manifest.json` is a checked SDK manifest example covering CLI, codegen, templates, sysroots, and build helpers.
- `tools/validate_sdk_examples.py` validates checked-in examples without external dependencies.

## Public Contracts

- CLI commands use `CliDescriptor`, `CliInvocation`, `CliPlan`, and `CliRegistry`.
- Local build helpers use `BuildHelperDescriptor` and `BuildHelperPlan`; they describe format, check, test, lint, docs, and example-validation helpers without invoking the host shell.
- Code generation uses `CodegenDescriptor`, `CodegenJob`, `CodegenArtifact`, and `CodegenRegistry`.
- Repository templates use `TemplateDescriptor`, `TemplateRecord`, `TemplateCatalog`, and `RenderPlan`.
- Sysroot and compatibility contracts use `SysrootDescriptor`, `SysrootPlan`, `SysrootRegistry`, and `CompatibilityCheck`.

## Feature Flags

- `std` is enabled by default for host-mode tests and the small CLI binary.
- `--no-default-features` builds the library as `no_std`.

## Security And Observability Notes

SDK APIs are planning contracts, not filesystem or process execution implementations. Security-sensitive operations fail closed through explicit `SdkRights`. Template rendering, sysroot modification, code generation, compatibility checks, and release-evidence-affecting operations can be marked audit-required. Public APIs carry `TraceContext`, `DataClass`, and `RedactionState` so developer tooling can propagate observability and redaction metadata without depending on sibling private modules.

## Troubleshooting

- `reserved_bits` means a caller supplied unknown rights, feature, or trace flag bits.
- `access_denied` means a CLI command, codegen job, template, or sysroot plan requires rights the caller does not hold.
- `audit_required` means the operation requires audit authority and an audit-ready sink.
- `sensitive_data` means a sensitive or secret template or generated artifact would be exposed without redaction.

Keep public API changes synchronized with `alani-spec/docs/repositories/alani-sdk.md`, Doc 42, and Doc 43.
