import argparse
import sys
from pathlib import Path
from typing import Literal, cast
import tomlkit
from tomlkit.items import Table


BumpType = Literal["major", "minor", "patch"]


def parse_version(version: str) -> tuple[int, int, int]:
    """Parse a semantic version string into major, minor, patch components."""
    parts = version.split(".")
    if len(parts) != 3:
        raise ValueError(f"Invalid version format: {version}")

    try:
        major, minor, patch = int(parts[0]), int(parts[1]), int(parts[2])
        return major, minor, patch
    except ValueError as e:
        raise ValueError(f"Invalid version format: {version}") from e


def bump_version(version: str, bump_type: BumpType) -> str:
    """
    Bump a semantic version according to the bump type.

    Args:
        version: Current version string (e.g., "1.2.3")
        bump_type: Type of bump - "major", "minor", or "patch"

    Returns:
        New version string
    """
    major, minor, patch = parse_version(version)

    if bump_type == "major":
        return f"{major + 1}.0.0"
    elif bump_type == "minor":
        return f"{major}.{minor + 1}.0"
    else:
        return f"{major}.{minor}.{patch + 1}"


def bump_crate_version(
    cargo_toml_path: Path, bump_type: BumpType, dry_run: bool = False
) -> tuple[str, str]:
    """
    Bump the version in a Cargo.toml file.

    Args:
        cargo_toml_path: Path to the Cargo.toml file
        bump_type: Type of version bump
        dry_run: If True, don't write changes

    Returns:
        Tuple of (old_version, new_version)
    """
    with open(cargo_toml_path, "r", encoding="utf-8") as f:
        doc = tomlkit.parse(f.read())

    package_item = doc.get("package")
    if package_item is None or not isinstance(package_item, Table):
        raise ValueError(f"No [package] table found in {cargo_toml_path}")

    package = cast(Table, package_item)
    version_item = package.get("version")

    if version_item is None:
        raise ValueError(f"No package.version found in {cargo_toml_path}")

    old_version = str(version_item)
    new_version = bump_version(old_version, bump_type)

    if not dry_run:
        package["version"] = new_version
        with open(cargo_toml_path, "w", encoding="utf-8") as f:
            f.write(tomlkit.dumps(doc))

    return old_version, new_version


def find_crate_manifests(crates_dir: Path) -> list[Path]:
    """
    Find all Cargo.toml files in the crates directory.

    Args:
        crates_dir: Path to the crates directory

    Returns:
        List of paths to Cargo.toml files
    """
    if not crates_dir.exists():
        raise FileNotFoundError(f"Crates directory not found: {crates_dir}")

    if not crates_dir.is_dir():
        raise NotADirectoryError(f"Not a directory: {crates_dir}")

    manifests = []
    for crate_dir in crates_dir.iterdir():
        if crate_dir.is_dir():
            cargo_toml = crate_dir / "Cargo.toml"
            if cargo_toml.exists():
                manifests.append(cargo_toml)

    return sorted(manifests)


def main() -> int:
    parser = argparse.ArgumentParser(
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s patch              # Bump patch version (0.1.0 -> 0.1.1)
  %(prog)s minor              # Bump minor version (0.1.0 -> 0.2.0)
  %(prog)s major              # Bump major version (0.1.0 -> 1.0.0)
  %(prog)s patch --dry-run    # Show what would be changed
  %(prog)s patch --crates-dir ./my-crates
        """,
    )

    parser.add_argument(
        "bump_type",
        choices=["major", "minor", "patch"],
        help="Type of version bump to perform",
    )

    parser.add_argument(
        "--crates-dir",
        type=Path,
        default=Path("crates"),
        help="Path to the crates directory (default: ./crates)",
    )

    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would be changed without making changes",
    )

    args = parser.parse_args()

    try:
        manifests = find_crate_manifests(args.crates_dir)

        if not manifests:
            print(f"No crates found in {args.crates_dir}")
            return 0

        print(f"Found {len(manifests)} crate(s) in {args.crates_dir}")

        if args.dry_run:
            print("\n[DRY RUN - No changes will be made]\n")

        success_count = 0
        for manifest in manifests:
            crate_name = manifest.parent.name
            try:
                old_version, new_version = bump_crate_version(
                    manifest, args.bump_type, dry_run=args.dry_run
                )
                status = "[DRY RUN]" if args.dry_run else "✓"
                print(f"{status} {crate_name}: {old_version} -> {new_version}")
                success_count += 1
            except Exception as e:
                print(f"{crate_name}: Error - {e}", file=sys.stderr)

        print(f"\n{success_count}/{len(manifests)} crate(s) processed successfully")

        if args.dry_run:
            print("\nRun without --dry-run to apply changes")

        return 0 if success_count == len(manifests) else 1

    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
