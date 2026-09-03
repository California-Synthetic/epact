use std::{env, error::Error, fs, io::Write, process::ExitCode};

use epact_compiler::{compile_program, verify_program_image};
use epact_protocol::{
    canonical_epact_json_bytes, EpactOperationRequest, EpactProgram, EpactProgramImage,
    EpactRuntimeEvent, EpactRuntimeState,
};
use epact_runtime::{evaluate_epact_operation, replay_epact_events};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("epact: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("compile") => {
            let path = one_path(&mut args)?;
            let program: EpactProgram = read_json(&path)?;
            write_json(&compile_program(program)?)?;
        }
        Some("verify-image") => {
            let path = one_path(&mut args)?;
            let image: EpactProgramImage = read_json(&path)?;
            verify_program_image(&image)?;
            println!("{}", image.image_sha256);
        }
        Some("replay") => {
            let image_path = args.next().ok_or(USAGE)?;
            let events_path = args.next().ok_or(USAGE)?;
            require_end(&mut args)?;
            let image: EpactProgramImage = read_json(&image_path)?;
            let events: Vec<EpactRuntimeEvent> = read_json(&events_path)?;
            write_json(&replay_epact_events(&image, &events)?)?;
        }
        Some("evaluate") => {
            let image_path = args.next().ok_or(USAGE)?;
            let events_path = args.next().ok_or(USAGE)?;
            let request_path = args.next().ok_or(USAGE)?;
            require_end(&mut args)?;
            let image: EpactProgramImage = read_json(&image_path)?;
            let events: Vec<EpactRuntimeEvent> = read_json(&events_path)?;
            let request: EpactOperationRequest = read_json(&request_path)?;
            let state: EpactRuntimeState = replay_epact_events(&image, &events)?;
            write_json(&evaluate_epact_operation(&image, &state, &request)?)?;
        }
        _ => return Err(USAGE.into()),
    }
    Ok(())
}

fn one_path(args: &mut impl Iterator<Item = String>) -> Result<String, Box<dyn Error>> {
    let path = args.next().ok_or(USAGE)?;
    require_end(args)?;
    Ok(path)
}

fn require_end(args: &mut impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    if args.next().is_some() {
        return Err(USAGE.into());
    }
    Ok(())
}

fn read_json<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, Box<dyn Error>> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn write_json(value: &impl serde::Serialize) -> Result<(), Box<dyn Error>> {
    std::io::stdout().write_all(&canonical_epact_json_bytes(value)?)?;
    println!();
    Ok(())
}

const USAGE: &str = "usage:\n  epact compile <program.json>\n  epact verify-image <image.json>\n  epact replay <image.json> <events.json>\n  epact evaluate <image.json> <events.json> <request.json>";
