use simplelog::{ConfigBuilder, LevelFilter, TermLogger, TerminalMode};

pub fn start_logger() {
    let config = ConfigBuilder::new()
        .set_location_level(LevelFilter::Error)
        .add_filter_ignore_str("gfx_backend_vulkan")
        .build();
    TermLogger::init(LevelFilter::Info, config, TerminalMode::Mixed).unwrap();
}
