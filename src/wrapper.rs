use crate::bindings::*;
use serde::{Deserialize, Serialize};
use std::ffi::CString;

#[derive(Debug)]
pub enum AccelerationMode {
    Auto,
    Cpu,
    Gpu,
}

#[derive(Debug)]
pub struct InitializeOptions {
    pub acceleration_mode: AccelerationMode,
    pub cpu_num_threads: u16,
    pub load_all_models: bool,
    pub open_jtalk_dict_dir: String,
}

impl Default for InitializeOptions {
    fn default() -> Self {
        let default_options = unsafe { voicevox_make_default_initialize_options() };
        Self {
            acceleration_mode: match default_options.acceleration_mode {
                0 => AccelerationMode::Auto,
                1 => AccelerationMode::Cpu,
                2 => AccelerationMode::Gpu,
                _ => AccelerationMode::Auto,
            },
            cpu_num_threads: default_options.cpu_num_threads,
            load_all_models: default_options.load_all_models,
            open_jtalk_dict_dir: "".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mora {
    pub text: String,
    pub consonant: Option<String>,
    pub consonant_length: Option<f32>,
    pub vowel: String,
    pub vowel_length: f64,
    pub pitch: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccentPhrase {
    pub moras: Vec<Mora>,
    pub accent: i32,
    pub pause_mora: Option<Mora>,
    pub is_interrogative: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioQuery {
    pub accent_phrases: Vec<AccentPhrase>,
    pub speed_scale: f64,
    pub pitch_scale: f64,
    pub intonation_scale: f64,
    pub volume_scale: f64,
    pub pre_phoneme_length: f64,
    pub post_phoneme_length: f64,
    pub output_sampling_rate: i32,
    pub output_stereo: bool,
    pub kana: String,
}

pub struct VoicevoxCore;

impl VoicevoxCore {
    pub fn new(options: InitializeOptions) -> Result<Self, VoicevoxResultCode> {
        let open_jtalk_dict_dir = CString::new(options.open_jtalk_dict_dir).unwrap();
        let options = VoicevoxInitializeOptions {
            acceleration_mode: match options.acceleration_mode {
                AccelerationMode::Auto => 0,
                AccelerationMode::Cpu => 1,
                AccelerationMode::Gpu => 2,
            },
            cpu_num_threads: options.cpu_num_threads,
            load_all_models: options.load_all_models,
            open_jtalk_dict_dir: open_jtalk_dict_dir.as_ptr(),
        };
        let result = unsafe { voicevox_initialize(options) };

        match result {
            0 => Ok(Self {}),
            _ => Err(result),
        }
    }

    pub fn get_version(&self) -> String {
        let version = unsafe { voicevox_get_version() };
        unsafe { std::ffi::CStr::from_ptr(version).to_str().unwrap().to_string() }
    }

    pub fn load_model(&self, speaker_id: u32) -> Result<(), VoicevoxResultCode> {
        let result = unsafe { voicevox_load_model(speaker_id) };

        match result {
            0 => Ok(()),
            _ => Err(result),
        }
    }

    pub fn is_gpu_mode(&self) -> bool {
        unsafe { voicevox_is_gpu_mode() }
    }

    pub fn is_model_loaded(&self, speaker_id: u32) -> bool {
        unsafe { voicevox_is_model_loaded(speaker_id) }
    }

    pub fn audio_query(
        &self,
        text: &str,
        speaker_id: u32
    ) -> Result<AudioQuery, VoicevoxResultCode> {
        let cstr = CString::new(text).unwrap();
        let mut output: *mut std::os::raw::c_char = std::ptr::null_mut();
        let options = unsafe { voicevox_make_default_audio_query_options() };
        let result = unsafe {
            voicevox_audio_query(cstr.as_ptr(), speaker_id, options, &mut output)
        };

        match result {
            0 => {
                let output_str = unsafe { std::ffi::CStr::from_ptr(output) };
                let output_str = output_str.to_str().unwrap();
                match serde_json::from_str::<AudioQuery>(output_str) {
                    Ok(audio_query) => Ok(audio_query),
                    Err(e) => {
                        eprintln!("Failed to parse JSON: {}", e);
                        eprintln!("JSON data: {}", output_str);
                        Err(VoicevoxResultCode::from(1))
                    }
                }
            },
            _ => Err(result),
        }
    }

    pub fn synthesis(
        &self,
        audio_query: AudioQuery,
        speaker_id: u32
    ) -> Result<Vec<u8>, VoicevoxResultCode> {
        let audio_query_json = match serde_json::to_string(&audio_query) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("Failed to serialize AudioQuery: {}", e);
                return Err(VoicevoxResultCode::from(1));
            }
        };
        let audio_query_json = CString::new(audio_query_json).unwrap();
        let options = unsafe { voicevox_make_default_synthesis_options() };
        let mut output_wav_length: usize = 0;
        let mut output_wav: *mut u8 = std::ptr::null_mut();
        
        let result = unsafe {
            voicevox_synthesis(audio_query_json.as_ptr(), speaker_id, options,
                &mut output_wav_length, &mut output_wav)
        };

        match result {
            0 => {
                let output_wav = unsafe {
                    std::slice::from_raw_parts(output_wav, output_wav_length)
                };
                Ok(output_wav.to_vec())
            },
            _ => Err(result),
        }
    }

    pub fn tts(
        &self,
        text: &str,
        speaker_id: u32
    ) -> Result<Vec<u8>, VoicevoxResultCode> {
        let cstr = CString::new(text).unwrap();
        let options = unsafe { voicevox_make_default_tts_options() };
        let mut output_wav_length: usize = 0;
        let mut output_wav: *mut u8 = std::ptr::null_mut();

        let result = unsafe {
            voicevox_tts(cstr.as_ptr(), speaker_id, options, &mut output_wav_length, &mut output_wav)
        };

        match result {
            0 => {
                let output_wav = unsafe {
                    std::slice::from_raw_parts(output_wav, output_wav_length)
                };
                Ok(output_wav.to_vec())
            },
            _ => Err(result),
        }
    }   
}

impl Drop for VoicevoxCore {
    fn drop(&mut self) {
        unsafe { voicevox_finalize() };
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let initialize_options = InitializeOptions {
            open_jtalk_dict_dir: "./voicevox_core/open_jtalk_dic_utf_8-1.11".to_string(),
            ..Default::default()
        };
        println!("{:?}", initialize_options);
        let core = VoicevoxCore::new(initialize_options);
        assert!(core.is_ok());
    }

    #[test]
    fn test_load_model() {
        let core = VoicevoxCore::new(InitializeOptions {
            open_jtalk_dict_dir: "./voicevox_core/open_jtalk_dic_utf_8-1.11".to_string(),
            ..Default::default()
        }).unwrap();
        let result = core.load_model(1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_audio_query() {
        let core = VoicevoxCore::new(InitializeOptions {
            open_jtalk_dict_dir: "./voicevox_core/open_jtalk_dic_utf_8-1.11".to_string(),
            ..Default::default()
        }).unwrap();
        core.load_model(1).unwrap();
        let aq = core.audio_query("こんにちは", 1);
        assert!(aq.is_ok());
    }

    #[test]
    fn test_synthesis() {
        let core = VoicevoxCore::new(InitializeOptions {
            open_jtalk_dict_dir: "./voicevox_core/open_jtalk_dic_utf_8-1.11".to_string(),
            ..Default::default()
        }).unwrap();
        core.load_model(1).unwrap();
        let aq = core.audio_query("こんにちは", 1).unwrap();
        let result = core.synthesis(aq, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_tts() {
        let core = VoicevoxCore::new(InitializeOptions {
            open_jtalk_dict_dir: "./voicevox_core/open_jtalk_dic_utf_8-1.11".to_string(),
            ..Default::default()
        }).unwrap();
        core.load_model(1).unwrap();
        let result = core.tts("こんにちは", 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_version() {
        let core = VoicevoxCore::new(InitializeOptions {
            open_jtalk_dict_dir: "./voicevox_core/open_jtalk_dic_utf_8-1.11".to_string(),
            ..Default::default()
        }).unwrap();
        let version = core.get_version();
        println!("{}", version);
        assert!(version.len() > 0);
    }

    #[test]
    fn test_is_gpu_mode() {
        let core = VoicevoxCore::new(InitializeOptions {
            open_jtalk_dict_dir: "./voicevox_core/open_jtalk_dic_utf_8-1.11".to_string(),
            ..Default::default()
        }).unwrap();
        let is_gpu_mode = core.is_gpu_mode();
        println!("{}", is_gpu_mode);
        assert!(is_gpu_mode == false);
    }

    #[test]
    fn test_is_model_loaded() {
        let core = VoicevoxCore::new(InitializeOptions {
            open_jtalk_dict_dir: "./voicevox_core/open_jtalk_dic_utf_8-1.11".to_string(),
            ..Default::default()
        }).unwrap();
        core.load_model(1).unwrap();
        let is_model_loaded = core.is_model_loaded(1);
        println!("{}", is_model_loaded);
        assert!(is_model_loaded == true);
    }
}