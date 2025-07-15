use crate::config::{Config, LogTarget};
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::{mem, process};
use syslog::{BasicLogger, Facility, Formatter3164};

fn init_syslog() -> anyhow::Result<()> {
    let formatter = Formatter3164 {
        facility: Facility::LOG_SYSLOG,
        hostname: None,
        process: "Rauthy NSS Proxy".into(),
        pid: process::id(),
    };

    let logger = syslog::unix(formatter)?;
    log::set_boxed_logger(Box::new(BasicLogger::new(logger)))
        .map(|()| log::set_max_level(LevelFilter::Info))?;

    Ok(())
}

fn init_file_console_log(config: &Config) -> anyhow::Result<()> {
    let target = &config.log_target;

    let mut builder = log4rs::Config::builder();
    let mut builder_root = Root::builder();

    if target == &LogTarget::Console || target == &LogTarget::ConsoleFile {
        let stdout = ConsoleAppender::builder().build();
        builder = builder.appender(Appender::builder().build("stdout", Box::new(stdout)));
        builder_root = builder_root.appender("stdout");
    }

    if target == &LogTarget::ConsoleFile || target == &LogTarget::File {
        let trigger = SizeTrigger::new(10 * 1024);
        let roller = FixedWindowRoller::builder()
            .build(&format!("{}/proxy.{{}}.log", config.data_dir), 5)?;
        let policy = CompoundPolicy::new(Box::new(trigger), Box::new(roller));

        let file = RollingFileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
            .build(format!("{}/proxy.log", config.data_dir), Box::new(policy))?;

        builder = builder.appender(Appender::builder().build("file", Box::new(file)));
        builder_root = builder_root.appender("file");
    }

    let config = builder.build(builder_root.build(LevelFilter::Info))?;

    let handle = log4rs::init_config(config)?;
    mem::forget(handle);

    Ok(())
}

pub fn init() -> anyhow::Result<()> {
    let config = Config::get();
    if config.log_target == LogTarget::Syslog {
        init_syslog()
    } else {
        init_file_console_log(config)
    }
}
