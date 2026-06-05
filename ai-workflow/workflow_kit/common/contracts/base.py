"""Base types and shared constants for output contracts."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any

COMMON_REQUIRED_KEYS = frozenset({"status", "tool_version", "warnings"})

@dataclass(frozen=True)
class OutputFieldShape:
    kind: str
    item_kind: str | None = None
    required_keys: frozenset[str] = frozenset()
    allow_null: bool = False
    properties: dict[str, "OutputFieldShape"] = field(default_factory=dict)
    item_properties: dict[str, "OutputFieldShape"] = field(default_factory=dict)

def output_field_shape_to_schema(shape: OutputFieldShape) -> dict[str, object]:
    result: dict[str, object] = {
        "kind": shape.kind,
        "item_kind": shape.item_kind,
        "required_keys": sorted(shape.required_keys),
        "allow_null": shape.allow_null,
    }
    if shape.properties:
        result["properties"] = {
            key: output_field_shape_to_schema(value)
            for key, value in sorted(shape.properties.items())
        }
    if shape.item_properties:
        result["item_properties"] = {
            key: output_field_shape_to_schema(value)
            for key, value in sorted(shape.item_properties.items())
        }
    return result

def json_schema_for_shape(shape: OutputFieldShape) -> dict[str, object]:
    if shape.kind == "string":
        type_value: object = ["string", "null"] if shape.allow_null else "string"
        return {"type": type_value}
    if shape.kind == "boolean":
        type_value = ["boolean", "null"] if shape.allow_null else "boolean"
        return {"type": type_value}
    if shape.kind == "integer":
        type_value = ["integer", "null"] if shape.allow_null else "integer"
        return {"type": type_value}
    if shape.kind == "list":
        item_schema: dict[str, object] = {}
        if shape.item_kind == "string":
            item_schema = {"type": "string"}
        elif shape.item_kind == "object":
            item_schema = {
                "type": "object",
                "required": sorted(shape.required_keys),
                "properties": {
                    key: json_schema_for_shape(value)
                    for key, value in sorted(shape.item_properties.items())
                }
            }
        return {"type": "array", "items": item_schema}
    if shape.kind == "object":
        return {
            "type": "object",
            "required": sorted(shape.required_keys),
            "properties": {
                key: json_schema_for_shape(value)
                for key, value in sorted(shape.properties.items())
            }
        }
    return {}
