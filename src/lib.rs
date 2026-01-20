use nih_plug::{prelude::*, util::db_to_gain};
use std::num::NonZeroU32;
use std::sync::Arc;

#[inline(always)]
fn hard_clip(signal: f32, ceiling: f32) -> f32 {
    let ceiling = ceiling.max(1.0e-12);
    signal.clamp(-ceiling, ceiling)
}
#[inline(always)]
fn soft_clip(signal: f32, ceiling: f32) -> f32 {
    let ceiling = ceiling.max(1.0e-12);
    let x = (signal / ceiling).tanh();
    x * ceiling
}

#[derive(Enum, Copy, Clone, Debug, PartialEq, Eq)]
enum ClippingModes {
    HardClip,
    SoftClip,
}

struct RClip {
    params: Arc<PluginParams>,
}

#[derive(Params)]
struct PluginParams {
    #[id = "mode"]
    pub mode: EnumParam<ClippingModes>,

    #[id = "gain"]
    pub gain: FloatParam,

    #[id = "threshold"]
    pub threshold: FloatParam,

    #[id = "delta"]
    pub delta: BoolParam,
}

impl Default for RClip {
    fn default() -> Self {
        Self {
            params: Arc::new(PluginParams::default()),
        }
    }
}

impl Default for PluginParams {
    fn default() -> Self {
        Self {
            mode: EnumParam::new("Mode", ClippingModes::HardClip),
            gain: FloatParam::new(
                "Gain",
                0.0,
                FloatRange::Linear {
                    min: -12.0,
                    max: 12.0,
                },
            )
            .with_step_size(0.1)
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" dB"),

            threshold: FloatParam::new(
                "Threshold",
                0.0,
                FloatRange::Linear {
                    min: -24.0,
                    max: 0.0,
                },
            )
            .with_step_size(0.1)
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" dB"),

            delta: BoolParam::new("Delta", false),
        }
    }
}

impl Plugin for RClip {
    const NAME: &'static str = "rClip";
    const VENDOR: &'static str = "gobin";
    const URL: &'static str = "https://example.com";
    const EMAIL: &'static str = "example@example.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            aux_input_ports: &[],
            aux_output_ports: &[],
            names: PortNames::const_default(),
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        true
    }

    fn reset(&mut self) {}

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let delta = self.params.delta.value();
        let mode = self.params.mode.value();

        for channel_samples in buffer.iter_samples() {
            let gain_db = self.params.gain.smoothed.next();
            let gain = db_to_gain(gain_db);

            let threshold_db = self.params.threshold.smoothed.next();
            let ceiling = db_to_gain(threshold_db);

            for sample in channel_samples {
                let dry = *sample;

                let signal = dry * gain;
                let wet = match mode {
                    ClippingModes::HardClip => hard_clip(signal, ceiling),
                    ClippingModes::SoftClip => soft_clip(signal, ceiling),
                };
                *sample = if delta { wet - dry } else { wet };
            }
        }

        ProcessStatus::Normal
    }

    // This can be used for cleaning up special resources like socket connections whenever the
    // plugin is deactivated. Most plugins won't need to do anything here.
    fn deactivate(&mut self) {}
}

impl ClapPlugin for RClip {
    const CLAP_ID: &'static str = "com.gobin.RClip";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A clipping plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for RClip {
    const VST3_CLASS_ID: [u8; 16] = *b"MStecktechPlugin";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_clap!(RClip);
nih_export_vst3!(RClip);
