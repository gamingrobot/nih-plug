use nih_plug::prelude::*;
use parking_lot::RwLock;
use std::sync::Arc;

struct Gain {
    params: Arc<GainParams>,
}

/// The [`Params`] derive macro gathers all of the information needed for the wrapper to know about
/// the plugin's parameters, persistent serializable fields, and nested parameter groups. You can
/// also easily implement [`Params`] by hand if you want to, for instance, have multiple instances
/// of a parameters struct for multiple identical oscillators/filters/envelopes.
#[derive(Params)]
struct GainParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined.
    #[id = "gain"]
    pub gain: FloatParam,

    /// This field isn't used in this example, but anything written to the vector would be restored
    /// together with a preset/state file saved for this plugin. This can be useful for storing
    /// things like sample data.
    #[persist = "industry_secrets"]
    pub random_data: RwLock<Vec<f32>>,

    /// You can also nest parameter structs. These will appear as a separate nested group if your
    /// DAW displays parameters in a tree structure.
    #[nested = "Subparameters"]
    pub sub_params: SubParams,
}

#[derive(Params)]
struct SubParams {
    #[id = "thing"]
    pub nested_parameter: FloatParam,

    #[nested = "Sub-Subparameters"]
    pub sub_sub_params: SubSubParams,
}

#[derive(Params)]
struct SubSubParams {
    #[id = "noope"]
    pub nope: FloatParam,
}

impl Default for Gain {
    fn default() -> Self {
        Self {
            params: Arc::new(GainParams::default()),
        }
    }
}

impl Default for GainParams {
    fn default() -> Self {
        Self {
            gain: FloatParam::new(
                "Gain",
                0.0,
                FloatRange::Linear {
                    min: -30.0,
                    max: 30.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_step_size(0.01)
            .with_unit(" dB")
            // This is actually redundant, because a step size of two decimal places already
            // causes the parameter to shown rounded
            .with_value_to_string(formatters::v2s_f32_rounded(2)),
            // Persisted fields can be intialized like any other fields, and they'll keep their
            // values when restoring the plugin's state.
            random_data: RwLock::new(Vec::new()),
            sub_params: SubParams {
                nested_parameter: FloatParam::new(
                    "Unused Nested Parameter",
                    0.5,
                    FloatRange::Skewed {
                        min: 2.0,
                        max: 2.4,
                        factor: FloatRange::skew_factor(2.0),
                    },
                )
                .with_value_to_string(formatters::v2s_f32_rounded(2)),
                sub_sub_params: SubSubParams {
                    nope: FloatParam::new("Nope", 0.5, FloatRange::Linear { min: 1.0, max: 2.0 }),
                },
            },
        }
    }
}

impl Plugin for Gain {
    const NAME: &'static str = "Gain";
    const VENDOR: &'static str = "Moist Plugins GmbH";
    const URL: &'static str = "https://youtu.be/dQw4w9WgXcQ";
    const EMAIL: &'static str = "info@example.com";

    const VERSION: &'static str = "0.0.1";

    const DEFAULT_NUM_INPUTS: u32 = 2;
    const DEFAULT_NUM_OUTPUTS: u32 = 2;

    const DEFAULT_AUX_INPUTS: Option<AuxiliaryIOConfig> = None;
    const DEFAULT_AUX_OUTPUTS: Option<AuxiliaryIOConfig> = None;

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    // Setting this to `true` will tell the wrapper to split the buffer up into smaller blocks
    // whenever there are inter-buffer parameter changes. This way no changes to the plugin are
    // required to support sample accurate automation and the wrapper handles all of the boring
    // stuff like making sure transport and other timing information stays consistent between the
    // splits.
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn accepts_bus_config(&self, config: &BusConfig) -> bool {
        // This works with any symmetrical IO layout
        config.num_input_channels == config.num_output_channels && config.num_input_channels > 0
    }

    // This plugin doesn't need any special initialization, but if you need to do anything expensive
    // then this would be the place. State is kept around when the host reconfigures the
    // plugin. If we do need special initialization, we could implement the `initialize()` and/or
    // `reset()` methods

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            // Smoothing is optionally built into the parameters themselves
            let gain = self.params.gain.smoothed.next();

            for sample in channel_samples {
                *sample *= util::db_to_gain(gain);
            }
        }

        ProcessStatus::Normal
    }

    // This can be used for cleaning up special resources like socket connections whenever the
    // plugin is deactivated. Most plugins won't need to do anything here.
    fn deactivate(&mut self) {}
}

impl ClapPlugin for Gain {
    const CLAP_ID: &'static str = "com.moist-plugins-gmbh.gain";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A smoothed gain parameter example plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for Gain {
    const VST3_CLASS_ID: [u8; 16] = *b"GainMoistestPlug";
    const VST3_CATEGORIES: &'static str = "Fx|Dynamics";
}

nih_export_clap!(Gain);
nih_export_vst3!(Gain);
