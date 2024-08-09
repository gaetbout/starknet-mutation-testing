# Starknet Mutation Testing

**Note:** This project is still in an experimental stage. Please ensure you always back up your code.

## What is Mutation Testing?

Mutation testing is a software testing technique used to evaluate the quality and effectiveness of test cases. It involves making small, deliberate changes to the program's source code, known as "mutants," and then running the test cases to determine if they can detect these changes. If a test case fails when a mutant is introduced, it means the test case is effective at catching that type of error. If the test case passes despite the mutant, it indicates a potential weakness in the test suite.

## Requirements

- This tool only works with Scarb.
- The folder where the contracts are located **must** be called `src`.
- The tests (if using snforge) **must** be in a folder called `tests`.
- If your tests are in the same file as the code, whenever the test attribute `#[cfg(test)]` is encountered, the rest of the file is ignored.
- If you are using snforge to test your project, ensure you have this snippet in your `Scarb.toml`:

```toml
[scripts]
test = "snforge test"
  ```

## Limitation
At the moment only one line mutation are supported. For example if you have an `assert(...)` spread on multiple lines, it won't be modified.

## Mutation supported
 - `==` => `!=`
 - `!=` => `==`
 - ` > ` => `>=`, ` < `
 - `>=` => `==`, ` > `
 - ` < ` => `<=`, ` > `
 - `<=` => `==`, ` < `
 - `assert()` => Commented
 
## Usage 

Clone this repository and run:
```shell
cargo run -- --path PATH
```

This will run all mutation tests.

To check a specific file only, use:
```shell
cargo run -- --path PATH --file PATH_TO_FILE
```

If you stop the execution early, you can clean the generated files by running:
```shell
cargo run -- --clean
```

For more information about all available options, run:
```shell
cargo run -- --help
```

## Interesting Links
 - [How to handle Errors](https://youtu.be/j-VQCYP7wyw?si=kJgRtmUxIR5hcnIR)

## License