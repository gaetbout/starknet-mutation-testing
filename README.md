# starknet-mutation-testing

This is still very experimental.  
This will only work if you are using scarb.  
The folder where the contracts are should be called `src` and the tests (if using snforge) should be in a folder caller.  
Please don't run this process many times, it will create some issues.
`tests`.  
To run the test, this is using `scarb test` so if you are using snforge, please make sure you have this snippet in the `Scarb.toml`:
```yaml
[scripts]
test = "snforge test"
```

Clone this repo and run:

```shell
cargo run -- --path PATH
```

This should run all mutation tests

## TODO 
 - Logger with different levels