use std::process::Command;

use gototranscoder::adapter::cbmc2esbmc;

fn generate_cbmc_gbf(input_c: &str, output_goto: &str) {
    let goto_cc = match std::env::var("GOTO_CC") {
        Ok(v) => v,
        Err(err) => panic!("Could not get GOTO_CC bin. {}", err),
    };
    assert!(!input_c.is_empty());
    println!("Invoking cbmc with: {}", input_c);

    let output = Command::new(goto_cc)
        .arg(input_c)
        .arg("-o")
        .arg(output_goto)
        .output()
        .expect("failed to execute process");

    if !output.status.success() {
        println!("CBMC exited with {}", output.status);
        println!("\tSTDOUT: {}", String::from_utf8_lossy(&output.stdout));
        println!("\tSTDERR: {}", String::from_utf8_lossy(&output.stderr));
        panic!("GOTO-CC failed");
    }
}

fn run_esbmc_gbf(input_gbf: &str, args: &[&str], status: i32, library_gbf: &str, entrypoint: &str) {
    let cargo_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(v) => v,
        Err(err) => panic!("Could not open cargo folder. {}", err),
    };
    let library_path =
        std::path::Path::new(&cargo_dir).join(format!("resources/{}", "library.goto"));
    let esbmc = match std::env::var("ESBMC") {
        Ok(v) => v,
        Err(err) => panic!("Could not get ESBMC bin. {}", err),
    };
    let output = Command::new(esbmc)
        .arg("--cprover")
        .arg("--function")
        .arg(entrypoint)
        .arg("--binary")
        .arg(library_path)
        .arg(input_gbf)
        .args(args)
        .output()
        .expect("Failed to execute process");

    if !output.status.success() {
        println!("ESBMC exited with {}", output.status);
        println!("\tSTDOUT: {}", String::from_utf8_lossy(&output.stdout));
        println!("\tSTDERR: {}", String::from_utf8_lossy(&output.stderr));
        println!("\t[{}, {}]", library_gbf, input_gbf)
    }
    assert_eq!(status, output.status.code().unwrap());
}

fn run_test(input_c: &str, args: &[&str], expected: i32) {
    let cargo_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(v) => v,
        Err(err) => panic!("Could not open cargo folder. {}", err),
    };
    let test_path = std::path::Path::new(&cargo_dir).join(format!("resources/test/{}", input_c));

    let cbmc_gbf = format!("{}.cbmc.goto", input_c);
    let esbmc_gbf = format!("{}.esbmc.goto", input_c);

    generate_cbmc_gbf(test_path.to_str().unwrap(), cbmc_gbf.as_str());
    cbmc2esbmc(cbmc_gbf.as_str(), esbmc_gbf.as_str());
    let library_path = std::path::Path::new(&cargo_dir).join("resources/library.goto");
    run_esbmc_gbf(
        &esbmc_gbf,
        args,
        expected,
        library_path.to_str().unwrap(),
        "main",
    );
    std::fs::remove_file(&cbmc_gbf).ok();
    std::fs::remove_file(&esbmc_gbf).ok();
}

fn run_goto_test(input_goto: &str, args: &[&str], expected: i32) {
    let cargo_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(v) => v,
        Err(err) => panic!("Could not open cargo folder. {}", err),
    };
    let test_path = std::path::Path::new(&cargo_dir).join(format!("resources/test/{}", input_goto));

    let esbmc_gbf = format!("{}.goto", input_goto); // TODO: generate UUID!
    cbmc2esbmc(test_path.to_str().unwrap(), esbmc_gbf.as_str());
    let library_path = std::path::Path::new(&cargo_dir).join("resources/library.goto");
    run_esbmc_gbf(
        &esbmc_gbf,
        args,
        expected,
        library_path.to_str().unwrap(),
        "__CPROVER__start",
    );

    std::fs::remove_file(&esbmc_gbf).ok();
}

fn run_goto_test_2(input_goto: &str, args: &[&str], expected: i32, entrypoint: &str) {
    let cargo_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(v) => v,
        Err(err) => panic!("Could not open cargo folder. {}", err),
    };
    let test_path = std::path::Path::new(&cargo_dir).join(format!("resources/test/{}", input_goto));

    let esbmc_gbf = format!("{}.goto", input_goto); // TODO: generate UUID!
    cbmc2esbmc(test_path.to_str().unwrap(), esbmc_gbf.as_str());
    let library_path = std::path::Path::new(&cargo_dir).join("resources/library.goto");
    run_esbmc_gbf(
        &esbmc_gbf,
        args,
        expected,
        library_path.to_str().unwrap(),
        entrypoint,
    );

    std::fs::remove_file(&esbmc_gbf).ok();
}

#[test]
fn hello_world() {
    // Basic
    run_test("hello_world.c", &["--goto-functions-only"], 0);
    run_test("hello_world.c", &["--incremental-bmc"], 0);
    run_test("hello_world_fail.c", &["--incremental-bmc"], 1);
}

#[test]
fn hello_add() {
    // +
    run_test("hello_add.c", &["--goto-functions-only"], 0);
    run_test("hello_add.c", &["--incremental-bmc"], 0);
    run_test("hello_add_fail.c", &["--incremental-bmc"], 1);
}

