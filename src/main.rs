extern crate soundio;

fn main() {
    println!("Soundio version: {}", soundio::version_string());

    let (major, minor, patch) = soundio::version();

    println!("Major version: {}, minor version: {}, patch version: {}", major, minor, patch);

    let backend_list = [
        soundio::Backend::Jack,
        soundio::Backend::PulseAudio,
        soundio::Backend::Alsa,
        soundio::Backend::CoreAudio,
        soundio::Backend::Wasapi,
        soundio::Backend::Dummy,
    ];

    for &backend in backend_list.iter() {
        println!("Backend {} available? {}", backend, soundio::have_backend(backend));
    } 

    println!("InitAudioBackend error: {}", soundio::Error::InitAudioBackend);
}