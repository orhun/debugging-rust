# Graham Scan TUI

A minimal, deliberately buggy Graham scan based on
[matrohin/gav](https://github.com/matrohin/gav). The original project visualizes
geometry algorithms in a graphical window; this version keeps the step-by-step
model and renders it with Ratatui's built-in terminal widgets.

The program treats ten survey markers as points and tries to build the smallest
fence containing all of them. One boundary case is wrong: the final computed
fence cuts off an extreme marker.

## Run the visualization

```console
cargo run
```

Controls:

- `Left`/`Right` or `h`/`l`: move one algorithm step
- `Enter`: start, pause, or resume
- `r`: reset
- `q`: quit

The green outline is the expected perimeter. The cyan path is the hull currently
being constructed; it turns red when the incorrect final result is shown.

## Record the visualization

The demo is recorded with [VHS](https://github.com/charmbracelet/vhs):

```console
vhs demo.tape
```

The tape starts the application, waits for the TUI to appear, and presses
`Enter` to play the complete scan. It writes the recording to `demo.gif`.

The LLDB debugging session has a separate tape:

```console
vhs debug-lldb.tape
```

It runs the failing library test, stops in the collinear-point comparison, and
inspects the values that reveal the reversed ordering. The recording is written
to `debug-lldb.gif`.

## Start debugging

The regression test is intentionally failing:

```console
cargo test
```

The algorithm and its recorded state transitions live in `src/lib.rs`. The
coordinates and expected perimeter are loaded from `data/graham.toml`. The
Ratatui runner in `src/main.rs` visualizes those states, so the same algorithm can
be investigated with prints, a command-line debugger, RustRover, or a
debugger-aware agent.

See [Graham scan](https://en.wikipedia.org/wiki/Graham_scan) for the algorithm.
