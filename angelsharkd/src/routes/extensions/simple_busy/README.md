# Daemon Extension `simple_busy`

This extension implements simple extension busyout and release toggling.

## Getting Started

To enable this feature, compile `angelsharkd` with the `simple_busy` flag:

```sh
cargo build --bin angelsharkd --features simple_busy ...
```

## `POST /extensions/service/busyout` Busy-out a station

The request consists of one or more entries including the ACM and extension to
be operated on.

```json
POST /extensions/service/toggle
[
    {
        "acm": "01",
        "ext": "17571230000"
    }
]
```

The response is a typical `angelsharkd` OSSI reponse.

```json
[
  {
    "acm": "01",
    "command": "busyout station 17571230000",
    "error": "",
    "fields": ["0001ff00", "0002ff00", "0005ff00", "0003ff00", "0004ff00"],
    "datas": [["S075157", "DIG-IP-S", "17571230000", "ABORT", "1010"]]
  }
]
```

## `POST /extensions/service/release` Release a station

This endpoint works identically to the busyout endpoint, but the response will
indicate whether the station was released.

## `POST /extensions/service/toggle` Busyout and then immediately release a station

This endpoint runs two OSSI commands for busyout-ing and releasing the given
stations, respectively.
