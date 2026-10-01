#!/usr/bin/env python3
"""Run UI regression tests from an isolated cwd (the app opens a cwd-local DB)."""
import json
import pathlib
import subprocess
import tempfile

root = pathlib.Path(__file__).resolve().parent.parent
subprocess.run(["cargo", "test", "-p", "cabinet", "--lib"], cwd=root, check=True)
build = subprocess.run(
    ["cargo", "test", "-p", "tdrace-app", "--test", "controls_ui_tests",
     "--test", "modality_flow_tests", "--test", "ui_table_migration_tests",
     "--no-run", "--message-format=json"],
    cwd=root, check=True, text=True, stdout=subprocess.PIPE,
)
executables = []
for line in build.stdout.splitlines():
    artifact = json.loads(line)
    if artifact.get("reason") == "compiler-artifact" and artifact.get("executable") and artifact.get("profile", {}).get("test"):
        executables.append(artifact["executable"])
assert len(executables) == 3, f"Expected three test binaries, got {executables}"
with tempfile.TemporaryDirectory(prefix="ui-migration-") as directory:
    for executable in executables:
        subprocess.run([executable, "--test-threads=1"], cwd=directory, check=True)
