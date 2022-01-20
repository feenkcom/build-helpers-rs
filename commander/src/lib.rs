use std::process::{Command, Output};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::Result;

pub struct CommandToExecute {
    name: String,
    command: Command,
    verbose: bool
}

impl CommandToExecute {
    pub fn new(command: Command) -> Self {
        Self {
            name: "".to_string(),
            command,
            verbose: false
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    pub fn execute(&mut self) -> Result<Output> {
        self.command.output()
    }
}

pub struct CommandsToExecute {
    commands: Vec<CommandToExecute>
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
            let pb = if command.is_verbose() {
                println!("[{}/{}] Executing {:?}", index, total, command.name());
                None
            } else {
                let pb = ProgressBar::new_spinner();

                pb.enable_steady_tick(120);
                pb.set_style(
                    ProgressStyle::default_spinner()
                        .tick_strings(&[
                            "🌑 ", "🌒 ", "🌓 ", "🌔 ", "🌕 ", "🌖 ", "🌗 ", "🌘 ", "✅ ",
                        ])
                        .template("{prefix:.bold.dim} {spinner:.blue} {wide_msg}"),
                );
                pb.set_message(format!("Executing {:?}", command.name()));
                pb.set_prefix(format!("[{}/{}]", index, total));

                Some(pb)
            };

            command.execute()?;

            if let Some(ref pb) = pb {
                pb.finish_with_message(format!("Finished {:?}", command.name()));
            } else {
                println!("Finished {:?}", command.name());
            }
        }

        Ok(())
    }
}