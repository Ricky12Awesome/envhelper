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
```sh
eval $(envhelper -f fish -- $(envsubst < examples/example.envhelper))
```