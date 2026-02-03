use colored::Colorize;
use log::Level;

pub fn configure_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            let level = format!("[{}]", record.level());
            let colored_level = match record.level() {
                Level::Error => level.bright_red(),
                Level::Warn => level.yellow(),
                Level::Info => level.blue(),
                Level::Debug => level.dimmed(),
                Level::Trace => level.dimmed(),
            };

            out.finish(format_args!("{} {}", colored_level, message))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()?;

    Ok(())
}
