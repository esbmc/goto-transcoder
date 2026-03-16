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

/// Parses a test.desc file and returns (source_file, esbmc_args, expected_exit_code).
/// Format: MODE (line 0, ignored), main file (line 1), args (line 2), expected result (remaining).
/// Panics if the expected result is not VERIFICATION SUCCESSFUL or FAILED.
fn parse_test_desc(content: &str) -> (String, Vec<String>, i32) {
    let lines: Vec<&str> = content.lines().collect();
    let source = lines
        .get(1)
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "main.c".to_string());
    let args = lines
        .get(2)
        .map(|s| s.split_whitespace().map(str::to_string).collect())
        .unwrap_or_default();
    let expected = lines
        .iter()
        .find_map(|line| {
            if line.contains("VERIFICATION SUCCESSFUL") {
                Some(0)
            } else if line.contains("VERIFICATION FAILED") {
                Some(1)
            } else {
                None
            }
        })
        .expect("test.desc must contain VERIFICATION SUCCESSFUL or VERIFICATION FAILED");
    (source, args, expected)
}

fn run_cbmc_regression_test(test_dir: &str) {
    let cargo_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(v) => v,
        Err(err) => panic!("Could not open cargo folder. {}", err),
    };
    let dir_path = std::path::Path::new(&cargo_dir)
        .join("resources/cbmc")
        .join(test_dir);

    let desc_content = std::fs::read_to_string(dir_path.join("test.desc"))
        .unwrap_or_else(|_| panic!("Could not read test.desc for: {}", test_dir));
    let (source_file, args, expected) = parse_test_desc(&desc_content);
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let source_path = dir_path.join(&source_file);

    let cbmc_gbf = format!("{}.cbmc.goto", test_dir);
    let esbmc_gbf = format!("{}.esbmc.goto", test_dir);

    generate_cbmc_gbf(source_path.to_str().unwrap(), cbmc_gbf.as_str());
    cbmc2esbmc(cbmc_gbf.as_str(), esbmc_gbf.as_str());
    let library_path = std::path::Path::new(&cargo_dir).join("resources/library.goto");
    run_esbmc_gbf(
        &esbmc_gbf,
        &args,
        expected,
        library_path.to_str().unwrap(),
        "main",
    );
    std::fs::remove_file(&cbmc_gbf).ok();
    std::fs::remove_file(&esbmc_gbf).ok();
}

fn run_cbmc_regression_test_file(input_c: &str, args: &[&str], expected: i32) {
    let cargo_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(v) => v,
        Err(err) => panic!("Could not open cargo folder. {}", err),
    };
    let test_path = std::path::Path::new(&cargo_dir).join(format!("resources/cbmc/{}", input_c));

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

// CBMC regression

#[test]
fn test_01_cbmc_unwinding_locality1() {
    run_cbmc_regression_test("01_cbmc_Unwinding_Locality1");
}

#[test]
fn test_01_cbmc_return_void() {
    run_cbmc_regression_test("01_cbmc-return_void");
}

#[test]
fn test_01_cbmc___func__1() {
    run_cbmc_regression_test_file("01_cbmc___func__1.c", &[], 0);
}

#[test]
fn test_01_cbmc_label1() {
    run_cbmc_regression_test_file("01_cbmc_label1.c", &["--unwind", "1"], 1);
}

#[test]
fn test_01_cbmc_void_ifthenelse() {
    run_cbmc_regression_test_file("01_cbmc_void_ifthenelse.c", &[], 1);
}

#[test]
fn test_01_cbmc_anonymous_struct1() {
    run_cbmc_regression_test("01_cbmc_Anonymous_Struct1");
}

#[test]
fn test_01_cbmc_anonymous_struct2() {
    run_cbmc_regression_test("01_cbmc_Anonymous_Struct2");
}

#[test]
fn test_01_cbmc_for1() {
    run_cbmc_regression_test("01_cbmc_for1");
}

#[test]
fn test_01_cbmc_for2() {
    run_cbmc_regression_test("01_cbmc_for2");
}

#[test]
fn test_01_cbmc_for3() {
    run_cbmc_regression_test("01_cbmc_for3");
}

#[test]
fn test_01_cbmc_sizeof1() {
    run_cbmc_regression_test("01_cbmc_Sizeof1")
}

#[test]
fn test_01_cbmc_statement_expression1() {
    run_cbmc_regression_test("01_cbmc_Statement_Expression1")
}

#[test]
fn test_01_cbmc_array_initialization1() {
    run_cbmc_regression_test("01_cbmc_Array_Initialization1");
}

#[test]
fn test_01_cbmc_structs2() {
    run_cbmc_regression_test("01_cbmc_Structs2");
}

#[test]
fn test_01_cbmc_structs3() {
    run_cbmc_regression_test("01_cbmc_Structs3");
}
#[test]
fn test_01_cbmc_switch1() {
    run_cbmc_regression_test("01_cbmc_switch1");
}

