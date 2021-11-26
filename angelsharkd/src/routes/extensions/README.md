# Angelshark Daemon Extensions

This module aims to provide a simple way of extending Angelshark's basic functionality (running commands on the ACM) with additional HTTP endpoints that run one or more commands to achieve a basic business task. 

For example, say you would like you, other users, or your own software to quickly search all extension-types for a keyword. This functionality is not in the base `angelsharkd`, but it can be easily implemented with the following steps:

1. Accept a keyword from the client's request
1. Download extension-type data from one or more ACMs
1. Filter out extensions that do not match a given keyword
1. Return the remaining, matching extensions to the client

This functionality may not be desirable for all end users, and therefor is completely opt-in with feature flags. At compile time, you can add `--features simple_search` to enable a given extension called `simple_search`, for example.
