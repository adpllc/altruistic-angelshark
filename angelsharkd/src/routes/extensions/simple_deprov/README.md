# Daemon Extension `simple_deprov`

This extension implements excruciatingly simple extension de-provisioning. For
example, if an agent was provisioned with a `station-user` and an
`agent-loginid`, you can submit those extensions, their type, and the ACM they
were provision on. They relevant commands to remove those objects will be
executed in parallel, and any errors encountered will be returned.

## Getting Started

To enable this feature, compile `angelsharkd` with the `simple_deprov` flag:

```sh
cargo build --bin angelsharkd --features simple_deprov ...
```

## `POST /extensions/deprov` Remove Objects

The request type is as follows:

```json
POST /extensions/deprov
[
    {
        "station-user": {
            "acm": "01",
            "ext": "17571230000"
        }
    },
    {
        "agent-loginid": {
            "acm": "01",
            "ext": "17571240000"
        }
    }
]
```

If all of the deprov commands were successful, the response is an empty array.

```json
200 OK
[]
```

If there were errors running the relevant deprov commands (such as when an
extension does not exist), they are included in the resulting array.

```json
200 OK
[
    "ACM lab: 1 00000000 309e Extension exists but assigned to a different object",
    "ACM lab: 1 00000000 2ed5 Extension assigned as remote extension on the uniform-dialplan form",
    "ACM lab: 1 00000000 2ed5 Extension assigned as remote extension on the uniform-dialplan form"
]
```

## Logging

The `deprov` endpoint always returns successfully. Any errors encountered during
the command execution are logged as `ERROR`.
