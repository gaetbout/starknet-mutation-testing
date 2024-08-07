# starknet-mutation-testing

This is still very experimental, please make sure to always backup your code.

## What is mutation testing

Mutation testing is a software testing technique used to evaluate the quality and effectiveness of test cases. It involves making small, deliberate changes to the program's source code, known as "mutants," and then running the test cases to determine if they can detect these changes. If a test case fails when a mutant is introduced, it means the test case is effective at catching that type of error. If the test case passes despite the mutant, it indicates a potential weakness in the test suite.

## Current limitation
This will only work if you are using scarb.  
The folder where the contracts are located **MUST** be called `src` and the tests (if using snforge) **MUST** be in a folder called `tests`.  
If your tests are in the same file as the code, whenever the test attribute is encountered `#[cfg(test)]`, the rest of the file is ignored.
If you are relying on snforge to test your project, please make sure you have this snippet in the `Scarb.toml`:
```yaml
[scripts]
test = "snforge test"
```

## Usage

Clone this repo and run:

```shell
cargo run -- --path PATH
```

If you want to check one file only please use:

```shell
cargo run -- --path PATH --file PATH_TO_FILE
```

This should run all mutation tests

To have more information about all the available options, please run:
```shell
cargo run -- --help
```

## TODO 
 - Logger with different levels

## Interesting links
 - [How to handle Errors](https://youtu.be/j-VQCYP7wyw?si=kJgRtmUxIR5hcnIR)
 
## License