#[test]
fn test_01_cbmc_switch3() {
    run_cbmc_regression_test("01_cbmc_switch3");
}

#[test]
fn test_01_cbmc_switch4() {
    run_cbmc_regression_test("01_cbmc_switch4");
}

#[test]
fn test_01_cbmc_switch5() {
    run_cbmc_regression_test("01_cbmc_switch5");
}

#[test]
fn test_01_cbmc_typedef_code() {
    run_cbmc_regression_test("01_cbmc_typedef_code");
}

#[test]
fn test_01_cbmc_typedef1() {
    run_cbmc_regression_test("01_cbmc_typedef1");
}

#[test]
fn test_01_cbmc_unbounded_array3() {
    run_cbmc_regression_test("01_cbmc_Unbounded_Array3");
}

#[test]
fn test_01_cbmc_unbounded_array4() {
    run_cbmc_regression_test("01_cbmc_Unbounded_Array4");
}

#[test]
fn test_01_cbmc_undef_function1() {
    run_cbmc_regression_test("01_cbmc_Undef_Function1");
}

#[test]
fn test_01_cbmc_recursive_structure1() {
    run_cbmc_regression_test("01_cbmc_Recursive_Structure1");
}

#[test]
fn test_01_cbmc_refinement1() {
    run_cbmc_regression_test("01_cbmc_Refinement1");
}

#[test]
fn test_01_cbmc_refinement6() {
    run_cbmc_regression_test("01_cbmc_Refinement6");
}

#[test]
fn test_01_cbmc_return1() {
    run_cbmc_regression_test("01_cbmc_return1");
}

#[test]
fn test_01_cbmc_pointer27() {
    run_cbmc_regression_test("01_cbmc_Pointer27");
}

#[test]
fn test_01_cbmc_pointer28() {
    run_cbmc_regression_test("01_cbmc_Pointer28");
}

#[test]
fn test_01_cbmc_pointer29() {
    run_cbmc_regression_test("01_cbmc_Pointer29");
}

#[test]
fn test_01_cbmc_pointer30() {
    run_cbmc_regression_test("01_cbmc_Pointer30");
}

#[test]
fn test_01_cbmc_pointer21() {
    run_cbmc_regression_test("01_cbmc_Pointer21");
}

#[test]
fn test_01_cbmc_pointer22() {
    run_cbmc_regression_test("01_cbmc_Pointer22");
}

#[test]
fn test_01_cbmc_pointer23() {
    run_cbmc_regression_test("01_cbmc_Pointer23");
}

#[test]
fn test_01_cbmc_pointer17() {
    run_cbmc_regression_test("01_cbmc_Pointer17");
}

#[test]
fn test_01_cbmc_pointer19() {
    run_cbmc_regression_test("01_cbmc_Pointer19");
}

#[test]
fn test_01_cbmc_pointer15() {
    run_cbmc_regression_test("01_cbmc_Pointer15");
}

#[test]
fn test_01_cbmc_pointer12() {
    run_cbmc_regression_test("01_cbmc_Pointer12");
}

#[test]
fn test_01_cbmc_pointer10() {
    run_cbmc_regression_test("01_cbmc_Pointer10");
}

#[test]
fn test_01_cbmc_pointer6() {
    run_cbmc_regression_test("01_cbmc_Pointer6");
}

#[test]
fn test_01_cbmc_pointer7() {
    run_cbmc_regression_test("01_cbmc_Pointer7");
}

#[test]
fn test_01_cbmc_pointer4() {
    run_cbmc_regression_test("01_cbmc_Pointer4");
}

#[test]
fn test_01_cbmc_pointer3() {
    run_cbmc_regression_test("01_cbmc_Pointer3");
}

#[test]
fn test_01_cbmc_pointer1() {
    run_cbmc_regression_test("01_cbmc_Pointer1");
}

#[test]
fn test_01_cbmc_offsetof1() {
    run_cbmc_regression_test("01_cbmc_offsetof1");
}

#[test]
fn test_01_cbmc_nondet1() {
    run_cbmc_regression_test("01_cbmc_Nondet1");
}

#[test]
fn test_01_cbmc_multi_dimensional_array2() {
    run_cbmc_regression_test("01_cbmc_Multi_Dimensional_Array2");
}

#[test]
fn test_01_cbmc_multiple() {
    run_cbmc_regression_test("01_cbmc_Multiple");
}

#[test]
fn test_01_cbmc_goto1() {
    run_cbmc_regression_test("01_cbmc_Goto1");
}

#[test]
fn test_01_cbmc_goto2() {
    run_cbmc_regression_test("01_cbmc_Goto2");
}

#[test]
fn test_01_cbmc_goto3() {
    run_cbmc_regression_test("01_cbmc_Goto3");
}

#[test]
fn test_01_cbmc_goto4() {
    run_cbmc_regression_test("01_cbmc_Goto4");
}

