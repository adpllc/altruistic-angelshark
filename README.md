# Altruistic Angelshark

[![Conventional git style](https://img.shields.io/badge/git%20style-conventional%20commit-blue)](https://conventionalcommits.org/)

Angelshark is a collection of programs that make automating over Communication
Managers (ACMs) easier. It uses the OSSI protocol over SSH to run user commands
on one or more configurable ACMs. This functionality is exposed as a developer
library (`libangelshark`), a command line application (`angelsharkcli`), and an
HTTP daemon (`angelsharkd`).

## Detailed User Guides

These guides give an overview of the end-user Angelshark programs, their
capabilities, and common use cases. It assumes the user has familiarity with
using a shell and the command line. Some of the commands may be formatted for
\*nix, others for Windows. They should all be directly translatable, just make
sure you're using (for example) `./angelsharkcli` for \*nix and
`./angelsharkcli.exe` for Windows.

- [`angelsharkcli`](angelsharkcli/README.md): a command line application for
  running OSSI-formatted commands on one or more ACMs and parses the output data
  into useful formats (CSV, JSON, etc.)
- [`angelsharkd`](angelsharkd/README.md): a HTTP service for running
  OSSI-formatted commands formatted in JSON on one or more ACMs, by one or more
  clients

## Installation

`angelsharkcli` is available as a prebuilt binary for many platforms in the
GitHub releases. You may also build it from scratch with
[`cargo`](https://rustup.rs):

```sh
cargo install --path angelsharkcli
```

To build and install `angelsharkd`:

```sh
cargo install --path angelsharkd
```

## Quick Examples

Get the numbers and names of all stations on a single ACM via the CLI.

```sh
$ printf 'aCM01\nclist stat\nf8005ff00\nf8003ff00\nt\n' | angelsharkcli print
17571230001    Doe, Jane
17571230002    Doe, John
17571230003    Conference Rm. 1
```

Check three ACMs for a station and print it as JSON via the CLI.

```sh
$ angelsharkcli print <<EOF
aCM01
aCM02
aCM03
clist stat 17571230009
f8005ff00
f8003ff00
t
EOF
angelsharkcli: ossi: error: 1 00000000 29cf No records match the specified query options
angelsharkcli: ossi: error: 1 00000000 29cf No records match the specified query options
[
  [
    "17571230009",
    "Carpenter, Adam"
  ]
]
```

Do the same thing over HTTP via `curl`.

```sh
$ nohup angelsharkd &
nohup: ignoring input and appending output to 'nohup.out'
$ curl -X POST http://localhost:80/ossi \
    -H 'Content-Type: application/json' \
    -d '[
    {
        "acms": [ "CM01", "CM02", "CM03" ],
        "command": "list stat 17571230009",
        "fields": ["8005ff00", "8003ff00"]
    }
]'

[
    {
        "acm": "CM01",
        "command": "list stat 17571230009",
        "fields": [ "8005ff00", "8003ff00" ],
        "datas": [ [ "17571230001", "Carpenter, Adam", ] ],
        "error": ""
    },
    {
        "acm": "CM02",
        "command": "list stat 17571230001",
        "fields": [ "8005ff00", "8003ff00" ],
        "datas": [ [] ],
        "error": "1 00000000 29cf No records match the specified query options"
    }
    {
        "acm": "CM03",
        "command": "list stat 17571230001",
        "fields": [ "8005ff00", "8003ff00" ],
        "datas": [ [] ],
        "error": "1 00000000 29cf No records match the specified query options"
    }
]

$ cat nohup.out
[2021-07-21T16:57:12Z INFO  angelsharkd] Reading config...
[2021-07-21T16:57:12Z INFO  angelsharkd] Initializing command runner...
[2021-07-21T16:57:12Z INFO  angelsharkd] Starting server...
[2021-07-21T16:58:49Z INFO  angelsharkd::activity_log] {"status":"200 OK","method":"POST","uri":"/ossi","commands":[{"acms":["CM01","CM02","CM03"],"command":"list stat 17571230009","fields":["8005ff00","8003ff00"],"datas":null}]}
```

## Why should I use Angelshark? Why might it be preferable to ASA or other projects?

- Angelshark is cross-platform. It is available for Windows, Mac OSX, Linux, and
  FreeBSD. Angelshark should run on anything that can compile it. If statically
  linked, no runtime dependencies are required.
- It requires no graphical desktop to generate the same output as ASA, making it
  more versatile for servers or containers.
- Angelshark executables are super-fast binaries and commands are run in
  parallel across multiple ACMs to deliver output faster.
- Angelshark does not need or use waits or timeouts (such as one would see with
  `expect` scripts).
- Angelshark can parse ACM output data into a variety of useful formats
  including tab-delimited (TSV), CSV, and JSON.
- The command line is easily scripted with login configuration, command files,
  and task schedulers such as Cron. It outputs tab-delimited data by default,
  making it friendly for `sed`, `grep`, and `awk`.
- It is extensible, allowing developers to use the daemon or the library to
  write their own software on top of Angelshark!
- Angelshark uses `libssh` internally and will work on platforms without an SSH
  client, such as OpenSSH.

``````plain
                                                  `r***`
                                                _i?`  .|v-
                                              <x>        ^x:
                                            rx_            =]~  `-'
                                _^**^**=` -V,                X]*<!~^*r*
:r*:                        rr**"      '<*n                 ,nl`?#i   `}~   *_
H  _r?r`                   (~                                 i"_n,     l~~|?
5     `*(r`            ``'_P`                                         <` 0_
P        `*^^^^^^^^^^^>!=:,-`'-,,:=!~><<^^^^**^^^^<<~~!=:,,-'`        !i H
5          `````.''-__,,,::"                                          =.-q^*x
H      :rrr>~!!==::,,,_--'.Z                                , y`6@=    -e`  r`
xr=**rr"                   n_                               ??"!~]`  !|?
 ..                        `}r:`         '(`                 ,Hx^^^^*-
                              -<***^^^^r(<H`                 }<
                                      `   :(r~`            '2,
                                             `<****=`    `vv
                                                   .!*^^^r`
``````
