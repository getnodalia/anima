# Working headless

Everything you do in Ánima is a named command, and the same commands run without a window.
This chapter is for technical artists and pipeline TDs: rendering from the command line, batch
jobs, scripting edits as JSON, and connecting AI assistants.

On macOS, press ⌘ where this guide says Ctrl, and ⌥ where it says Alt.

## The command-line tool

`anima-cli` comes with Ánima:

* **Windows**: `anima-cli.exe` sits next to `anima.exe` in the install folder. It isn't
  added to your `PATH`, so call it by its full path or add the folder yourself.
* **macOS**: it is inside the app at `Anima.app/Contents/MacOS/anima-cli`, and also a
  separate download, `anima-cli-<version>-macos-universal.zip`, on the
  [releases page](https://github.com/getnodalia/anima/releases).
* **Linux**: the deb and rpm packages install it as `/usr/bin/anima-cli`; the tarball has it
  next to `anima`.

Run it with no arguments, or with `--help`, to see every subcommand. `anima-cli --version`
prints the version.

Two rules hold everywhere:

* Subcommands that open a document accept a `.anima` file, an Animate `.fla` or `.xfl`
  file, an XFL folder, or `--sample` for the built-in sample animation. See
  [Converting Animate files](#converting-animate-files) to save an import as `.anima`.
* **Frame flags on the command line are 1-based**, as on the timeline: `--frame 1` is the first
  frame. Frame numbers inside JSON params are 0-based, as everywhere in the engine.

## Rendering

| Subcommand | What it writes |
|---|---|
| `info <file>` | A JSON description of the document: stage, scenes, layers, keyframes, library. |
| `render <file> --out frame.png [--frame N] [--scale S]` | One frame as a PNG. |
| `render-seq <file> --out-dir dir [--from A] [--to B] [--scale S]` | A numbered PNG sequence. |
| `gif <file> --out anim.gif [--from A] [--to B] [--scale S] [--overwrite yes]` | An animated GIF. |
| `apng <file> --out anim.png [...]` | An animated PNG, full colour with real transparency. |
| `webp <file> --out anim.webp [...]` | A lossless animated WebP. |
| `new --out file.anima [--width W] [--height H] [--fps F]` | A new, empty document. |

`apng` and `webp` take the same range and scale flags as `gif`, plus `--loop forever` (the
default) or `--loop N` for how many times it plays, and `--background yes` to draw the stage
colour instead of transparency.

Every frame is drawn by the same renderer the app uses for its exports, so what you render here
matches File › Export exactly.

```sh
anima-cli render shot010.anima --frame 24 --out shot010_f24.png
anima-cli render-seq shot010.anima --out-dir frames/shot010 --scale 0.5
anima-cli gif --sample --out preview.gif --scale 0.5
anima-cli webp shot010.anima --out loop.webp --loop 3
```

### Replacing files

Exports never overwrite a file silently. If the output already exists, the command fails and
says so ("Replace existing file? f.png already exists…"). Pass `--overwrite yes` to `gif`,
`apng` and `webp` when replacing is what you want. `render` has no such flag: delete the old file
first, or run the export command itself with `"confirm": true` (see
[Destructive commands need "confirm"](#destructive-commands-need-confirm)):

```sh
anima-cli exec shot010.anima file.export_png '{"path":"f24.png","frame":23,"confirm":true}'
```

## Running any command

Every menu item is a command with an id, and every command can be run from the command line.

`anima-cli commands` prints the whole registry as JSON: each command's `id`, `label`, `menu`,
default `shortcut`, a short `params` description, and whether it changes the document. The same
list, readable, is in [the command reference](reference/commands.md).

The `menu` field tells you where the command lives in the app. `"File/Export"` is
**File › Export**, `"Modify/Transform"` is **Modify › Transform**, and an empty menu means the
command has no menu item (it is used by panels, tools or scripts). For example,
`timeline.insert_keyframe` is **Insert › Timeline › Keyframe** (**F6**), and `file.export_html`
is **File › Export › Export HTML Canvas…**.

### One command: exec

```sh
anima-cli exec <file|--sample> <command.id> ['<json-params>'] [--save out.anima]
```

The result prints as JSON. Add `--save` to write the changed document.

```sh
anima-cli exec --sample draw.rect '{"x":10,"y":10,"w":80,"h":40}' --save box.anima
anima-cli exec shot010.anima file.export_html '{"path":"web/shot010.html"}'
anima-cli exec shot010.anima file.export_video '{"path":"shot010.mp4"}'
```

Video export streams frames to your own copy of ffmpeg. Ánima looks for it in the `ffmpeg`
param, then the `ANIMA_FFMPEG` environment variable, then your `PATH`.

On Windows PowerShell, quoting JSON on the command line is awkward. Put the steps in a script
file instead (next section).

### Several commands: run

A script is a JSON array of steps, each with a `cmd` and optional `params`:

```json
[
  {"cmd": "layer.new", "params": {"name": "Title"}},
  {"cmd": "draw.rect", "params": {"x": 40, "y": 40, "w": 400, "h": 120, "fill": "#dac604"}},
  {"cmd": "timeline.insert_keyframe", "params": {"frame": 23}},
  {"cmd": "file.export_gif", "params": {"path": "title.gif"}}
]
```

```sh
anima-cli run shot010.anima title.json --save shot010_titled.anima
```

Steps run in order. The first one that fails stops the run, and the error names the step. The
results of all the steps print as a JSON array.

### Publishing a document

If a document has **File › Publish Settings** set up, one command writes every format it has
switched on, to the output names saved in the document (relative to the document's folder):

```sh
anima-cli exec shot010.anima file.publish '{"confirm": true}'
```

`confirm` lets it replace files left by an earlier publish. The result is the same report the
app shows: what each format wrote, skipped or why it failed.

## Batch jobs for pipelines

Because each call is a plain process with JSON in and out, Ánima fits into any build system,
render farm or shell loop. Exit status is 0 on success and 1 on any error, with the message on
standard error, so a failed step stops your pipeline.

Render every document in a folder to a half-size GIF:

```sh
for f in shots/*.anima; do
  anima-cli gif "$f" --out "out/$(basename "$f" .anima).gif" --scale 0.5 --overwrite yes
done
```

The same in PowerShell:

```powershell
Get-ChildItem shots\*.anima | ForEach-Object {
  & anima-cli gif $_.FullName --out "out\$($_.BaseName).gif" --scale 0.5 --overwrite yes
}
```

Renders are deterministic: the same document and flags give the same pixels on every machine.

### Converting Animate files

Pass a `.fla`, `.xfl` file or XFL folder directly to any subcommand that opens a document:

```sh
anima-cli info intro.fla
```

To save an import as `.anima` and print its import report, use `file.open` and `--save`:

```sh
anima-cli exec --sample file.open '{"path":"intro.fla"}' --save intro.anima
```

The result includes what was imported, kept or left out, as the app's import report does. See
[Opening Animate files](opening-animate-files.md) for what comes across.

### Plug-ins on the command line

Headless runs load no plug-ins unless you ask, so a script gives the same result on every
machine. Add `--plugins <folder>` to any subcommand (or set `ANIMA_PLUGINS`) to load the
`.wasm` files in that folder first, then run them with `plugin.run`:

```sh
anima-cli exec --sample plugin.run '{"id":"org.anima.star-field","params":{"count":200}}' \
    --plugins my-plugins --save stars.anima
```

See [Plug-ins](plugins.md).

## A live session: serve

`anima-cli serve [file|--sample]` keeps one document open and reads one JSON request per
line on standard input, answering one line each on standard output. Tools that drive Ánima
interactively (a game engine exporter, a custom panel in another app) use it.

```text
{"id": 1, "cmd": "timeline.insert_keyframe", "params": {"frame": 10}}
{"id": 1, "ok": true, "result": {}, "revision": 7, "frame": 0}
```

The first line out is `{"ready": true, ...}`. A request with `"cmd": "commands"` returns the
registry, a failed command answers `"ok": false` with an `error`, and `{"cmd": "quit"}` ends the
session. Undo and redo are commands too (`edit.undo`, `edit.redo`).

## Destructive commands need "confirm"

In the app, actions that would lose work for good ask first: discarding unsaved changes,
replacing a file, deleting an autosaved copy or an installed plug-in. Headless, there is nobody
to ask, so those commands refuse until their params include `"confirm": true`:

```text
error: step 2: file.new: Discard unsaved changes? “Untitled” has changes that are not saved.
They will be lost, and its undo history is cleared. (pass "confirm": true to proceed)
```

Add `"confirm": true` only when you mean it. Ordinary edits, such as deleting a layer, never need
it: they can be undone with `edit.undo` in the same session.

## AI assistants (MCP)

`anima-cli mcp` is a Model Context Protocol server. An AI assistant that supports MCP can
then open a document, draw, animate, look at frames and export, through the same commands the
app uses.

```sh
anima-cli mcp                    # a new, empty document
anima-cli mcp scene.anima     # open a document
anima-cli mcp --sample           # the sample animation
```

To register it with Claude Code:

```sh
claude mcp add anima -- /path/to/anima-cli mcp
```

Most other clients take an entry like this (use the full path to `anima-cli`):

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

The assistant gets five tools:

| Tool | What it does |
|---|---|
| `anima_commands` | Lists commands, optionally filtered by words such as `tween` or `export`. |
| `anima_run` | Runs one command with its params. |
| `anima_run_script` | Runs several commands in order, stopping at the first error. |
| `anima_document` | Describes the document: stage, scenes, layers, keyframes, library, selection. |
| `anima_render` | Returns a frame as a PNG picture, exactly as exported. |

Things to know:

* Every edit is checked and can be undone, as in the app.
* Commands that would lose work or replace a file still need `"confirm": true`. The server tells
  the assistant to check with you before confirming.
* The assistant reads and writes files with your permissions, at the paths it chooses. Only
  connect assistants you trust with those files.
* The server works on its own copy of the document, not on the window you have open. Save from
  the assistant, then open the file in Ánima. To have the assistant work in the window
  instead, see the next section.

### An assistant in the window you have open

Start Ánima with `anima --control 9878`, and give the assistant
`anima-cli mcp --bridge 9878` as its server instead of `anima-cli mcp`. The same five
tools then act on the document in front of you: each edit shows at once and undoes with
**Ctrl+Z** like your own. While this is on, the status bar says **Remote control on (port
9878)**.

* Only programs on this computer, running as you, can connect: the port is not open to the
  network, and each run has its own secret token, kept in the settings folder until Ánima
  closes.
* An edit from the assistant waits while you are in the middle of a drag, so your undo steps
  and its own never mix.
* Commands that would lose work still need `"confirm": true`, as above.
* Without `--control` nothing listens: remote control is off unless you start Ánima this way.

The full details are in
[the MCP notes on GitHub](https://github.com/getnodalia/anima/blob/main/docs/mcp.md).

## This guide as a web page

`anima-cli guide --out <folder>` writes this user guide as HTML pages into a folder, with its
pictures, so you can read it offline or host it next to your studio's own documentation:

```sh
anima-cli guide --out docs/anima-guide
```

`anima-cli guide search <words>` searches it from the terminal, with the same results, in the
same order, as the search field in the guide's pages:

```sh
anima-cli guide search onion skin
anima-cli guide search export gif --limit 3 --json yes
```

Each result gives the section's heading, its chapter, the page to open and a passage around the
first match. See [Using this guide](using-this-guide.md).

## Performance check

`anima-cli bench` builds a large generated document and times editing, undo, rendering,
saving and autosave on your machine. Run it on a release build; `bench --check yes` compares the
default scenario with Ánima's published budgets and fails if one is missed.
