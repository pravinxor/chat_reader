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
