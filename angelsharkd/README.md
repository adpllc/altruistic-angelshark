# `angelsharkd`

This program listens for HTTP connections. It accepts OSSI commands in JSON
format and returns OSSI output in JSON format. Below are examples of how to use
the available endpoints.

## Endpoints

- `GET /` greeting and version number
- `POST /ossi` run OSSI commands

## Running Commands via `POST /ossi`

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
    "acms": [
        "<configured ACM name>",
        ...
    ],
    "command": "<command to be run>",
    "fields": [
        "<optional field hex address>",
        ...
    ],
    "datas": [
        "<optional replacement data>",
        ...
    ]
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
        "fields": [
            "<field hex address>",
            ...
        ],
        "datas": [
            [
                "<entry data>",
                ...
            ],
            ...
        ],
        "error": ""
    }
]
```

Here are some examples.

`POST /ossi`

```json
[
  {
    "acms": ["CM01"],
    "command": "list stat 17571230009"
  }
]
```

`200 OK`

```json
[
  {
    "acm": "CM01",
    "command": "list stat 17571230009",
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

`POST /ossi`

```json
[
  {
    "acms": ["CM01"],
    "command": "cha stat 17571230000",
    "fields": ["8003ff00"],
    "datas": ["Carpenter, Adam T."]
  }
]
```

`200 OK`

```json
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

### To Panic or Not to Panic (Query Strings)

This endpoint accepts a single query string in the form of `?panicky=true`. By
default, any execution errors are simply filtered out of the output. This
ensures that the output is always valid OSSI JSON. If you would rather get an
error response for any Angelshark-related errors (not SAT errors), you can set
this parameter.

More examples.

`POST /ossi`

```json
[
  {
    "acms": ["CM01"],
    "command": "list stat nonexistent"
  }
]
```

`200 OK`

```json
[]
```

`POST /ossi?panicky=true`

```json
[
  {
    "acms": ["CM01"],
    "command": "list stat nonexistent"
  }
]
```

`500 Internal Server Error`

```plain
Unable to execute command -- ACM didn't feel like it today....
```

## Login Configuration

See [`angelshark.toml.sample`](/samples/angelshark.toml.sample) to get an idea
of how to configure `angelsharkd`.

## Logging

`angelsharkd` writes standard access logs to STDOUT every time it responds to a
request. Inner `libangelshark` errors may be written to STDOUT.