#[test]
fn test_01_cbmc_goto5() {
    run_cbmc_regression_test("01_cbmc_Goto5");
}

#[test]
fn test_01_cbmc_return3() {
    run_cbmc_regression_test("01_cbmc_return3")
}

#[test]
fn test_01_cbmc_goto6() {
    run_cbmc_regression_test("01_cbmc_Goto6");
}

#[test]
fn test_01_cbmc_goto7() {
    run_cbmc_regression_test("01_cbmc_Goto7");
}

#[test]
fn test_01_cbmc_if1() {
    run_cbmc_regression_test("01_cbmc_if1");
}

#[test]
fn test_01_cbmc_if2() {
    run_cbmc_regression_test("01_cbmc_if2");
}

#[test]
fn test_01_cbmc_if3() {
    run_cbmc_regression_test("01_cbmc_if3");
}

#[test]
fn test_01_cbmc_if4() {
    run_cbmc_regression_test("01_cbmc_if4");
}

#[test]
fn test_01_cbmc_initialization1() {
    run_cbmc_regression_test("01_cbmc_Initialization1");
}

#[test]
fn test_01_cbmc_longconst() {
    run_cbmc_regression_test("01_cbmc_LongConst");
}

#[test]
fn test_01_cbmc_mod1() {
    run_cbmc_regression_test("01_cbmc_Mod1");
}

#[test]
fn test_01_cbmc_ms_types1() {
    run_cbmc_regression_test("01_cbmc_MS_types1");
}

#[test]
fn test_01_cbmc_function6() {
    run_cbmc_regression_test("01_cbmc_Function6");
}

#[test]
fn test_01_cbmc_function7() {
    run_cbmc_regression_test("01_cbmc_Function7");
}

#[test]
fn test_01_cbmc_function_pointer9() {
    run_cbmc_regression_test("01_cbmc_Function_Pointer9");
}

#[test]
fn test_01_cbmc_function_pointer10() {
    run_cbmc_regression_test("01_cbmc_Function_Pointer10");
}

#[test]
fn test_01_cbmc_function_pointer13() {
    run_cbmc_regression_test("01_cbmc_Function_Pointer13");
}

#[test]
fn test_01_cbmc_function_pointer14() {
    run_cbmc_regression_test("01_cbmc_Function_Pointer14");
}

#[test]
fn test_01_cbmc_function_knr1() {
    run_cbmc_regression_test("01_cbmc_Function-KnR1");
}

#[test]
fn test_01_cbmc_function1() {
    run_cbmc_regression_test("01_cbmc_Function1");
}

#[test]
fn test_01_cbmc_function2() {
    run_cbmc_regression_test("01_cbmc_Function2");
}

#[test]
fn test_01_cbmc_function3() {
    run_cbmc_regression_test("01_cbmc_Function3");
}

#[test]
fn test_01_cbmc_function4() {
    run_cbmc_regression_test("01_cbmc_Function4");
}

#[test]
fn test_01_cbmc_function_pointer3() {
    run_cbmc_regression_test("01_cbmc_Function_Pointer3");
}

#[test]
fn test_01_cbmc_function_pointer4() {
    run_cbmc_regression_test("01_cbmc_Function_Pointer4");
}

#[test]
fn test_01_cbmc_function_pointer5() {
    run_cbmc_regression_test("01_cbmc_Function_Pointer5");
}

#[test]
fn test_01_cbmc_function_pointer1() {
    run_cbmc_regression_test("01_cbmc_Function_Pointer1");
}

#[test]
fn test_01_cbmc_externfixedbv2() {
    run_cbmc_regression_test("01_cbmc_ExternFixedbv2");
}

#[test]
fn test_01_cbmc_division_assignment() {
    run_cbmc_regression_test("01_cbmc_Division_Assignment");
}

#[test]
fn test_01_cbmc_division() {
    run_cbmc_regression_test("01_cbmc_Division");
}

#[test]
fn test_01_cbmc_ellipsis() {
    run_cbmc_regression_test("01_cbmc_Ellipsis");
}

#[test]
fn test_01_cbmc_empty_declaration1() {
    run_cbmc_regression_test("01_cbmc_Empty_Declaration1");
}

#[test]
fn test_01_cbmc_charconst1() {
    run_cbmc_regression_test("01_cbmc_charConst1");
}

#[test]
fn test_01_cbmc_bv_arithmetic5() {
    run_cbmc_regression_test("01_cbmc_BV_Arithmetic5");
}

#[test]
fn test_01_cbmc_bv_arithmetic3() {
    run_cbmc_regression_test("01_cbmc_BV_Arithmetic3");
}

#[test]
fn test_01_cbmc_bv_arithmetic1() {
    run_cbmc_regression_test("01_cbmc_BV_Arithmetic1");
}

#[test]
fn test_01_cbmc_array_pointer4() {
    run_cbmc_regression_test("01_cbmc_Array_Pointer4");
}

#[test]
fn test_01_cbmc_array_pointer5() {
    run_cbmc_regression_test("01_cbmc_Array_Pointer5");
}

