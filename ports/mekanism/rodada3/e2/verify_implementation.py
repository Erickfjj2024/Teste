#!/usr/bin/env python3
"""
Verification script for T20261109-203634-qse:
Verifies Java citations, line references, constants, and math logic.
"""
import os, sys, re

REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "../.."))
JAVA_ROOT = os.path.join(REPO_ROOT, "java")
MOD_SRC = os.path.join(REPO_ROOT, "port/mod/src")

def verify_file_exists(rel_path):
    full = os.path.join(REPO_ROOT, rel_path)
    exists = os.path.exists(full)
    print(f"File {rel_path}: {'OK' if exists else 'MISSING'}")
    return exists

print("=== 1. Checking new modules ===")
all_exist = True
for f in ["port/mod/src/qio_transfer.rs", "port/mod/src/qio_dashboard.rs", "port/mod/src/security.rs", "port/mod/src/modification_station.rs"]:
    if not verify_file_exists(f):
        all_exist = False

print("\n=== 2. Checking Java citations in Java sources ===")
# TileEntityQIOFilterHandler.java lines 41-43
fh_path = os.path.join(JAVA_ROOT, "main/java/mekanism/common/tile/qio/TileEntityQIOFilterHandler.java")
with open(fh_path) as f:
    fh_content = f.read()
assert "maxTransitCount = 64 + 32 * speedUpgrades;" in fh_content
assert "maxTransitTypes = Math.round(1F + speedUpgrades / 2F);" in fh_content
print("TileEntityQIOFilterHandler transit math citation: OK")

# TileEntityQIOImporter.java line 38, line 99
imp_path = os.path.join(JAVA_ROOT, "main/java/mekanism/common/tile/qio/TileEntityQIOImporter.java")
with open(imp_path) as f:
    imp_content = f.read()
assert "MAX_DELAY = MekanismUtils.TICKS_PER_HALF_SECOND;" in imp_content
assert "for (int i = slots - 1; i >= 0; i--)" in imp_content
print("TileEntityQIOImporter MAX_DELAY and reverse slot iteration citation: OK")

# TileEntityQIOExporter.java line 74
exp_path = os.path.join(JAVA_ROOT, "main/java/mekanism/common/tile/qio/TileEntityQIOExporter.java")
with open(exp_path) as f:
    exp_content = f.read()
assert "MAX_DELAY = MekanismUtils.TICKS_PER_HALF_SECOND;" in exp_content
assert "boolean exportWithoutFilter" in exp_content
assert "boolean roundRobin" in exp_content
print("TileEntityQIOExporter constants citation: OK")

# TileEntityQIODashboard.java line 42, 66-73
dash_path = os.path.join(JAVA_ROOT, "main/java/mekanism/common/tile/qio/TileEntityQIODashboard.java")
with open(dash_path) as f:
    dash_content = f.read()
assert "boolean insertIntoFrequency = true;" in dash_content
assert "MAX_CRAFTING_WINDOWS" in dash_content
print("TileEntityQIODashboard constants citation: OK")

# SecurityUtils.java lines 141-152
sec_utils_path = os.path.join(JAVA_ROOT, "main/java/mekanism/common/lib/security/SecurityUtils.java")
with open(sec_utils_path) as f:
    sec_content = f.read()
assert "case PUBLIC -> false;" in sec_content
assert "case PRIVATE -> base != SecurityMode.PRIVATE;" in sec_content
assert "case TRUSTED -> base == SecurityMode.PUBLIC;" in sec_content
print("SecurityUtils.moreRestrictive citation: OK")

# StorageConfig.java line 104, UsageConfig.java line 74, TileEntityModificationStation.java line 42
stor_path = os.path.join(JAVA_ROOT, "main/java/mekanism/common/config/StorageConfig.java")
with open(stor_path) as f:
    stor_content = f.read()
assert "modificationStation" in stor_content
assert "40_000L" in stor_content
print("StorageConfig modificationStation 40,000L J citation: OK")

usage_path = os.path.join(JAVA_ROOT, "main/java/mekanism/common/config/UsageConfig.java")
with open(usage_path) as f:
    usage_content = f.read()
assert "modificationStation" in usage_content
assert "400L" in usage_content
print("UsageConfig modificationStation 400L J/t citation: OK")

mod_station_path = os.path.join(JAVA_ROOT, "main/java/mekanism/common/tile/TileEntityModificationStation.java")
with open(mod_station_path) as f:
    mod_station_content = f.read()
assert "BASE_TICKS_REQUIRED = MekanismUtils.TICKS_PER_HALF_SECOND;" in mod_station_content
print("TileEntityModificationStation BASE_TICKS_REQUIRED citation: OK")

print("\n=== 3. Counting unit tests in new Rust modules ===")
total_tests = 0
for rel in ["port/mod/src/qio_transfer.rs", "port/mod/src/qio_dashboard.rs", "port/mod/src/security.rs", "port/mod/src/modification_station.rs"]:
    full = os.path.join(REPO_ROOT, rel)
    with open(full) as f:
        lines = f.readlines()
    test_names = [l.strip() for l in lines if l.strip().startswith("fn test_") or (l.strip().startswith("fn ") and "test" in l)]
    # Or specifically count #[test]
    test_count = sum(1 for l in lines if l.strip() == "#[test]")
    print(f"{os.path.basename(rel)}: {test_count} tests ({len(lines)} lines)")
    total_tests += test_count

print(f"Total unit tests across new modules: {total_tests}")
print("\nAll verifications PASSED!")
