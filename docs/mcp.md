# Ánima for AI assistants (MCP)

`anima-cli mcp` is a [Model Context Protocol](https://modelcontextprotocol.io) server. Any
MCP client (an AI assistant, an editor, a script) can then open a document, draw, animate,
look at frames and export through the same commands the app uses.

```sh
anima-cli mcp                      # a new, empty document
anima-cli mcp scene.anima       # open a document
anima-cli mcp --sample             # the sample animation
```

The server talks MCP's stdio transport: JSON-RPC 2.0, one message per line on stdin and stdout.
It speaks protocol revisions 2025-06-18, 2025-03-26 and 2024-11-05.

## Connecting a client

Most clients take a server entry like this one (use the full path to `anima-cli`):

```json
{
  "mcpServers": {
    "anima": {
      "command": "/path/to/anima-cli",
      "args": ["mcp", "/path/to/scene.anima"]
    }
  }
}
```

## Tools

| Tool | What it does |
|---|---|
| `anima_commands` | Lists the commands (id, label, menu, shortcut, params), optionally filtered by words such as `tween` or `export`. |
| `anima_run` | Runs one command with its params. |
| `anima_run_script` | Runs several commands in order; stops at the first error and says which step failed. |
| `anima_document` | Describes the document: stage, scenes, layers and keyframes, library, and the current scene, frame, layer and selection. |
| `anima_render` | Returns a frame of the current scene as a PNG picture, exactly as exported. |

The assistant sees one tool per job, not one per command: the command list is long and grows
with every release, and `anima_commands` lets it find what it needs. Frames in params are
0-based, as everywhere in the engine (the CLI's own `--frame` flags are 1-based, as in the
timeline).

## Safety

* Everything runs through the engine's commands, so params are checked and every edit can be
  undone with `edit.undo`.
* Commands that would lose unsaved work or replace a file refuse until their params include
  `"confirm": true` (ADR 0004). The server's instructions ask the assistant to check with you
  before confirming.
* Commands read and write files with your permissions, at the paths the assistant gives (open,
  save, import, export), as with the CLI. Only connect clients you trust with those files.
* A failing command comes back as a tool result marked as an error, so the assistant can read
  the message and try again; it never stops the server.
