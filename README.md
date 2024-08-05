# starknet-mutation-testing

This is still very experimental, please make sure to always backup your code.

## TODO What is mutation testing

## Current limitation
This will only work if you are using scarb.  
The folder where the contracts are should be called `src` and the tests (if using snforge) should be in a folder called `tests`.  
If you tests are in the same file as the code, whenever the test attribute is encountered `#[cfg(test)]`, the rest of the file is ignored.
To run the test, this is using `scarb test` so if you are using snforge, please make sure you have this snippet in the `Scarb.toml`:
```yaml
[scripts]
test = "snforge test"
```

## Usage

Clone this repo and run:

```shell
cargo run -- --path PATH
```

This should run all mutation tests

To have more information about all the available options, please run:
```shell
cargo run -- --help
```

## TODO 
 - Logger with different levels
## License
