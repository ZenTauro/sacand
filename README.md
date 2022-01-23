# SACAND
This is sacand, a Simple Audio Control and Notifications Daemon

As it name oaths to imply, this is intended to be a simple audio
control daemon that sends notifications reporting pulseaudio status
through libnotify

## Usage and building
The program itself is a daemon that can be started by running the compiled
program.

To build the daemon you only need to execute the following command:
``` sh
cargo build
```

To communicate with the daemon, you need to write to the UNIX socket that
the daemon is listening to:
``` sh
printf "-5" | ncat -U "${XDG_RUNTIME_DIR}/sacand"
```

The messages that the daemon accepts are the following:
| Message | Meaning                 |
|---------|-------------------------|
| "+n"    | Increment the volume n% |
| "-n"    | Decrement the volume n% |

If the message is any other than the ones described in the table, the
daemon will simply show a notification displaying the current volume.

## License
The program sacand is free software and it is under the GNU GPLv3 License.
See the file [LICENSE.md](LICENSE.md) for copying conditions. 
