# PyInstaller spec for agent-web-search single-file executable.
#
# Packs the Python interpreter + agent_web_search + ddgs + httpx + readability
# into one standalone binary. Users download one file and run it — no Python
# install needed (ADR-0006).
#
# Build: pyinstaller agent-web-search.spec --noconfirm
# Verified working: ddgs search returns results through the bundled exe.

from PyInstaller.utils.hooks import collect_submodules, copy_metadata

block_cipher = None

# Collect all submodules of deps that use dynamic imports / lazy loading,
# so PyInstaller bundles them rather than missing them at runtime.
hiddenimports = []
hiddenimports += collect_submodules("ddgs")
hiddenimports += collect_submodules("httpx")
hiddenimports += collect_submodules("anyio")
hiddenimports += ["readability", "lxml", "lxml.html", "cssselect"]
# mcp server modules we actually use (NOT mcp.cli — it sys.exits on import).
hiddenimports += [
    "mcp",
    "mcp.server",
    "mcp.server.stdio",
    "mcp.server.models",
    "mcp.server.session",
    "mcp.server.lowlevel",
    "mcp.types",
    "mcp.shared",
    "mcp.shared.session",
    "mcp.shared.exceptions",
    "pydantic",
    "pydantic.fields",
    "sse_starlette",
]

# Bundle our own dist-info so importlib.metadata.version() resolves at runtime.
# __init__.py reads __version__ from package metadata; without this the frozen
# binary would hit PackageNotFoundError and silently fall back to "0.0.0+dev".
datas = copy_metadata("agent-web-search")

# Excluded — only things known-safe to drop. (mcp.cli sys.exits on import;
# the rest are unused heavy stdlib. Do NOT exclude xml/email — pkg_resources
# and plistlib depend on them.)
excludes = [
    "mcp.cli",
    "tkinter",
    "pydoc",
]

a = Analysis(
    ["src/agent_web_search/__main__.py"],
    pathex=["src"],
    binaries=[],
    datas=datas,
    hiddenimports=hiddenimports,
    hookspath=[],
    runtime_hooks=[],
    excludes=excludes,
    cipher=block_cipher,
)

pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.zipfiles,
    a.datas,
    [],
    name="agent-web-search",
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    upx_exclude=[],
    runtime_tmpdir=None,
    console=True,
    disable_windowed_traceback=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)
