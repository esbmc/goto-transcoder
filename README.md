# GOTO Transcoder

This project is still in early development stages. The goal here is to have a tool that facilitates visualizing and changing GOTO programs generated from ESBMC and CBMC by:
- Parsing the GBF (goto binary format?) from ESBMC and CBMC
- Writing into GBF to ESBMC/CBMC. Allowing to convert between both versions.
 
# Steps to verify Rust code

For these steps let's verify a Rust hello world, we will assume that you have Kani available in your system. We will start with
the Hello World from the [Kani tutorial](https://model-checking.github.io/kani/kani-tutorial.html):

```rust
// File: test.rs
#[kani::proof]
fn main() {
    assert!(1 == 2);
}
```

## Use Kani to generate the CBMC GOTO program

Invoke Kani and ask it to keep the intermediate files: `kani test.rs --keep-temps`. This generates a `.out` file that is in the GBF
format. We can double-check this by invoking it with CBMC: `cbmc *test4main.out --show-goto-functions`:

```
[...]
main /* _RNvCshu9GRFEWjwO_4test4main */
        // 12 file test.rs line 3 column 10 function main
        DECL _RNvCshu9GRFEWjwO_4test4main::1::var_0 : struct tag-Unit
        // 13 file /Users/runner/work/kani/kani/library/std/src/lib.rs line 44 column 9 function main
        DECL _RNvCshu9GRFEWjwO_4test4main::1::var_1 : struct tag-Unit
        // 14 file /Users/runner/work/kani/kani/library/std/src/lib.rs line 44 column 22 function main
        DECL _RNvCshu9GRFEWjwO_4test4main::1::var_2 : c_bool[8]
[...]
```

## Convert the CBMC goto into ESBMC goto

1. Clone goto-transcoder: `git clone https://github.com/rafaelsamenezes/goto-transcoder.git`
2. Convert to the ESBMC file: `cargo run cbmc2esbmc  <kani-out>.out <entrypoint> <esbmc>.goto`

```
Running: goto-transcoder file.cbmc.out  _RNvCshu9GRFEWjwO_4test4main file.esbmc.goto
[2024-10-09T13:07:20Z INFO  gototranscoder] Converting CBMC input into ESBMC
[2024-10-09T13:07:20Z INFO  gototranscoder] Done
```

This will generate the `file.esbmc.goto`, which can be used as the ESBMC input.

## Invoke ESBMC

1. Invoke ESBMC with the program: `esbmc --binary file.esbmc.goto`.

```
Solving with solver Z3 v4.13.0
Runtime decision procedure: 0.001s
Building error trace

[Counterexample]


State 1 file test.rs line 4 column 5 function main thread 0
----------------------------------------------------
Violated property:
  file test.rs line 4 column 5 function main
  KANI_CHECK_ID_test.cbacc14fa409fc10::test_0
  0


VERIFICATION FAILED
```

### Contributing

- [Architecture](docs/Architecture.md)
- [Development](docs/Development.md)
