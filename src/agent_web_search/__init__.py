"""agent-web-search: a free, unlimited, stable web-search MCP tool."""

from importlib.metadata import PackageNotFoundError, version

try:
    # Single source of truth: pyproject.toml's [project] version. Reading it
    # via importlib.metadata means changing pyproject.toml is the only edit
    # needed at release — __version__, the MCP serverInfo.version, and the
    # PyInstaller-built binary all follow automatically once installed.
    __version__ = version("agent-web-search")
except PackageNotFoundError:
    # Not installed (e.g. running from a source checkout without `pip install
    # -e .`). Fall back rather than crash — tests and dev workflows that don't
    # care about the version still work.
    __version__ = "0.0.0+dev"