#[test]
fn test_01_cbmc_arraycomp() {
    run_cbmc_regression_test("01_cbmc_arraycomp");
}

#[test]
fn test_01_cbmc_ashr() {
    run_cbmc_regression_test("01_cbmc_ASHR");
}

#[test]
fn test_01_cbmc_ashr2() {
    run_cbmc_regression_test("01_cbmc_ASHR2");
}

#[test]
fn test_01_cbmc_asm1() {
    run_cbmc_regression_test("01_cbmc_asm1");
}

#[test]
fn test_01_cbmc_assignment_to_typecast() {
    run_cbmc_regression_test("01_cbmc_Assignment_to_typecast");
}

#[test]
fn test_01_cbmc_forward_declaration1() {
    run_cbmc_regression_test("01_cbmc_Forward_Declaration1");
}

#[test]
fn test_01_cbmc_extern() {
    run_cbmc_regression_test("01_cbmc_Extern");
}

#[test]
fn test_01_cbmc_externextern3() {
    run_cbmc_regression_test("01_cbmc_ExternExtern3");
}

#[test]
fn test_01_cbmc_externfailed_symbols1() {
    run_cbmc_regression_test("01_cbmc_ExternFailed_Symbols1");
}

#[test]
fn test_01_cbmc_externfailing_assert1() {
    run_cbmc_regression_test("01_cbmc_ExternFailing_Assert1");
}

// Broken tests (looks OK but don't run)

#[ignore]
#[test]
fn test_01_cbmc_function_pointer11() {
    run_cbmc_regression_test("01_cbmc_Function_Pointer11");
}

#[ignore]
#[test]
fn test_01_cbmc_function_pointer12() {
    run_cbmc_regression_test("01_cbmc_Function_Pointer12");
}

#[ignore]
#[test]
fn test_01_cbmc_static1() {
    run_cbmc_regression_test("01_cbmc_Static1");
}

#[ignore]
#[test]
fn test_01_cbmc_static2() {
    run_cbmc_regression_test("01_cbmc_Static2");
}

#[ignore]
#[test]
fn test_01_cbmc_string_abstraction1() {
    run_cbmc_regression_test("01_cbmc_String_Abstraction1");
}

#[ignore]
#[test]
fn test_01_cbmc_string_abstraction2() {
    run_cbmc_regression_test("01_cbmc_String_Abstraction2");
}

#[ignore]
#[test]
fn test_01_cbmc_string_abstraction3() {
    run_cbmc_regression_test("01_cbmc_String_Abstraction3");
}

#[ignore]
#[test]
fn test_01_cbmc_string_abstraction4() {
    run_cbmc_regression_test("01_cbmc_String_Abstraction4");
}

#[ignore]
#[test]
fn test_01_cbmc_string_abstraction5() {
    run_cbmc_regression_test("01_cbmc_String_Abstraction5");
}

#[ignore]
#[test]
fn test_01_cbmc_string_abstraction7() {
    run_cbmc_regression_test("01_cbmc_String_Abstraction7");
}

#[ignore]
#[test]
fn test_01_cbmc_string_abstraction8() {
    run_cbmc_regression_test("01_cbmc_String_Abstraction8");
}

#[ignore]
#[test]
fn test_01_cbmc_string_abstraction9() {
    run_cbmc_regression_test("01_cbmc_String_Abstraction9");
}

#[ignore]
#[test]
fn test_01_cbmc_string_abstraction10() {
    run_cbmc_regression_test("01_cbmc_String_Abstraction10");
}

#[ignore]
#[test]
fn test_01_cbmc_union1() {
    run_cbmc_regression_test("01_cbmc_union1");
}

#[ignore]
#[test]
fn test_01_cbmc_union2() {
    run_cbmc_regression_test("01_cbmc_union2");
}

#[ignore]
#[test]
fn test_01_cbmc_union3() {
    run_cbmc_regression_test("01_cbmc_union3");
}

#[ignore]
#[test]
fn test_01_cbmc_union4() {
    run_cbmc_regression_test("01_cbmc_union4");
}

#[ignore]
#[test]
fn test_01_cbmc_union5() {
    run_cbmc_regression_test("01_cbmc_union5");
}

#[ignore]
#[test]
fn test_01_cbmc_array_access2() {
    run_cbmc_regression_test("01_cbmc_Array_Access2");
}

#[ignore]
#[test]
fn test_01_cbmc_array_access3() {
    run_cbmc_regression_test("01_cbmc_Array_Access3");
}

#[ignore]
#[test]
fn test_01_cbmc_recursion() {
    run_cbmc_regression_test("01_cbmc_Recursion");
}

#[ignore]
#[test]
fn test_01_cbmc_recursion2() {
    run_cbmc_regression_test("01_cbmc_Recursion2");
}

#[ignore]
#[test]
fn test_01_cbmc_recursion3() {
    run_cbmc_regression_test("01_cbmc_Recursion3");
}

