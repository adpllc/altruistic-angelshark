# `angelsharkcli`

This program accepts OSSI-formatted commands as input and writes the results
back out to the user. Input is read from STDIN, output is written to STDOUT (if
printing is being used) and errors are written to STDERR.

## Help

```
$ angelsharkcli -h
Altruistic Angelshark CLI 0.1.0
Adam T. Carpenter <adam.carpenter@adp.com>

Reads STDIN and parses all lines as commands to be fed to one or more ACMs. When it reaches EOF, it stops parsing and
starts executing the command(s) on the ACM(s). What it does with the output can be configured with subcommands and
flags. If you like keeping your commands in a file, consider using the `<` to read it on STDIN. The default behavior is
to run commands but print no output (for quick changes). Errors are printed on STDERR.

USAGE:
    angelsharkcli [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -l, --login-file <config>    Set ACM login configuration file [default: ./asa.cfg]

SUBCOMMANDS:
    help     Prints this message or the help of the given subcommand(s)
    man      Prints command manual pages via `ossim` term
    print    Prints command output to STDOUT (or files) in a useful format
    test     Prints parsed logins and inputs but does not run anything
```

## Input Syntax

The input syntax loosely follows that of the OSSI protocol, with one slight
modification to allow for ACM selection. One command is made up of multiple
lines and the program accepts one or more commands. Every line begins with a
single character (a, c, f, d, or t) with the following meanings.

- `a` Signifies one or more ACM names to run the command on. Accepts multiple
  values, either as tab-separated entries on the same line or as completely
  separate lines, each of which must start with `a`.
- `c` The command to be run. Only one of these per terminated body of input.
  Shorthands are supported.
- `f` Signifies one or more optional fields to be displayed or mutated. If you
  want to limit the output of list or display commands to particular fields, you
  can include them here. These are also used for change commands to specify
  which fields of data are to be mutated. Accepts multiple values, either as
  tab-sepaated entries on the same line or as completely separate lines, each of
  which must start with `f`.
- `d` Signifies one or more data entries to be inserted into provided fields for
  a change command. Accepts multiple values, either as tab-separated entries on
  the same line or as completely separate lines, each of which must start with
  `d`.
- `t` Is the terminator, and signifies the end of one command. Since you can
  input many commands, use `t` on its own line to separate them. Every command
  must end with a `t`, even if you're just sending one command.

Example:

```plain
aCM01    CM02
clist station
f8003ff00
t
```

Example with multiple fields and data insertion:

```plain
aCM01
aCM02
ccha stat 17571230000
f8003ff00    0031ff00
dCarpenter, Adam
d1002
t
```

## Login Configuration

The program expects a file called 'asa.cfg' to be in the PWD at runtime. You can
change the location and name of this file with the CLI options (invoke the
program with `--help`). The syntax of this file is as follows:

```plain
<acm name> <username>:<password>@<address>:<port>
...(repeated if desired for multiple ACMs)
```

The ACM name is a name you give to that login information to be provided in the
input later. It can be anything (e.g. CM11). The username and password are known
working credentials for that ACM. The address is the physical host or IP address
for that ACM. The port is the SSH port directly into SAT. The default port, if
none is provided, is 5022.

You can download a [sample `asa.cfg.sample`](/asa.cfg.sample) to start with.

Example:

```plain
CM01 myuser:p@$$w0rd@10.0.0.1:5022
CM02 myuser:p@$$w0rd@10.0.0.2:5022
```
