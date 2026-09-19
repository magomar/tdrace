#!/usr/bin/env python3
import re
import sys
from pathlib import Path

def verify_docs(docs_dir: Path) -> int:
    files = sorted(list(docs_dir.rglob("*.md")))
    print(f"🔍 Checking OKF v0.2 compliance across {len(files)} Markdown files in {docs_dir}...\n")

    errors = []
    link_count = 0
    doc_count = 0

    for file_path in files:
        doc_count += 1
        content = file_path.read_text(encoding="utf-8")

        # 1. Check YAML frontmatter presence
        if not content.startswith("---"):
            errors.append(f"[{file_path.relative_to(docs_dir)}] Missing YAML frontmatter start ('---')")
            continue

        parts = content.split("---", 2)
        if len(parts) < 3:
            errors.append(f"[{file_path.relative_to(docs_dir)}] Unclosed YAML frontmatter ('---')")
            continue

        frontmatter = parts[1]
        for req_field in ["type:", "title:", "description:", "status:"]:
            if req_field not in frontmatter:
                errors.append(f"[{file_path.relative_to(docs_dir)}] Missing required field '{req_field}' in frontmatter")

        # Check index.md has okf_version: "0.2"
        if file_path.name == "index.md" and file_path.parent == docs_dir:
            if 'okf_version: "0.2"' not in frontmatter and "okf_version: '0.2'" not in frontmatter:
                errors.append(f"[{file_path.relative_to(docs_dir)}] Root index.md missing okf_version: '0.2'")

        # 2. Check relative markdown links
        # Ignore backtick-wrapped links (code examples)
        clean_content = re.sub(r'`[^`]+`', '', content)
        clean_content = re.sub(r'```[\s\S]*?```', '', clean_content)
        links = re.findall(r'\[([^\]]+)\]\(([^)]+)\)', clean_content)

        for link_text, link_target in links:
            # Skip external links and anchors-only
            if link_target.startswith(("http://", "https://", "mailto:", "file://", "ftp://")):
                continue
            if link_target.startswith("#"):
                continue

            link_count += 1
            target_path_str = link_target.split("#")[0]
            if not target_path_str:
                continue

            target_resolved = (file_path.parent / target_path_str).resolve()
            if not target_resolved.exists():
                errors.append(f"[{file_path.relative_to(docs_dir)}] Broken relative link: '{link_target}' -> '{target_resolved}' not found")

    print(f"📊 Summary:")
    print(f"  - Total OKF Documents: {doc_count}")
    print(f"  - Verified Relative Links: {link_count}")

    if errors:
        print(f"\n❌ FAILED with {len(errors)} error(s):")
        for err in errors:
            print(f"  • {err}")
        return 1

    print("\n✅ SUCCESS: All OKF v0.2 documents, frontmatter schemas, and cross-links are 100% valid!")
    return 0

if __name__ == "__main__":
    base_dir = Path(__file__).resolve().parent.parent
    target_docs = base_dir / "docs"
    sys.exit(verify_docs(target_docs))
