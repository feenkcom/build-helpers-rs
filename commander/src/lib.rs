use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use new_string_template::template::Template;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::io::{Error, ErrorKind, Result};
use std::process::{Command, Output};

pub struct CommandToExecute {
    name: String,
    command: Command,
    verbose: bool,
    log_prefix: String,
    log_message: String,
}

impl CommandToExecute {
    pub fn new(command: Command) -> Self {
        Self {
            name: "".to_string(),
            command,
            verbose: false,
            log_prefix: "[{ index }/{ total }]".to_string(),
            log_message: "Executing { name }".to_string(),
        }
    }

    pub fn new_with(command_name: impl AsRef<OsStr>, builder: impl Fn(&mut Command)) -> Self {
        let mut command = Command::new(command_name);
        builder(&mut command);
        Self::new(command)
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn with_log_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.log_prefix = prefix.into();
        self
    }

    pub fn without_log_prefix(mut self) -> Self {
        self.log_prefix = "".to_string();
        self
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    pub fn execute(&mut self) -> Result<Output> {
        let output = self.command.output()?;
        println!("executed command {:?}", &self.command);
        println!("output {:?}", &output);
        if !output.status.success() {
            let stderr = String::from_utf8(output.stderr).unwrap();

            return Err(Error::new(
                ErrorKind::Other,
                format!(
                    "Command {} didn't finish successfully (exit code = {:?}): {:?}\n{}",
                    self.name(),
                    output.status.code(),
                    &self.command,
                    stderr
                ),
            ));
        }
        Ok(output)
    }
}

pub struct CommandsToExecute {
    commands: Vec<CommandToExecute>,
}

impl CommandsToExecute {
    pub fn new() -> Self {
        Self { commands: vec![] }
    }

    pub fn add(mut self, command: CommandToExecute) -> Self {
        self.commands.push(command);
        self
    }

    pub fn execute(&mut self) -> Result<()> {
        let mut index = 0 as usize;
        let total = self.commands.len();

        for command in &mut self.commands {
            index += 1;

            let mut data = HashMap::<String, String>::new();
            data.insert("index".to_string(), index.to_string());
            data.insert("total".to_string(), total.to_string());
            data.insert("name".to_string(), command.name().to_string());
            let prefix = Template::new(command.log_prefix.as_str())
                .render_string(&data)
                .unwrap();
            let message = Template::new(command.log_message.as_str())
                .render_string(&data)
                .unwrap();

            let pb = if command.is_verbose() {
                None
            } else {
                let pb = ProgressBar::with_draw_target(!0, ProgressDrawTarget::stderr());
                if pb.is_hidden() {
                    None
                } else {
                    pb.enable_steady_tick(120);
                    pb.set_style(
                        ProgressStyle::default_spinner()
                            .tick_strings(&[
                                "🌑 ", "🌒 ", "🌓 ", "🌔 ", "🌕 ", "🌖 ", "🌗 ", "🌘 ", "✅ ",
                            ])
                            .template("{prefix:.bold.dim} {spinner:.blue} {wide_msg}"),
                    );
                    pb.set_message(format!("{}", &message));
                    pb.set_prefix(format!("{}", &prefix));
                    Some(pb)
                }
            };

            if pb.is_none() {
                print!("{} {}...", prefix, command.name());
            }

            command.execute()?;

            if let Some(ref pb) = pb {
                pb.finish_with_message(format!("Finished {:?}", command.name()));
            } else {
                println!(" Done");
            }
        }

        Ok(())
    }
}
