# Quantum Soft Keyboard

The keyboard remapping software you never knew you wanted.

Inspired by the open source keyboard firmware project,
[QMK](https://github.com/qmk/qmk_firmware), the goal of `qsk` is enable similar
features on arbitrary keyboards connected to a host system. For example, the
built-in keyboard on a laptop or that crummy old generic keyboard that came
with your 1990s era department store desktop computer.

# Features

* standard keyboard remapping, eg remap `F` -> `U`
* composable layers of keymappings activated by specially-assigned keys
* "tap toggle", which causes a given key to send its usual keystroke when
  tapped within a given time limit and to activate a specified layer while held

This feature set is still fairly small relative to QMK's quite prolific feature
set. Features are implemented on an as-needed basis -- contributions welcome!

# Usage

Build:

```
git clone https://github.com/waynr/qsk
cd qsk
cargo build
```

Run:
```
./target/debug/quantum-soft-keyboard -v /dev/input/by-path/<target-keyboard>
```

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
