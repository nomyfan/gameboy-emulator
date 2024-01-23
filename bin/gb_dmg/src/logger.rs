use std::io::Write;

pub fn init() {
    let mut builder = env_logger::builder();
    builder.format(|buf, record| {
        writeln!(
            buf,
            "{}:{} {} [{}] - {}",
            record.file().unwrap_or("unknown"),
            record.line().unwrap_or(0),
            chrono::offset::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f"),
            record.level(),
            record.args()
        )
    });

    if let Ok(path) = std::env::var("RUST_LOG_FILE") {
        let target = Box::new(std::fs::File::create(path).expect("Can't create file"));
        builder.target(env_logger::Target::Pipe(target));
    }

    builder.init();
}
