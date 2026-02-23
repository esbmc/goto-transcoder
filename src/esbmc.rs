use crate::bytereader::ByteReader;
use crate::irep::Irept;

#[derive(Clone, Debug)]
pub struct ESBMCParseResult {
    #[allow(dead_code)]
    pub reader: ByteReader,
    pub symbols_irep: Vec<Irept>,
    pub functions_irep: Vec<(String, Irept)>,
}

#[allow(dead_code)]
pub fn process_esbmc_file(path: &str) -> Result<ESBMCParseResult, String> {
    let mut result = ESBMCParseResult {
        reader: ByteReader::read_file(path),
        functions_irep: Vec::new(),
        symbols_irep: Vec::new(),
    };

    result
        .reader
        .check_esbmc_header()
        .expect("invalid ESBMC header — is this a .goto file?");
    result
        .reader
        .check_esbmc_version()
        .expect("unsupported ESBMC version");

    // Symbol table
    let number_of_symbols = result.reader.read_esbmc_word();
    for _ in 0..number_of_symbols {
        let symbol = result.reader.read_esbmc_reference();
        result.symbols_irep.push(symbol.clone());
    }

    // Functions
    let number_of_functions = result.reader.read_esbmc_word();
    for _ in 0..number_of_functions {
        let function = (
            result.reader.read_esbmc_string(),
            result.reader.read_esbmc_reference(),
        );
        result.functions_irep.push(function.clone());
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytewriter::ByteWriter;

    #[test]
    fn test_file() {
        let cargo_dir = match std::env::var("CARGO_MANIFEST_DIR") {
            Ok(v) => v,
            Err(err) => panic!("Could not open cargo folder. {}", err),
        };
        let test_path = std::path::Path::new(&cargo_dir).join("resources/test/hello.goto");
        assert!(test_path.exists());

        let result = process_esbmc_file(test_path.to_str().unwrap()).unwrap();

        std::fs::remove_file("/tmp/test.goto").ok();
        ByteWriter::write_to_file(result.symbols_irep, result.functions_irep, "/tmp/test.goto");
    }
}
