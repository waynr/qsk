# qsk (aka Quantum Soft Keyboard)

[<img alt="github" src="https://img.shields.io/badge/github-waynr/qsk?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/waynr/qsk)
[<img alt="crates.io" src="https://img.shields.io/crates/v/qsk.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/qsk)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-qsk?style=for-the-badge&labelColor=555555" height="20">](https://docs.rs/qsk)
[<img alt="build status" src="https://img.shields.io/github/workflow/status/waynr/qsk/cargo%20build%20%26%20test/main?style=for-the-badge" height="20">](https://github.com/waynr/qsk/actions?query=branch%3Amain)

The keyboard remapping software you never knew you wanted.

Inspired by the open source keyboard firmware project,
[QMK](https://github.com/qmk/qmk_firmware), the goal of `qsk` is enable similar
features on arbitrary keyboards connected to a host system. For example, the
built-in keyboard on a laptop or that crummy old generic keyboard that came
with your 1990s era department store desktop computer.

# Features

* standard keyboard remapping, eg remap `F` -> `U`
* composable layers of keymappings activated by keys with special functionality
* "tap toggle", which causes a given key to send its usual keystroke when
  tapped within a given time limit and to activate a specified layer while held

This feature set is still fairly small relative to QMK's quite prolific feature
set. Features are implemented on an as-needed basis -- contributions welcome!

# Usage (linux-only for now)

## Try The Example Remapper

Install:

```bash
cargo install qsk
```

Get a list of available devices:

```bash
qsk list-devices
```

After identifying the device you want to use, run the remapper:

```bash
sudo qsk remap /path/to/device-file
```

**Note**: `sudo` is necessary above because by default your linux login
user won't have the permissions necessary to grab your chosen keyboard input
device nor to create new virtual keyboard device through which your remapped
key strokes will be emitted.

## Customize and Build Your Own Remapper

The previous section describes how to use the binary shipped via crates.io,
which for now can't have its keymaps customized. In the future it will be
possible to pass it a path to a file with a keymapping DSL/script. For now,
keyboard remapping definitions must be compiled in. To make this easier, `qsk`
provides a [`cargo-generate`](https://crates.io/crates/cargo-generate) template
that helps you get started quickly to create a `qsk` project of your own:

```bash
cargo generate --git https://github.com/waynr/qsk.git qsk-template
```

`cargo-generate` will prompt you for values to fill in the `qsk` template
project, one of which will be "Project Name". The value you pass to this will
be the name of the directory of your new `qsk` project. To build it:

```bash
cd $PROJECT_NAME
cargo build
```

Get a list of available devices:

```bash
./target/debug/$PROJECT_NAME list-devices
```

After identifying the device you want to use, run the remapper:

```bash
sudo ./target/debug/$PROJECT_NAME remap /path/to/device-file
```

# The QSK Procedural Macro Remapping DSL

The abovementioned template produces a `main.rs` that looks like the following:

```rust
use std::error;

use qsk_macros;

use qsk::entrypoint;

fn main() -> Result<(), Box<dyn error::Error>> {
    let layer_composer = qsk_macros::remap!(
        ModLayer[Active]: {
            Y -> HOME,
            F -> TapToggle(Navigation, F),
        },
        Navigation: {
            END -> Exit(),
            Y -> HOME,
            U -> PAGEDOWN,
            I -> PAGEUP,
            O -> END,
            H -> LEFT,
            J -> DOWN,
            K -> UP,
            SEMICOLON -> RIGHT,
        },
    )?;

    entrypoint(layer_composer)?;
    Ok(())
}
```

This demonstrates the `qsk_macros::remap!` macro which takes as input a
mini-DSL that simplifies definition of layered keyboard remapping. The only
alternative to this currently would be defining a `qsk_types::LayerComposer`
directly in Rust. All you have to do in your generated project is to update the
key mappings, save, and build.

There are several categories of identifier to be concerned with when defining
keyboard remapping layers:

* **`Layer Name`** are identifiers like `Navigation` and `ModLayer` shown above.
    These names precede a colon with an optional set of square brackets and are
    used by Key Functions. For example, `Navigation` in `TapToggle` indicates
    that the `Navigation` layer should be activated when the key on the left
    side of the `->` is held.
* **`Layer Option`** are identifiers like `Active` in square brackets above.
    These are used to configure individual layers.
* **`Key Codes`** are identifiers like `K`, `END`, and `UP` shown above. On the
    left side of a `->` the key code indicates the "input" key that will be
    remapped. On the right side of a `->` this indicates what key code will be
    output given the key code on the left.
* **`Key Functions`** are identifies like `Exit` and `TapToggle` shown above. These
    can only appear on the right side of a `->` and are used to bestow special
    properties on the corresponding key indicated on the left side of the `->`.

## Key Functions

* **`TapToggle(<layer_ref>, <tap_key>)`** When the key on the left side of the
    `->` is pressed and held, the layer named `<layer_ref>` is activated. When
    it is tapped within the default tap toggle timeout (180 milliseconds).
* **`Exit()`** When the key on the left side of the `->` is pressed, the
    program will exit gracefully.

## Layer Options

* **`Active`** indicates that the layer should be set to "active" state on
    program initialization.

# Differences from QMK

Assuming you are familiar with QMK, you might be interested to know how this
project differs from it.

If you're not familiar with QMK, then the TL;DR is that it is software that you
can use to customize the behavior of supported keyboards to dynamically alter
the behavior of keys according. If you would like to know more, please check
out the [QMK documentation site](https://docs.qmk.fm/#/) but be warned that it
is a somewhat deep rabbithole.

## Target Runtime

QMK compiles to firmware that must be loaded onto a given target keyboard that
it supports. As such, it imposes no resource consumption burden on the host
system and minimizes latency due to the (presumably) dedicated nature of its
microcontroller.

`qsk`, on the otherhand, compiles to a binary that necessarily runs on the host
system receiving the original hardware input events and sending the same or
different events as determined by its configuration.

It requires permissions on the host system necessary to:

* Grab the input of an existing input device to receive its input events.
* Create a new virtual input device to which it sends keystrokes that it
  either generates or passes through from the source input device.

Additionally, you must tell `qsk` what source input device to grab when
executing the binary.

## Additional latency

As you can imagine, there is potential for a tool like `qsk` to inject
non-trivial between the time it receives a keyboard event and the time that it
sends corresponding potentially altered keyboard events.

The intent in choosing Rust for this tool, aside from indulging a personal
preference, is to safely minimize latency while providing opportunities to
extend its features along a number of axes. That said, there has not yet been
an effort to characterize the latency involved here but I (waynr) can attest
that it doesn't seem to be noticeable for everyday use.

## Remaps input events, not actual keys

In `qsk` we don't map desired keyboard events/behaviors to specific hardware
keys but to other keyboard or input events. Because of this you have to be
conscious of what input events your desired physical device and target host OS
map to in order to effectively remap it.

It is possible that we could in the future do something fancy like inspect
details of a given input device and allow the user to configure it using a GUI
and a presumed default layout presented to us by the input event interface.
Contributions in this area are encouraged!

# Supported Operating Systems

Because of the nature of `qsk` it's most likely that support needs to be
kernel/OS specific since that is the most natural API boundary where an
interface might be made available for such things.

## Linux

For Linux we implement device support through
[`evdev`](https://en.wikipedia.org/wiki/Evdev), a generic input event
interface provided by the kernel. This can be illustrated with the following
rough chain of relationships:

```
brain -> fingers -> keyboard -> CPU interrupts -> interrupt handlers (kernel) ->
evdev subsystem (kernel/userspace) -> qsk -> X11 input drivers OR libinput for
Wayland -> your program
```

Beware that while `qsk` is attached to a given input source it will "grab" that
input so that it has the exclusive right to read events from it.

## [TODO] Mac

I don't have any mac computers so it's not practical for me to implement
support for them. I am somewhat skeptical that it is as easy as for Linux, but
happy to be proven wrong by someone with a mac. Please feel encouraged to
implement support, I would be happy to provide whatever guidance you want or
need!

## [TODO] Windows

I don't have any windows computers so it's not practical for me to implement
support for them. I am somwhat skeptical that it is as easy as for Linux, but
am happy to be proven wrong by someone with a windows. Please feel encouraged
to implement support, I would be happy to provide whatever guidance you want or
need!

## [TODO] ??

Got an operating system or computing paradigm I don't know about? Let me know!

# Maintainer

Wayne Warren is an everyday normal guy who likes to write software in Rust and
was driven to write `qsk` out of annoyance at the lack of muscle memory
compatibility between his highly-customized mechanical keyboard firmware and
his various laptops' super uncustomizable built-in keyboards.
