pub struct Whisper {}

use std::io::prelude::*;
impl Whisper {
    pub fn generate_script(audioname: &str) -> String {
        format!(
            "import whisper\nmodel = whisper.load_model('tiny.en')\naudio = whisper.load_audio('{}')\noptions = whisper.DecodingOptions(language='English')\nwhisper.transcribe(model, audio, verbose=True, language='English')",
            audioname
        )
    }
    pub fn process(url: &str) {
        let filter = regex::Regex::new("their").unwrap();
        println!("starting Whisper");
        let mut process = std::process::Command::new("python")
            .arg("-c")
            .arg(Self::generate_script(url))
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let out_handle = process.stdout.take().unwrap();
        let reader = std::io::BufReader::new(out_handle);
        reader
            .lines()
            .filter_map(|line| line.ok())
            .filter(|line| filter.is_match(line))
            .for_each(|line| println!("{}", line));
        println!("Finishing whisper");
    }
}
