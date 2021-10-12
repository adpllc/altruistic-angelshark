# Altruistic Angelshark

![Conventional git style](https://img.shields.io/badge/git%20style-conventional%20commit-blue)

![GitHub Workflow Status](https://img.shields.io/github/workflow/status/adpllc/altruistic-angelshark/Publish)

Altruistic Angelshark is a project devoted to making Communication Manager (ACM)
automation easier. It uses the OSSI protocol over SSH to run user commands on
one or more configurable ACMs. This functionality is exposed as a developer
library ([`libangelshark`](./libangelshark)), a command line application
([`angelsharkcli`](./angelsharkcli)), and an HTTP daemon
([`angelsharkd`](./angelsharkd)).

## Detailed User Guides

These guides give an overview of the end-user Angelshark applications, their
capabilities, and common use cases. It assumes the user has familiarity with
using a shell and the command line. Most commands are formatted for \*nix. They
should all be directly translatable to Windows, just make sure you're using (for
example) `angelsharkcli` for \*nix and `angelsharkcli.exe` for Windows.

- [`angelsharkcli`](angelsharkcli/README.md): a command line application for
  running OSSI-formatted commands on one or more ACMs and parses the output data
  into useful formats (CSV, JSON, etc.)
- [`angelsharkd`](angelsharkd/README.md): a HTTP service for running
  OSSI-formatted commands formatted in JSON on one or more ACMs, by one or more
  clients

## Installation

`angelsharkcli` is available as a prebuilt binary for many platforms in the
GitHub releases. To install `angelsharkcli` from source, use
[`cargo`](https://rustup.rs):

```
cargo install --git https://github.com/adpllc/altruistic-angelshark.git angelsharkcli
```

To install `angelsharkd` from source:

```
cargo install --git https://github.com/adpllc/altruistic-angelshark.git angelsharkd
```

## Quick Examples

Get the numbers and names of all stations on a single ACM via the CLI.

```sh
$ printf 'a03\nclist stat\nf8005ff00\nf8003ff00\nt\n' | angelsharkcli print
17571230001    Arnold, Ray
17571230002    Muldoon, Robert
17571230003    Panic Rm. 1
```

Check three ACMs for a station and print it as JSON via the CLI.

```sh
$ angelsharkcli print --format json <<EOF
a01
a02
a03
clist stat 17571230000
f8005ff00
f8003ff00
t
EOF
[
  [
    "17571230000",
    "Nedry, Dennis",
  ]
]
angelsharkcli: ossi (02): 1 00000000 29cf No records match the specified query options
angelsharkcli: ossi (01): 1 00000000 29cf No records match the specified query options
```

Do the same thing over HTTP with `curl`.

```sh
$ nohup angelsharkd &
nohup: ignoring input and appending output to 'nohup.out'
$ curl -X POST http://localhost:8080/ossi -H 'Content-Type: application/json' -d '[
    {
        "acms": [
            "lab",
            "04",
            "11"
        ],
        "command": "list stat 17576123489",
        "fields": [
            "8005ff00",
            "8003ff00"
        ]
    }
]'

[
    {
        "acm": "01",
        "command": "list stat 17571230000",
        "error": "1 00000000 29cf No records match the specified query options",
        "fields": [ "8005ff00", "8003ff00" ],
        "datas": []
    },
    {
        "acm": "03",
        "command": "list stat 17571230000",
        "error": "",
        "fields": [ "8005ff00", "8003ff00" ],
        "datas": [
            [
                "17571230000",
                "Nedry, Dennis"
            ]
        ]
    },
    {
        "acm": "02",
        "command": "list stat 17571230000",
        "error": "1 00000000 29cf No records match the specified query options",
        "fields": [ "8005ff00", "8003ff00" ],
        "datas": []
    }
]

$ cat nohup.out
[2021-10-12T19:34:55Z INFO  angelsharkd] Starting server on 127.0.0.1:8080 ...
[2021-10-12T19:35:44Z INFO  warp::filters::log] 127.0.0.1:49366 "POST /ossi HTTP/1.1" 200 "-" "curl/7.71.1" 4.2123963s
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
