use std::io::prelude::*;
fn generate_script(audioname: &str, language: Option<&str>) -> String {
    let language = match language {
        Some(language) => format!("'{}'", language),
        None => String::from("None"),
    };
    format!(
            "import whisper\nmodel = whisper.load_model('tiny')\naudio = whisper.load_audio('{}')\nwhisper.transcribe(model, audio, verbose=True, language={})",
            audioname,
            language
        )
}
pub fn process(
    task: &oqueue::Task,
    title: &dyn std::fmt::Display,
    url: &str,
    language: Option<&str>,
    filter: &regex::Regex,
) {
    let mut process = std::process::Command::new("python")
        .arg("-c")
        .arg(generate_script(url, language))
        .stderr(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let out_handle = process.stdout.take().unwrap();
    let reader = std::io::BufReader::new(out_handle);
    let mut displayed_title = false;
    for line in reader
        .lines()
        .filter_map(|line| line.ok())
        .filter(|line| filter.is_match(line))
    {
        if !displayed_title {
            writeln!(task, "{}", title);
            displayed_title = true;
        }
        writeln!(task, "{}", line)
    }
}

fn has_whisper() -> bool {
    let process = std::process::Command::new("python")
        .arg("-c")
        .arg("import whisper")
        .output();
    if let Ok(output) = process {
        if !output.status.success() {
            println!("Error: Whisper is not installed");
            print!("Would you like to install it (y/N)? ");
            std::io::stdout().flush().unwrap();
            let mut response = String::new();
            std::io::stdin().read_line(&mut response).unwrap();
            response.make_ascii_lowercase();
            if response.trim() == "y" {
                let process = std::process::Command::new("python")
                    .arg("-m")
                    .arg("pip")
                    .arg("install")
                    .arg("whisper")
                    .spawn();
                if let Ok(mut output) = process {
                    let output = output.wait();
                    if let Ok(status) = output {
                        if !status.success() {
                            eprintln!("\nInstallation failed!\n");
                        } else {
                            println!("\nInstallation successful\n");
                            return true;
                        }
                    } else if let Err(e) = output {
                        eprintln!("{}", e);
                    }
                } else if let Err(e) = process {
                    eprintln!("{}", e);
                }
            } else {
                eprintln!("Exiting...");
            }
        } else {
            return true;
        }
    } else if let Err(e) = process {
        eprintln!("Error: {}", e);
    }
    return false;
}

pub fn check_whisper() -> bool {
    let process = std::process::Command::new("python")
        .arg("--version")
        .output();
    match process {
        Ok(version) => {
            print!("Utilizing {}", String::from_utf8(version.stdout).unwrap());
            return has_whisper();
        }
        Err(e) => {
            println!(
                "Error: {}\nIn order to use this feature, you must have Python\nDownload: https://www.python.org/downloads/",
                e
            );
            return false;
        }
    }
}
