# pwrsurge
A power manager written in Rust using [`neli`](https://github.com/jbaublitz/neli).

# Interface
`pwrsurge` only accepts two command line arguments:
* `-l` - This is the path to the shared object (`.so` on Linux) that
contains the power management handler code. Default is
`/etc/pwrsurge/pwrsurge.conf`.
* `-c` - This is the path to the configuration file. Default is
`/etc/pwrsurge/pwrsurge.conf`.

# Power management library interface
The interface can theoretically be used with C, Rust or Golang. Given
that the two optional methods must conform to the C ABI, C may require
the least boilerplate code.

The first method in the interface is:

```c
int evdev_handler(input_event *event);
```

in C or:

```rust
#[no_mangle]
pub fn evdev_handler(event: *const InputEvent) -> i32;

```

in Rust where `InputEvent` is a C-compatible (`#[repr(C)]`) Rust
version of C's `input_event` struct. See the examples directory for
more details.

The second method in the interface is:

```c
int acpi_handler(acpi_event *event);
```

in C or:

```rust
#[no_mangle]
pub fn acpi_handler(event: *const u8) -> i32;

```

in Rust where the pointer points to a buffer representing an event
that is is a C-compatible representation of
[this struct](https://github.com/torvalds/linux/blob/master/drivers/acpi/event.c#L52).
See the examples directory for more details.

# Config file

See the examples directory for a more robust version of the
configuration file.

The configuration file allows a user to whitelist which events should
be handled by `pwrsurge`. All other events are ignored.

The `[acpi]` section handles whitelisting ACPI events and the
`[evdev]` section handles whitelisting events from evdev devices
such as the keyboard and laptop lid.

If a whitelist is not specified, all events are handled.

# Documentation
Documentation lives [here](https://docs.rs/crate/pwrsurge).

Check out the examples directory for a prototype of a power management
handler. Some of the commented out code may also be useful if you
are not using a desktop environment that handles this for you.
