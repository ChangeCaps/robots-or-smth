use crate::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AudioMessage {
    pub source: String,
}

impl AudioMessage {
    pub fn new(source: String) -> Self {
        Self { source }
    }
}

fn client_audio_system(
    audio: Res<Audio>,
    audio_sources: Res<Assets<AudioSource>>,
    mut net: ResMut<NetworkResource>,
) {
    for (_handle, connection) in net.connections.iter_mut() {
        let channels = connection.channels().unwrap();

        while let Some(audio_message) = channels.recv::<AudioMessage>() {
            audio.play(audio_sources.get_handle(audio_message.source.as_str()));
        }
    }
}

pub struct NetworkAudioPlugin(bool);

impl NetworkAudioPlugin {
    pub fn server() -> Self {
        Self(true)
    }

    pub fn client() -> Self {
        Self(false)
    }
}

impl Plugin for NetworkAudioPlugin {
    fn build(&self, app_builder: &mut AppBuilder) {
        if !self.0 {
            app_builder.add_system(client_audio_system.system());
        }
    }
}
