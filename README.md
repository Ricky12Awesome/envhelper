# envhelper

Tool for helping setting environment variables for different shells

## Install

```sh
cargo install --git "https://github.com/Ricky12Awesome/envhelper"
```

## Uninstall

```sh
cargo uninstall envhelper
```

## Usage

Use `envsubst` to parse existing environment like `$HOME` to its actual value

`example.envhelper` is a line separated list of arguments

```sh
envsubst < examples/example.envhelper | envhelper -f fish | .
```

### Operators

|  OP   |                    Description                     |
|:-----:|:--------------------------------------------------:|
| `+=`  |  Appends existing environment **if not** present   |
| `+=!` |  Appends existing environment **even if** present  |
|  `=`  | Overrides existing environment **if not** present  |
| `=!`  | Overrides existing environment **even if** present |

## Format

Format is simple, name of var followed by op and then value `NAME {OP} VALUE`
these can be in a line separated list with support of comments (see usage above)