#[ignore]
#[test]
fn test_01_cbmc_pointer18() {
    run_cbmc_regression_test("01_cbmc_Pointer18");
}

#[ignore]
#[test]
fn test_01_cbmc_abs1() {
    run_cbmc_regression_test_file("01_cbmc_abs1.c", &["--goto-functions-only"], 0);
}

#[ignore]
#[test]
fn test_01_cbmc_while1() {
    run_cbmc_regression_test_file("01_cbmc_while1.c", &["--goto-functions-only"], 0);
}

#[ignore]
#[test]
fn test_01_cbmc_argc1() {
    run_cbmc_regression_test("01_cbmc_argc1");
}

#[ignore]
#[test]
fn test_01_cbmc_array_access1() {
    run_cbmc_regression_test("01_cbmc_Array_Access1");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv1() {
    run_cbmc_regression_test("01_cbmc_Fixedbv1");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv2() {
    run_cbmc_regression_test("01_cbmc_Fixedbv2");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv3() {
    run_cbmc_regression_test("01_cbmc_Fixedbv3");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv4() {
    run_cbmc_regression_test("01_cbmc_Fixedbv4");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv5() {
    run_cbmc_regression_test("01_cbmc_Fixedbv5");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv6() {
    run_cbmc_regression_test("01_cbmc_Fixedbv6");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv7() {
    run_cbmc_regression_test("01_cbmc_Fixedbv7");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv8() {
    run_cbmc_regression_test("01_cbmc_Fixedbv8");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv9() {
    run_cbmc_regression_test("01_cbmc_Fixedbv9");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv10() {
    run_cbmc_regression_test("01_cbmc_Fixedbv10");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv11() {
    run_cbmc_regression_test("01_cbmc_Fixedbv11");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv12() {
    run_cbmc_regression_test("01_cbmc_Fixedbv12");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv13() {
    run_cbmc_regression_test("01_cbmc_Fixedbv13");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv14() {
    run_cbmc_regression_test("01_cbmc_Fixedbv14");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv15() {
    run_cbmc_regression_test("01_cbmc_Fixedbv15");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv16() {
    run_cbmc_regression_test("01_cbmc_Fixedbv16");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv17() {
    run_cbmc_regression_test("01_cbmc_Fixedbv17");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv18() {
    run_cbmc_regression_test("01_cbmc_Fixedbv18");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv19() {
    run_cbmc_regression_test("01_cbmc_Fixedbv19");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv20() {
    run_cbmc_regression_test("01_cbmc_Fixedbv20");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv21() {
    run_cbmc_regression_test("01_cbmc_Fixedbv21");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv22() {
    run_cbmc_regression_test("01_cbmc_Fixedbv22");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv23() {
    run_cbmc_regression_test("01_cbmc_Fixedbv23");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv23_no_ir() {
    run_cbmc_regression_test("01_cbmc_Fixedbv23-no-ir");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv24() {
    run_cbmc_regression_test("01_cbmc_Fixedbv24");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv24_no_ir() {
    run_cbmc_regression_test("01_cbmc_Fixedbv24-no-ir");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv25() {
    run_cbmc_regression_test("01_cbmc_Fixedbv25");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv26() {
    run_cbmc_regression_test("01_cbmc_Fixedbv26");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv27() {
    run_cbmc_regression_test("01_cbmc_Fixedbv27");
}

#[ignore]
#[test]
fn test_01_cbmc_fixedbv28() {
    run_cbmc_regression_test("01_cbmc_Fixedbv28");
}

#[ignore]
#[test]
fn test_01_cbmc_enum1() {
    run_cbmc_regression_test("01_cbmc_enum1");
}

#[ignore]
#[test]
fn test_01_cbmc_enum2() {
    run_cbmc_regression_test("01_cbmc_enum2");
}

#[ignore]
#[test]
fn test_01_cbmc_enum3() {
    run_cbmc_regression_test("01_cbmc_enum3");
}

#[ignore]
#[test]
fn test_01_cbmc_for4() {
    run_cbmc_regression_test("01_cbmc_for4");
}

#[ignore]
#[test]
fn test_01_cbmc_free1() {
    run_cbmc_regression_test("01_cbmc_Free1");
}

#[ignore]
#[test]
fn test_01_cbmc_free2() {
    run_cbmc_regression_test("01_cbmc_Free2");
}

#[ignore]
#[test]
fn test_01_cbmc_free3() {
    run_cbmc_regression_test("01_cbmc_Free3");
}

#[ignore]
#[test]
fn test_01_cbmc_free4() {
    run_cbmc_regression_test("01_cbmc_Free4");
}

#[ignore]
#[test]
fn test_01_cbmc_free6() {
    run_cbmc_regression_test("01_cbmc_Free6");
}

#[ignore]
#[test]
fn test_01_cbmc_free7() {
    run_cbmc_regression_test("01_cbmc_Free7");
}

#[ignore]
#[test]
fn test_01_cbmc_free8() {
    run_cbmc_regression_test("01_cbmc_Free8");
}

#[ignore]
#[test]
fn test_01_cbmc_linked_list1() {
    run_cbmc_regression_test("01_cbmc_Linked_List1");
}

#[ignore]
#[test]
fn test_01_cbmc_linked_list2() {
    run_cbmc_regression_test("01_cbmc_Linked_List2");
}

#[ignore]
#[test]
fn test_01_cbmc_malloc1() {
    run_cbmc_regression_test("01_cbmc_Malloc1");
}

#[ignore]
#[test]
fn test_01_cbmc_malloc2() {
    run_cbmc_regression_test("01_cbmc_Malloc2");
}

#[ignore]
#[test]
fn test_01_cbmc_malloc3() {
    run_cbmc_regression_test("01_cbmc_Malloc3");
}

#[ignore]
#[test]
fn test_01_cbmc_malloc4() {
    run_cbmc_regression_test("01_cbmc_Malloc4");
}

#[ignore]
#[test]
fn test_01_cbmc_malloc5() {
    run_cbmc_regression_test("01_cbmc_Malloc5");
}

#[ignore]
#[test]
fn test_01_cbmc_malloc6() {
    run_cbmc_regression_test("01_cbmc_Malloc6");
}

#[ignore]
#[test]
fn test_01_cbmc_malloc7() {
    run_cbmc_regression_test("01_cbmc_Malloc7");
}

#[ignore]
#[test]
fn test_01_cbmc_malloc8() {
    run_cbmc_regression_test("01_cbmc_Malloc8");
}

#[ignore]
#[test]
fn test_01_cbmc_malloc9() {
    run_cbmc_regression_test("01_cbmc_Malloc9");
}

#[ignore]
#[test]
fn test_01_cbmc_malloc10() {
    run_cbmc_regression_test("01_cbmc_Malloc10");
}

#[ignore]
#[test]
fn test_01_cbmc_malloc11() {
    run_cbmc_regression_test("01_cbmc_Malloc11");
}

#[ignore]
#[test]
fn test_01_cbmc_malloc12() {
    run_cbmc_regression_test("01_cbmc_Malloc12");
}

#[ignore]
#[test]
fn test_01_cbmc_malloc13() {
    run_cbmc_regression_test("01_cbmc_Malloc13");
}

#[ignore]
#[test]
fn test_01_cbmc_malloc14() {
    run_cbmc_regression_test("01_cbmc_Malloc14");
}

#[ignore]
#[test]
fn test_01_cbmc_malloc15() {
    run_cbmc_regression_test("01_cbmc_Malloc15");
}

#[ignore]
#[test]
fn test_01_cbmc_malloc16() {
    run_cbmc_regression_test("01_cbmc_Malloc16");
}

#[ignore]
#[test]
fn test_01_cbmc_malloc17() {
    run_cbmc_regression_test("01_cbmc_Malloc17");
}

#[ignore]
#[test]
fn test_01_cbmc_malloc19() {
    run_cbmc_regression_test("01_cbmc_Malloc19");
}

#[ignore]
#[test]
fn test_01_cbmc_malloc20() {
    run_cbmc_regression_test("01_cbmc_Malloc20");
}

#[ignore]
#[test]
fn test_01_cbmc_malloc21() {
    run_cbmc_regression_test("01_cbmc_Malloc21");
}

#[ignore]
#[test]
fn test_01_cbmc_malloc22() {
    run_cbmc_regression_test("01_cbmc_Malloc22");
}

#[ignore]
#[test]
fn test_01_cbmc_malloc23() {
    run_cbmc_regression_test("01_cbmc_Malloc23");
}

#[ignore]
#[test]
fn test_01_cbmc_malloc24() {
    run_cbmc_regression_test("01_cbmc_Malloc24");
}

#[ignore]
#[test]
fn test_01_cbmc_malloc25() {
    run_cbmc_regression_test("01_cbmc_Malloc25");
}

#[ignore]
#[test]
fn test_01_cbmc_malloc27() {
    run_cbmc_regression_test("01_cbmc_Malloc27");
}

#[ignore]
#[test]
fn test_01_cbmc_malloc28() {
    run_cbmc_regression_test("01_cbmc_Malloc28");
}

#[ignore]
#[test]
fn test_01_cbmc_pointer25() {
    run_cbmc_regression_test("01_cbmc_Pointer25");
}

#[ignore]
#[test]
fn test_01_cbmc_pointer26() {
    run_cbmc_regression_test("01_cbmc_Pointer26");
}

#[ignore]
#[test]
fn test_01_cbmc_function_pointer8() {
    run_cbmc_regression_test("01_cbmc_Function_Pointer8");
}

#[ignore]
#[test]
fn test_01_cbmc_pointer8() {
    run_cbmc_regression_test("01_cbmc_Pointer8");
}

#[ignore]
#[test]
fn test_01_cbmc_structs4() {
    run_cbmc_regression_test("01_cbmc_Structs4");
}

#[ignore]
#[test]
fn test_01_cbmc_array_initialization2() {
    run_cbmc_regression_test("01_cbmc_Array_Initialization2");
}

#[ignore]
#[test]
fn test_01_cbmc_array_initialization3() {
    run_cbmc_regression_test("01_cbmc_Array_Initialization3");
}

#[ignore]
#[test]
fn test_01_cbmc_array_pointer1() {
    run_cbmc_regression_test("01_cbmc_Array_Pointer1");
}

#[ignore]
#[test]
fn test_01_cbmc_array_pointer2() {
    run_cbmc_regression_test("01_cbmc_Array_Pointer2");
}

#[ignore]
#[test]
fn test_01_cbmc_array_pointer3() {
    run_cbmc_regression_test("01_cbmc_Array_Pointer3");
}

#[ignore]
#[test]
fn test_01_cbmc_bitfields() {
    run_cbmc_regression_test("01_cbmc_Bitfields");
}

#[ignore]
#[test]
fn test_01_cbmc_bool1() {
    run_cbmc_regression_test("01_cbmc_Bool1");
}

#[ignore]
#[test]
fn test_01_cbmc_boolean_guards1() {
    run_cbmc_regression_test("01_cbmc_Boolean_Guards1");
}

#[ignore]
#[test]
fn test_01_cbmc_bv_arithmetic2() {
    run_cbmc_regression_test("01_cbmc_BV_Arithmetic2");
}

#[ignore]
#[test]
fn test_01_cbmc_bv_arithmetic4() {
    run_cbmc_regression_test("01_cbmc_BV_Arithmetic4");
}

#[ignore]
#[test]
fn test_01_cbmc_character_handling1() {
    run_cbmc_regression_test("01_cbmc_character_handling1");
}

#[ignore]
#[test]
fn test_01_cbmc_defines1() {
    run_cbmc_regression_test("01_cbmc_Defines1");
}

#[ignore]
#[test]
fn test_01_cbmc_end_thread1() {
    run_cbmc_regression_test("01_cbmc_End_thread1");
}

#[ignore]
#[test]
fn test_01_cbmc_error_label1() {
    run_cbmc_regression_test("01_cbmc_error-label1");
}

#[ignore]
#[test]
fn test_01_cbmc_exit1() {
    run_cbmc_regression_test("01_cbmc_exit1");
}

#[ignore]
#[test]
fn test_01_cbmc_externfixedbv1() {
    run_cbmc_regression_test("01_cbmc_ExternFixedbv1");
}

#[ignore]
#[test]
fn test_01_cbmc_function_option1() {
    run_cbmc_regression_test("01_cbmc_function_option1");
}

#[ignore]
#[test]
fn test_01_cbmc_function_pointer2() {
    run_cbmc_regression_test("01_cbmc_Function_Pointer2");
}

#[ignore]
#[test]
fn test_01_cbmc_function_pointer6() {
    run_cbmc_regression_test("01_cbmc_Function_Pointer6");
}

#[ignore]
#[test]
fn test_01_cbmc_function_pointer7() {
    run_cbmc_regression_test("01_cbmc_Function_Pointer7");
}

#[ignore]
#[test]
fn test_01_cbmc_function5() {
    run_cbmc_regression_test("01_cbmc_Function5");
}

#[ignore]
#[test]
fn test_01_cbmc_function8() {
    run_cbmc_regression_test("01_cbmc_Function8");
}

#[ignore]
#[test]
fn test_01_cbmc_function9() {
    run_cbmc_regression_test("01_cbmc_Function9");
}

#[ignore]
#[test]
fn test_01_cbmc_gcc_conditional_expr1() {
    run_cbmc_regression_test("01_cbmc_gcc_conditional_expr1");
}

#[ignore]
#[test]
fn test_01_cbmc_global_initialization() {
    run_cbmc_regression_test("01_cbmc_Global_Initialization");
}

#[ignore]
#[test]
fn test_01_cbmc_multi_dimensional_array1() {
    run_cbmc_regression_test("01_cbmc_Multi_Dimensional_Array1");
}

#[ignore]
#[test]
fn test_01_cbmc_negation() {
    run_cbmc_regression_test("01_cbmc_Negation");
}

#[ignore]
#[test]
fn test_01_cbmc_overflow_addition1() {
    run_cbmc_regression_test("01_cbmc_Overflow_Addition1");
}

#[ignore]
#[test]
fn test_01_cbmc_pointer2() {
    run_cbmc_regression_test("01_cbmc_Pointer2");
}

#[ignore]
#[test]
fn test_01_cbmc_pointer5() {
    run_cbmc_regression_test("01_cbmc_Pointer5");
}

#[ignore]
#[test]
fn test_01_cbmc_pointer9() {
    run_cbmc_regression_test("01_cbmc_Pointer9");
}

#[ignore]
#[test]
fn test_01_cbmc_pointer11() {
    run_cbmc_regression_test("01_cbmc_Pointer11");
}

#[ignore]
#[test]
fn test_01_cbmc_pointer13() {
    run_cbmc_regression_test("01_cbmc_Pointer13");
}

#[ignore]
#[test]
fn test_01_cbmc_pointer16() {
    run_cbmc_regression_test("01_cbmc_Pointer16");
}

#[ignore]
#[test]
fn test_01_cbmc_pointer20() {
    run_cbmc_regression_test("01_cbmc_Pointer20");
}

#[ignore]
#[test]
fn test_01_cbmc_pointer24() {
    run_cbmc_regression_test("01_cbmc_Pointer24");
}

#[ignore]
#[test]
fn test_01_cbmc_pointer31() {
    run_cbmc_regression_test("01_cbmc_Pointer31");
}

#[ignore]
#[test]
fn test_01_cbmc_pointer32() {
    run_cbmc_regression_test("01_cbmc_Pointer32");
}

#[ignore]
#[test]
fn test_01_cbmc_return2() {
    run_cbmc_regression_test("01_cbmc_return2");
}



#[ignore]
#[test]
fn test_01_cbmc_sideeffects1() {
    run_cbmc_regression_test("01_cbmc_Sideeffects1");
}



#[ignore]
#[test]
fn test_01_cbmc_statement_expression2() {
    run_cbmc_regression_test("01_cbmc_Statement_Expression2");
}

#[ignore]
#[test]
fn test_01_cbmc_string1() {
    run_cbmc_regression_test("01_cbmc_String1");
}

#[test]
fn test_01_cbmc_string2() {
    run_cbmc_regression_test("01_cbmc_String2") 
}

#[test]
fn test_01_cbmc_string3() {
    run_cbmc_regression_test("01_cbmc_String3")
}

#[ignore]
#[test]
fn test_01_cbmc_string4() {
    run_cbmc_regression_test("01_cbmc_String4");
}

#[ignore]
#[test]
fn test_01_cbmc_struct_array1() {
    run_cbmc_regression_test("01_cbmc_Struct_Array1");
}

#[ignore]
#[test]
fn test_01_cbmc_struct_bytewise1() {
    run_cbmc_regression_test("01_cbmc_Struct_Bytewise1");
}

#[test]
fn test_01_cbmc_struct_bytewise2() {
    run_cbmc_regression_test("01_cbmc_Struct_Bytewise2")
}

#[ignore]
#[test]
fn test_01_cbmc_struct_bytewise3() {
    run_cbmc_regression_test("01_cbmc_Struct_Bytewise3");
}

#[test]
fn test_01_cbmc_struct_hierarchy1() {
    run_cbmc_regression_test("01_cbmc_Struct_Hierarchy1")
}

#[ignore]
#[test]
fn test_01_cbmc_struct_initialization2() {
    run_cbmc_regression_test("01_cbmc_Struct_Initialization2");
}

#[test]
fn test_01_cbmc_struct_initialization3() {
    run_cbmc_regression_test("01_cbmc_Struct_Initialization3")
}

#[test]
fn test_01_cbmc_struct_pointer1() {
    run_cbmc_regression_test("01_cbmc_Struct_Pointer1")
}

#[ignore]
#[test]
fn test_01_cbmc_structs5() {
    run_cbmc_regression_test("01_cbmc_Structs5");
}

#[ignore]
#[test]
fn test_01_cbmc_structs6() {
    run_cbmc_regression_test("01_cbmc_Structs6");
}

#[ignore]
#[test]
fn test_01_cbmc_structs7() {
    run_cbmc_regression_test("01_cbmc_Structs7");
}

#[ignore]
#[test]
fn test_01_cbmc_unbounded_array1() {
    run_cbmc_regression_test("01_cbmc_Unbounded_Array1");
}

#[ignore]
#[test]
fn test_01_cbmc_switch2() {
    run_cbmc_regression_test("01_cbmc_switch2");
}

#[ignore]
#[test]
fn test_01_cbmc_unsigned_char1() {
    run_cbmc_regression_test("01_cbmc_unsigned_char1");
}

// Disabled due to needed linking

#[ignore]
#[test]
fn test_01_cbmc_static_functions1() {
    run_cbmc_regression_test("01_cbmc_Static_Functions1");
}

#[ignore]
#[test]
fn test_01_cbmc_linking1() {
    run_cbmc_regression_test("01_cbmc_Linking1");
}

#[ignore]
#[test]
fn test_01_cbmc_linking2() {
    run_cbmc_regression_test("01_cbmc_Linking2");
}

#[ignore]
#[test]
fn test_01_cbmc_inline1() {
    run_cbmc_regression_test("01_cbmc_inline1");
}

#[ignore]
#[test]
fn test_01_cbmc_free5() {
    run_cbmc_regression_test("01_cbmc_Free5");
}
