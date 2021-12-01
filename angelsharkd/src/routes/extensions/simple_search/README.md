# Daemon Extension `simple_search`

This extension implements fast and simple extension searching for clients. Data
to be searched is downloaded from configured ACMs as a "haystack." Clients pass
a list of one or more "needles" to search for. Haystack entries matching all
provided needles are returned.

The haystack starts out empty. To trigger a download, use the `refresh`
endpoint. Refreshes happen in the background, so clients always receive an
immediate response. This means that clients will be searching on "stale" data
(or on the first run, no data) until the refresh completes.

There is no built-in scheduler for periodic haystack refreshes. It is
recommended to use an external scheduler such as `cron(8)`.

## Getting Started

To enable this feature, compile `angelsharkd` with the `simple_search` flag:

```sh
cargo build --bin angelsharkd --features simple_search ...
```

This extension expects a single environment variable,
`ANGELSHARKD_EXT_SEARCH_ACMS`, to be set at runtime. This var should be a list
of ACM names from your `asa.cfg`. For example, if your `asa.cfg` is configured
for ACMs named 01, 02, and 03, and you want to search over 01 and 03, run
`angelsharkd` like this:

```
ANGELSHARKD_ADDR='127.0.0.1:3000' ANGELSHARKD_EXT_SEARCH_ACMS='01 03' angelsharkd
```

## `GET /extensions/search/refresh` Refresh Haystack

`GET /extensions/search/refresh`

```
200 OK text/plain
Refresh scheduled.
```

## `POST /extensions/search` Search Haystack

The return type is the entire list of fields from the `list extension-type`
command, the ROOM field from the `list station` command, and the configured name
of the ACM the entry was found on.

```json
POST /extensions/search
[
	"carpenat"
]
```

```json
200 OK
[
    [
        "17571230000",
        "station-user",
        "5",
        "1",
        "1005",
        "Carpenter, Adam",
        "2",
        "",
        "CM01",
        "carpenat"
    ],
	...
]
```

## Logging

The `refresh` endpoint always returns successfully. Any errors encountered
during the refresh are logged as `ERROR`. Successful completed refreshes are
logged as `INFO`.
