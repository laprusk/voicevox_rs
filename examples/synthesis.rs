use vvcapi::{VoicevoxCore, InitializeOptions};
use std::io::Write;

fn main() {
    let open_jtalk_dict_dir = String::from("./voicevox_core/open_jtalk_dic_utf_8-1.11");
    let core = VoicevoxCore::new(InitializeOptions {
        open_jtalk_dict_dir,
        ..Default::default()
    }).unwrap();
    let text = "永遠と、呼ぶことが、できたなら";
    let speaker_id = 1;

    core.load_model(speaker_id).unwrap();

    let mut aq = core.audio_query(text, speaker_id).unwrap();
    aq.speed_scale += 0.3;
    aq.pitch_scale += 0.1;
    let wav = core.synthesis(aq, speaker_id).unwrap();

    let mut file = std::fs::File::create("output.wav").unwrap();
    file.write_all(&wav).unwrap();
}
