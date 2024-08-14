use vvcapi::{VoicevoxCore, InitializeOptions};
use std::io::Write;

fn main() {
    let open_jtalk_dict_dir = String::from("./voicevox_core/open_jtalk_dic_utf_8-1.11");
    let core = VoicevoxCore::new(InitializeOptions {
        open_jtalk_dict_dir,
        ..Default::default()
    }).unwrap();
    let text = "絶え間なくそそぐ愛の名を";
    let speaker_id = 1;

    core.load_model(speaker_id).unwrap();

    let wav = core.tts(text, speaker_id).unwrap();
    let mut file = std::fs::File::create("output.wav").unwrap();
    file.write_all(&wav).unwrap();
}
