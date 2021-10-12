# `angelsharkd`

This program listens for HTTP requests. It accepts OSSI commands in JSON format
and returns OSSI output in JSON format. Below are examples of how to use the
available endpoints.

## `GET /` Current Running Version

TODO:

## `POST /ossi` Run OSSI Command(s)

Runs provided command(s) on configured ACM(s) and returns the results.

The input syntax loosely follows that of the Avaya OSSI protocol (with one
slight modification to allow for ACM specification). For more information, read
the team documentation on the protocol.

Input is a JSON array of objects. Every one of those objects is a single command
to be run. The fields of that object determine the properties of the OSSI
command to be constructed. The fields are as follows:

- `"acms"`: An array of one or more configured ACM names to run the command on.
  Required.
- `"command"`: A string representing the command to be run. Required.
- `"fields"`: An array of field hex addresses. Optional. Used for shortening
  output data entries or naming field data to be mutated.
- `"datas"`: An array of strings. For change commands, there must be as many
  `datas` as `fields`.

Request Template:

```json
{
  "acms": ["<configured ACM name>", "..."],
  "command": "<command to be run>",
  "fields": ["<optional field hex address>", "..."],
  "datas": ["<optional replacement data>", "..."]
}
```

Output is a JSON array of results. It mostly mimics the request syntax although
every element of the array is for a single command run on a single ACM. It also
accounts for errors at the OSSI level. Every object has the following fields:

- `"acm"`: The ACM the command was run on.
- `"command"`: The command that was run.
- `"fields"`: An array of field hex addresses ordinally corresponding to the
  data entries.
- `"datas"`: An array of arrays of strings. Every inner array is a single data
  entry.
- `"error"`: A string. Empty if there was no error. Populated if SAT failed to
  run the command.

Response Template:

```json
[
  {
    "acm": "<ACM command was run on>",
    "command": "<command that was run>",
    "fields": ["<field hex address>", "..."],
    "datas": [["<entry data>", "..."], "..."],
    "error": ""
  }
]
```

Here are some examples.

```json
POST /
  ossi[
    {
      "acms": ["CM01"],
      "command": "list stat 17571230000"
    }
  ]
```

```json
200 OK
[
  {
    "acm": "CM01",
    "command": "list stat 17571230000",
    "fields": [
      "8005ff00",
      "8004ff00",
      "8003ff00",
      "0031ff00",
      "8007ff00",
      "8001ff00",
      "0033ff00",
      "4e22ff00",
      "004fff00",
      "700dff00",
      "6a01ff00",
      "0019ff00",
      "ce2aff00",
      "8002ff00",
      "4a3bff00",
      "0032ff00"
    ],
    "datas": [
      [
        "17571230000",
        "S9999",
        "Carpenter, Adam",
        "1234",
        "5678",
        "5",
        "",
        "",
        "1234",
        "",
        "no",
        "",
        "",
        "1",
        "1",
        ""
      ]
    ],
    "error": ""
  }
]
```

```json
POST /
  ossi[
    {
      "acms": ["CM01"],
      "command": "cha stat 17571230000",
      "fields": ["8003ff00"],
      "datas": ["Carpenter, Adam T."]
    }
  ]
```

```json
200 OK
[
  {
    "acm": "CM01",
    "command": "cha stat 17571230009",
    "fields": [],
    "datas": [[]],
    "error": ""
  }
]
```

### `?panicky=true` Query Parameter for Error Handling

This endpoint accepts a query string in the form of `?panicky=true`. By default,
any execution errors are simply filtered out of the output. This ensures that
the output is always valid OSSI JSON. If you would rather get an error response
for any Angelshark-related errors (not SAT errors), you can set this parameter.

```json
POST /
  ossi[
    {
      "acms": ["fake"],
      "command": "list stat"
    }
  ]
```

```json
200 OK
[]
```

```json
POST /ossi?panicky=true
[
  {
    "acms": ["fake"],
    "command": "list stat"
  }
]
```

```json
500 Internal Server Error
{
  "reason": "Failed to open TCP stream to host. Make sure the config is correct and the host is otherwise reachable."
}
```

### `?no_cache=true` Query Parameter to Disable Caching

By default, all responses are stored in a timed cache for thirty minutes. If you
wish to bypass the cache (such as to validate the results of a recent change
command), you can pass `?no_cache=true` as a query string parameter.

Query parameters may be combined (ex. `?no_cache=true&panicky=true`).

## Configuration

`angelsharkd` can be configured with a variety of environment variables set at
runtime.

- `ANGELSHARKD_ORIGIN`: origin for CORS preflight requests. This is required in
  release (not debug) mode.
- `ANGELSHARKD_DEBUG`: enables debug mode. Extra logs will be written out and
  CORS will be turned off.
- `ANGELSHARKD_LOGINS`: override ACM logins file from `./asa.cfg`.
- `ANGELSHARKD_ADDR`: override socket address to listen on. Takes the format
  `127.0.0.1:8080`.

## Login Configuration

TODO:

## Logging

`angelsharkd` continuously writes logs to STDERR. In debug mode, additional,
potentially sensitive information is also logged, including request parameters.