#[test]
fn hello_sub() {
    // -
    run_test("hello_sub.c", &["--goto-functions-only"], 0);
    run_test("hello_sub.c", &["--incremental-bmc"], 0);
    run_test("hello_sub_fail.c", &["--incremental-bmc"], 1);
}
#[test]
fn hello_mul() {
    // *
    run_test("hello_mul.c", &["--goto-functions-only"], 0);
    run_test("hello_mul.c", &["--incremental-bmc"], 0);
    run_test("hello_mul_fail.c", &["--incremental-bmc"], 1);
}
#[test]
fn hello_div() {
    // /
    run_test("hello_div.c", &["--goto-functions-only"], 0);
    run_test("hello_div.c", &["--incremental-bmc"], 0);
    run_test("hello_div_fail.c", &["--incremental-bmc"], 1);
    run_test("hello_div_zero_fail.c", &["--incremental-bmc"], 1);
    run_test(
        "hello_div_zero_fail.c",
        &["--incremental-bmc", "--no-div-by-zero-check"],
        0,
    );
}
#[test]
fn hello_eq() {
    // ==/!=
    run_test("hello_equality.c", &["--goto-functions-only"], 0);
    run_test("hello_equality.c", &["--incremental-bmc"], 0);
    run_test("hello_equality_fail.c", &["--incremental-bmc"], 1);
}
#[test]
fn hello_ptr() {
    // pointer (address_of)
    run_test("hello_ptr.c", &["--goto-functions-only"], 0);
    run_test("hello_ptr.c", &["--incremental-bmc"], 0);
    run_test("hello_ptr_fail.c", &["--incremental-bmc"], 1);
}
#[test]
fn hello_array() {
    // array
    run_test("hello_array.c", &["--goto-functions-only"], 0);
    run_test("hello_array.c", &["--incremental-bmc"], 0);
    run_test("hello_array_fail.c", &["--goto-functions-only"], 0);
    run_test("hello_array_fail.c", &["--incremental-bmc"], 1);
    run_test("hello_array_fail_oob.c", &["--goto-functions-only"], 0);
    run_test("hello_array_fail_oob.c", &["--incremental-bmc"], 1);
    run_test("hello_array_fail_oob.c", &["--no-bounds-check"], 0);
    run_test("hello_array_init.c", &["--goto-functions-only"], 0);
    run_test("hello_array_init.c", &["--incremental-bmc"], 0);
}
#[test]
fn hello_struct() {
    // Struct
    run_test("hello_struct.c", &["--goto-functions-only"], 0);
    run_test("hello_struct.c", &["--incremental-bmc"], 0);
    run_test("hello_struct_fail.c", &["--incremental-bmc"], 1);
    run_test("hello_struct_init.c", &["--incremental-bmc"], 0);
}
#[test]
fn hello_call() {
    // Function call
    run_test("hello_func.c", &["--goto-functions-only"], 0);
    run_test("hello_func.c", &["--incremental-bmc"], 0);
    run_test("hello_func_fail.c", &["--incremental-bmc"], 1);
    run_test("hello_func_parameter.c", &["--incremental-bmc"], 0);
    run_test("hello_func_parameter_fail.c", &["--incremental-bmc"], 1);
}
#[test]
fn hello_goto() {
    // Goto-Label
    run_test("hello_label.c", &["--goto-functions-only"], 0);
    run_test("hello_label.c", &["--k-induction"], 0);
    run_test("hello_label_fail.c", &["--incremental-bmc"], 1);
}
#[test]
fn hello_if() {
    // If
    run_test("hello_if.c", &["--goto-functions-only"], 0);
    run_test("hello_if.c", &["--incremental-bmc"], 0);
    run_test("hello_if_fail.c", &["--incremental-bmc"], 1);
}

#[test]
fn struct_array() {
    // Struct of arrays
    run_test("struct_array.c", &["--goto-functions-only"], 0);
    run_test("struct_array.c", &["--incremental-bmc"], 0);
    run_test("struct_array_fail.c", &["--incremental-bmc"], 1);
}

#[test]
fn goto_test() {
    run_goto_test("mul.goto", &["--goto-functions-only"], 0);
}

//     ////////////////
//     // KANI TESTS //
//     ////////////////
//     // TODO: Integrate Kani into the test framework

#[test]
fn hello_rust_book() {
    run_goto_test("hello_world.rs.goto", &["--goto-functions-only"], 0);
    run_goto_test("hello_world.rs.goto", &["--incremental-bmc"], 1);
}

#[test]
fn first_steps_book() {
    run_goto_test("first_steps.rs.goto", &["--goto-functions-only"], 0);
    run_goto_test("first_steps.rs.goto", &["--incremental-bmc"], 1);
    run_goto_test("first-steps-pass.goto", &["--incremental-bmc"], 0);
}

#[test]
fn unchecked_add_contract() {
    // Disabled because ESBMC does not support: object_size, overflow_result-+
    run_goto_test_2(
        "checked_unchecked_add_i8.goto",
        &["--goto-functions-only"],
        0,
        "_RNvNtNtCsesPP5EAma4_4core3num6verify24checked_unchecked_add_i8",
    );
}
