#!/usr/bin/env python3
"""Validate SDK JSON examples against the local SDK manifest metadata."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
SCHEMA_PATH = ROOT / "schemas" / "sdk-manifest.schema.json"
EXAMPLE_DIR = ROOT / "examples"
LABEL_RE = re.compile(r"^[A-Za-z0-9:_.\-/@]+$")
MAX_LABEL_LEN = 192


def load_json(path: Path) -> Any:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def require_object(value: Any, path: str, errors: list[str]) -> dict[str, Any] | None:
    if not isinstance(value, dict):
        errors.append(f"{path}: expected object")
        return None
    return value


def validate_label(value: Any, path: str, errors: list[str], max_len: int = MAX_LABEL_LEN) -> None:
    if not isinstance(value, str) or not value:
        errors.append(f"{path}: expected non-empty string")
        return
    if len(value) > max_len:
        errors.append(f"{path}: label too long")
    if not LABEL_RE.fullmatch(value):
        errors.append(f"{path}: invalid label characters")


def validate_bool(value: Any, path: str, errors: list[str]) -> None:
    if not isinstance(value, bool):
        errors.append(f"{path}: expected boolean")


def validate_enum(value: Any, path: str, allowed: set[str], errors: list[str]) -> None:
    if not isinstance(value, str) or value not in allowed:
        errors.append(f"{path}: unsupported value {value!r}")


def validate_schema(value: Any, path: str, expected: str, errors: list[str]) -> None:
    if value != expected:
        errors.append(f"{path}: expected {expected!r}")


def validate_rights(value: Any, path: str, allowed: set[str], errors: list[str]) -> set[str]:
    if not isinstance(value, list) or not value:
        errors.append(f"{path}: expected non-empty list")
        return set()
    seen: set[str] = set()
    for index, right in enumerate(value):
        item_path = f"{path}[{index}]"
        validate_enum(right, item_path, allowed, errors)
        if isinstance(right, str):
            if right in seen:
                errors.append(f"{item_path}: duplicate right")
            seen.add(right)
    return seen


def validate_common_entry(
    entry: dict[str, Any],
    path: str,
    expected_schema: str,
    allowed_rights: set[str],
    errors: list[str],
) -> set[str]:
    validate_label(entry.get("name"), f"{path}.name", errors, max_len=96)
    validate_schema(entry.get("schema"), f"{path}.schema", expected_schema, errors)
    rights = validate_rights(entry.get("required_rights"), f"{path}.required_rights", allowed_rights, errors)
    validate_bool(entry.get("requires_audit"), f"{path}.requires_audit", errors)
    if entry.get("requires_audit") is True and "audit" not in rights:
        errors.append(f"{path}.required_rights: audit-required entry must include audit")
    return rights


def reject_unknown_keys(
    entry: dict[str, Any], path: str, allowed_keys: set[str], errors: list[str]
) -> None:
    for key in entry:
        if key not in allowed_keys:
            errors.append(f"{path}.{key}: unexpected field")


def validate_cli(entry_value: Any, path: str, metadata: dict[str, Any], errors: list[str]) -> None:
    entry = require_object(entry_value, path, errors)
    if entry is None:
        return
    allowed_keys = {"name", "schema", "kind", "required_rights", "requires_audit"}
    reject_unknown_keys(entry, path, allowed_keys, errors)
    for key in allowed_keys:
        if key not in entry:
            errors.append(f"{path}.{key}: missing required field")
    validate_common_entry(entry, path, metadata["cli_schema"], metadata["rights"], errors)
    validate_enum(entry.get("kind"), f"{path}.kind", metadata["command_kinds"], errors)


def validate_codegen(entry_value: Any, path: str, metadata: dict[str, Any], errors: list[str]) -> None:
    entry = require_object(entry_value, path, errors)
    if entry is None:
        return
    allowed_keys = {
        "name",
        "schema",
        "schema_kind",
        "target",
        "source_schema",
        "output_path",
        "required_rights",
        "requires_audit",
    }
    reject_unknown_keys(entry, path, allowed_keys, errors)
    for key in allowed_keys:
        if key not in entry:
            errors.append(f"{path}.{key}: missing required field")
    validate_common_entry(entry, path, metadata["codegen_schema"], metadata["rights"], errors)
    validate_enum(entry.get("schema_kind"), f"{path}.schema_kind", metadata["schema_kinds"], errors)
    validate_enum(entry.get("target"), f"{path}.target", metadata["targets"], errors)
    validate_label(entry.get("source_schema"), f"{path}.source_schema", errors, max_len=128)
    validate_label(entry.get("output_path"), f"{path}.output_path", errors)


def validate_template(entry_value: Any, path: str, metadata: dict[str, Any], errors: list[str]) -> None:
    entry = require_object(entry_value, path, errors)
    if entry is None:
        return
    allowed_keys = {
        "name",
        "schema",
        "kind",
        "version",
        "path",
        "required_rights",
        "requires_audit",
    }
    reject_unknown_keys(entry, path, allowed_keys, errors)
    for key in allowed_keys:
        if key not in entry:
            errors.append(f"{path}.{key}: missing required field")
    validate_common_entry(entry, path, metadata["templates_schema"], metadata["rights"], errors)
    validate_enum(entry.get("kind"), f"{path}.kind", metadata["template_kinds"], errors)
    validate_label(entry.get("version"), f"{path}.version", errors, max_len=96)
    validate_label(entry.get("path"), f"{path}.path", errors)


def validate_sysroot(entry_value: Any, path: str, metadata: dict[str, Any], errors: list[str]) -> None:
    entry = require_object(entry_value, path, errors)
    if entry is None:
        return
    allowed_keys = {
        "name",
        "schema",
        "target_triple",
        "host",
        "root",
        "required_rights",
        "requires_audit",
    }
    reject_unknown_keys(entry, path, allowed_keys, errors)
    for key in allowed_keys:
        if key not in entry:
            errors.append(f"{path}.{key}: missing required field")
    validate_common_entry(entry, path, metadata["sysroot_schema"], metadata["rights"], errors)
    validate_label(entry.get("target_triple"), f"{path}.target_triple", errors, max_len=96)
    validate_enum(entry.get("host"), f"{path}.host", metadata["host_triples"], errors)
    validate_label(entry.get("root"), f"{path}.root", errors)


def validate_build_helper(
    entry_value: Any, path: str, metadata: dict[str, Any], errors: list[str]
) -> None:
    entry = require_object(entry_value, path, errors)
    if entry is None:
        return
    allowed_keys = {"name", "schema", "kind", "workspace", "required_rights", "requires_audit"}
    reject_unknown_keys(entry, path, allowed_keys, errors)
    for key in allowed_keys:
        if key not in entry:
            errors.append(f"{path}.{key}: missing required field")
    validate_common_entry(entry, path, metadata["build_helper_schema"], metadata["rights"], errors)
    validate_enum(entry.get("kind"), f"{path}.kind", metadata["build_helper_kinds"], errors)
    validate_label(entry.get("workspace"), f"{path}.workspace", errors)


def validate_list(
    value: Any,
    path: str,
    validator: Any,
    metadata: dict[str, Any],
    errors: list[str],
) -> None:
    if not isinstance(value, list) or not value:
        errors.append(f"{path}: expected non-empty list")
        return
    seen: set[str] = set()
    for index, entry in enumerate(value):
        item_path = f"{path}[{index}]"
        if isinstance(entry, dict) and isinstance(entry.get("name"), str):
            if entry["name"] in seen:
                errors.append(f"{item_path}.name: duplicate entry")
            seen.add(entry["name"])
        validator(entry, item_path, metadata, errors)


def main() -> int:
    schema = load_json(SCHEMA_PATH)
    metadata = {
        "manifest_schema": schema.get("x-alani-schema-version"),
        "repository": schema.get("x-alani-repository"),
        "cli_schema": schema.get("x-alani-cli-schema"),
        "codegen_schema": schema.get("x-alani-codegen-schema"),
        "templates_schema": schema.get("x-alani-templates-schema"),
        "sysroot_schema": schema.get("x-alani-sysroot-schema"),
        "build_helper_schema": schema.get("x-alani-build-helper-schema"),
        "modules": set(schema.get("x-alani-modules", [])),
        "rights": set(schema.get("x-alani-rights", [])),
        "command_kinds": set(schema.get("x-alani-command-kinds", [])),
        "build_helper_kinds": set(schema.get("x-alani-build-helper-kinds", [])),
        "schema_kinds": set(schema.get("x-alani-codegen-schema-kinds", [])),
        "targets": set(schema.get("x-alani-codegen-targets", [])),
        "template_kinds": set(schema.get("x-alani-template-kinds", [])),
        "host_triples": set(schema.get("x-alani-host-triples", [])),
    }
    if metadata["manifest_schema"] != "alani.sdk.manifest.v1":
        print("invalid SDK manifest schema metadata", file=sys.stderr)
        return 1
    required_metadata = (
        "repository",
        "cli_schema",
        "codegen_schema",
        "templates_schema",
        "sysroot_schema",
        "build_helper_schema",
    )
    if any(not metadata[key] for key in required_metadata):
        print("SDK schema metadata is incomplete", file=sys.stderr)
        return 1
    set_metadata = (
        "modules",
        "rights",
        "command_kinds",
        "build_helper_kinds",
        "schema_kinds",
        "targets",
        "template_kinds",
        "host_triples",
    )
    if any(not metadata[key] for key in set_metadata):
        print("SDK schema enum metadata is incomplete", file=sys.stderr)
        return 1

    examples = sorted(EXAMPLE_DIR.glob("*.json"))
    if not examples:
        print("no SDK JSON examples found", file=sys.stderr)
        return 1

    errors: list[str] = []
    for example in examples:
        manifest = require_object(load_json(example), example.name, errors)
        if manifest is None:
            continue
        allowed_root = {
            "schema_version",
            "repository",
            "version",
            "modules",
            "cli",
            "codegen",
            "templates",
            "sysroots",
            "build_helpers",
        }
        reject_unknown_keys(manifest, example.name, allowed_root, errors)
        for key in allowed_root:
            if key not in manifest:
                errors.append(f"{example.name}.{key}: missing required field")
        validate_schema(
            manifest.get("schema_version"),
            f"{example.name}.schema_version",
            metadata["manifest_schema"],
            errors,
        )
        validate_schema(
            manifest.get("repository"), f"{example.name}.repository", metadata["repository"], errors
        )
        validate_label(manifest.get("version"), f"{example.name}.version", errors, max_len=32)

        modules = manifest.get("modules")
        if not isinstance(modules, list):
            errors.append(f"{example.name}.modules: expected {sorted(metadata['modules'])}")
        elif any(not isinstance(module, str) for module in modules):
            errors.append(f"{example.name}.modules: expected string module names")
        elif set(modules) != metadata["modules"]:
            errors.append(f"{example.name}.modules: expected {sorted(metadata['modules'])}")

        validate_list(manifest.get("cli"), f"{example.name}.cli", validate_cli, metadata, errors)
        validate_list(
            manifest.get("codegen"),
            f"{example.name}.codegen",
            validate_codegen,
            metadata,
            errors,
        )
        validate_list(
            manifest.get("templates"),
            f"{example.name}.templates",
            validate_template,
            metadata,
            errors,
        )
        validate_list(
            manifest.get("sysroots"),
            f"{example.name}.sysroots",
            validate_sysroot,
            metadata,
            errors,
        )
        validate_list(
            manifest.get("build_helpers"),
            f"{example.name}.build_helpers",
            validate_build_helper,
            metadata,
            errors,
        )

    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1

    print(f"validated {len(examples)} SDK example(s) against {SCHEMA_PATH.name}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
