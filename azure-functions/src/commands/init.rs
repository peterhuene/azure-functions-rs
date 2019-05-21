use crate::{codegen::Function, commands::SyncExtensions, registry::Registry};
use clap::{App, Arg, ArgMatches, SubCommand};
use serde::Serialize;
use serde_json::{json, to_string_pretty, Serializer};
use std::env::{self, current_dir, current_exe};
use std::fs;
use std::path::{Path, PathBuf};

pub struct Init<'a> {
    pub script_root: PathBuf,
    pub local_settings: Option<&'a str>,
    pub host_settings: Option<&'a str>,
    pub sync_extensions: bool,
    pub no_debug_info: bool,
    pub verbose: bool,
}

impl<'a> Init<'a> {
    pub fn create_subcommand<'b>() -> App<'a, 'b> {
        SubCommand::with_name("init")
                .about("Initializes the Azure Functions application script root.")
                .arg(
                    Arg::with_name("script_root")
                        .long("script-root")
                        .value_name("SCRIPT_ROOT")
                        .help("The script root directory to initialize the application in.")
                        .required(true),
                )
                .arg(
                    Arg::with_name("local_settings")
                        .long("local-settings")
                        .value_name("LOCAL_SETTINGS_FILE")
                        .help("The path to the local settings file to use. Defaults to the `local.settings.json` file in the directory containing `Cargo.toml`, if present.")
                        .validator(|v| {
                            if Path::new(&v).is_file() {
                                Ok(())
                            } else {
                                Err(format!("local settings file '{}' does not exist", v))
                            }
                        })
                )
                .arg(
                    Arg::with_name("host_settings")
                        .long("host-settings")
                        .value_name("HOST_SETTINGS_FILE")
                        .help("The path to the host settings file to use. Defaults to the `host.json` file in the directory containing `Cargo.toml`, if present.")
                        .validator(|v| {
                            if Path::new(&v).is_file() {
                                Ok(())
                            } else {
                                Err(format!("host settings file '{}' does not exist", v))
                            }
                        })
                )
                .arg(
                    Arg::with_name("sync_extensions")
                        .long("sync-extensions")
                        .short("s")
                        .help("Synchronize the Azure Function binding extensions.")
                )
                .arg(
                    Arg::with_name("no_debug_info")
                        .long("--no-debug-info")
                        .help("Do not copy debug information for the worker executable.")
                )
                .arg(
                    Arg::with_name("verbose")
                        .long("verbose")
                        .short("v")
                        .help("Use verbose output.")
                )
    }

    pub fn execute(
        &self,
        registry: Registry<'static>,
        extensions: &[(&str, &str)],
    ) -> Result<(), String> {
        self.create_script_root();

        match self.get_local_path(&self.host_settings, "host.json") {
            Some(path) => self.copy_host_settings_file(&path),
            None => self.create_host_settings_file(),
        };

        match self.get_local_path(&self.local_settings, "local.settings.json") {
            Some(path) => self.copy_local_settings_file(&path),
            None => self.create_local_settings_file(),
        };

        let current_exe =
            current_exe().expect("failed to determine the path to the current executable");

        let worker_dir = self.create_worker_dir();
        let worker_exe = worker_dir.join(current_exe.file_name().unwrap());

        self.copy_worker_executable(&current_exe, &worker_exe);

        if !self.no_debug_info {
            self.copy_worker_debug_info(&current_exe, &worker_exe);
        }

        self.create_worker_config_file(&worker_dir, &worker_exe);

        self.delete_existing_function_directories();

        for (name, info) in registry.iter() {
            let function_dir = self.create_function_directory(name);

            let source_file = Init::get_source_file_path(
                Path::new(
                    info.manifest_dir
                        .as_ref()
                        .expect("Functions should have a manifest directory.")
                        .as_ref(),
                ),
                Path::new(
                    info.file
                        .as_ref()
                        .expect("Functions should have a file.")
                        .as_ref(),
                ),
            );

            self.copy_source_file(&function_dir, &source_file, name);
            self.create_function_config_file(&function_dir, info);
        }

        if self.sync_extensions {
            let command = SyncExtensions {
                script_root: self.script_root.clone(),
                verbose: self.verbose,
            };
            return command.execute(registry, extensions);
        }

        Ok(())
    }

    fn get_local_path(&self, path: &Option<&str>, filename: &str) -> Option<PathBuf> {
        if let Some(path) = path {
            return Some(path.into());
        }

        env::var("CARGO_MANIFEST_DIR")
            .map(|dir| {
                let path = PathBuf::from(dir).join(filename);
                if path.is_file() {
                    Some(path)
                } else {
                    None
                }
            })
            .unwrap_or(None)
    }

    fn create_script_root(&self) {
        if self.script_root.exists() {
            if self.verbose {
                println!(
                    "Using existing Azure Functions application at '{}'.",
                    self.script_root.display()
                );
            }
        } else {
            if self.verbose {
                println!(
                    "Creating Azure Functions application at '{}'.",
                    self.script_root.display()
                );
            }

            fs::create_dir_all(&self.script_root).unwrap_or_else(|e| {
                panic!(
                    "failed to create Azure Functions application directory '{}': {}",
                    self.script_root.display(),
                    e
                )
            });
        }
    }

    fn copy_host_settings_file(&self, local_host_file: &Path) {
        let output_host_file = self.script_root.join("host.json");

        if self.verbose {
            println!(
                "Copying host settings file '{}' to '{}'.",
                local_host_file.display(),
                output_host_file.display()
            );
        }

        fs::copy(local_host_file, output_host_file).unwrap_or_else(|e| {
            panic!(
                "failed to copy host settings file '{}': {}",
                local_host_file.display(),
                e
            )
        });
    }

    fn create_host_settings_file(&self) {
        let settings = self.script_root.join("host.json");

        if self.verbose {
            println!(
                "Creating default host settings file '{}'.",
                settings.display()
            );
        }

        fs::write(
            &settings,
            to_string_pretty(&json!(
            {
                "version": "2.0",
                "logging": {
                    "logLevel": {
                        "default": "Warning"
                    }
                }
            }))
            .unwrap(),
        )
        .unwrap_or_else(|e| panic!("failed to create '{}': {}", settings.display(), e));
    }

    fn copy_local_settings_file(&self, local_settings_file: &Path) {
        let output_settings = self.script_root.join("local.settings.json");

        if self.verbose {
            println!(
                "Copying local settings file '{}' to '{}'.",
                local_settings_file.display(),
                output_settings.display()
            );
        }

        fs::copy(local_settings_file, output_settings).unwrap_or_else(|e| {
            panic!(
                "failed to copy local settings file '{}': {}",
                local_settings_file.display(),
                e
            )
        });
    }

    fn create_local_settings_file(&self) {
        let settings = self.script_root.join("local.settings.json");

        if self.verbose {
            println!(
                "Creating default local settings file '{}'.",
                settings.display()
            );
        }

        fs::write(
            &settings,
            to_string_pretty(&json!(
            {
                "IsEncrypted": false,
                "Values": {
                    "FUNCTIONS_WORKER_RUNTIME": "Rust",
                    "languageWorkers:workersDirectory": "workers"
                },
                "ConnectionStrings": {
                }
            }))
            .unwrap(),
        )
        .unwrap_or_else(|e| panic!("failed to create '{}': {}", settings.display(), e));
    }

    fn create_worker_dir(&self) -> PathBuf {
        let worker_dir = self.script_root.join("workers").join("rust");

        if worker_dir.exists() {
            fs::remove_dir_all(&worker_dir).unwrap_or_else(|e| {
                panic!(
                    "failed to delete Rust worker directory '{}': {}",
                    worker_dir.display(),
                    e
                )
            });
        }

        if self.verbose {
            println!("Creating worker directory '{}'.", worker_dir.display());
        }

        fs::create_dir_all(&worker_dir).unwrap_or_else(|e| {
            panic!(
                "failed to create directory for worker executable '{}': {}",
                worker_dir.display(),
                e
            )
        });

        worker_dir
    }

    fn copy_worker_executable(&self, current_exe: &Path, worker_exe: &Path) {
        if self.verbose {
            println!(
                "Copying current worker executable to '{}'.",
                worker_exe.display()
            );
        }

        fs::copy(current_exe, worker_exe).expect("Failed to copy worker executable");
    }

    #[cfg(target_os = "windows")]
    fn copy_worker_debug_info(&self, current_exe: &Path, worker_exe: &Path) {
        let current_pdb = current_exe.with_extension("pdb");
        if !current_pdb.is_file() {
            return;
        }

        let worker_pdb = worker_exe.with_extension("pdb");

        if self.verbose {
            println!(
                "Copying worker debug information to '{}'.",
                worker_pdb.display()
            );
        }

        fs::copy(current_pdb, worker_pdb).expect("Failed to copy worker debug information");
    }

    #[cfg(target_os = "macos")]
    fn copy_worker_debug_info(&self, current_exe: &Path, worker_exe: &Path) {
        use fs_extra::dir;

        let current_dsym = current_exe.with_extension("dSYM");
        if !current_dsym.exists() {
            return;
        }

        let worker_dsym = worker_exe.with_extension("dSYM");

        if self.verbose {
            println!(
                "Copying worker debug information to '{}'.",
                worker_dsym.display()
            );
        }

        let mut options = dir::CopyOptions::new();
        options.copy_inside = true;

        dir::copy(current_dsym, worker_dsym, &options)
            .expect("Failed to copy worker debug information");
    }

    #[cfg(target_os = "linux")]
    fn copy_worker_debug_info(&self, _: &Path, _: &Path) {
        // No-op
    }

    fn create_worker_config_file(&self, worker_dir: &Path, worker_exe: &Path) {
        let config = worker_dir.join("worker.config.json");
        if config.exists() {
            return;
        }

        if self.verbose {
            println!("Creating worker config file '{}'.", config.display());
        }

        fs::write(
            &config,
            to_string_pretty(&json!(
            {
                "description":{
                    "language": "Rust",
                    "extensions": [".rs"],
                    "defaultExecutablePath": worker_exe.to_str().unwrap(),
                    "arguments": ["run"]
                }
            }))
            .unwrap(),
        )
        .unwrap_or_else(|e| panic!("failed to create '{}': {}", config.display(), e));
    }

    fn delete_existing_function_directories(&self) {
        for entry in fs::read_dir(&self.script_root).expect("failed to read script root directory")
        {
            let path = self
                .script_root
                .join(entry.expect("failed to read script root entry").path());
            if !path.is_dir() || !Init::has_rust_files(&path) {
                continue;
            }

            if self.verbose {
                println!(
                    "Deleting existing Rust function directory '{}'.",
                    path.display()
                );
            }

            fs::remove_dir_all(&path).unwrap_or_else(|e| {
                panic!(
                    "failed to delete function directory '{}': {}",
                    path.display(),
                    e
                )
            });
        }
    }

    fn create_function_directory(&self, function_name: &str) -> PathBuf {
        let function_dir = self.script_root.join(function_name);

        if self.verbose {
            println!("Creating function directory '{}'.", function_dir.display());
        }

        fs::create_dir(&function_dir).unwrap_or_else(|e| {
            panic!(
                "failed to create function directory '{}': {}",
                function_dir.display(),
                e
            )
        });

        function_dir
    }

    fn copy_source_file(&self, function_dir: &Path, source_file: &Path, function_name: &str) {
        let destination_file = function_dir.join(
            source_file
                .file_name()
                .expect("expected the source file to have a file name"),
        );

        if source_file.is_file() {
            if self.verbose {
                println!(
                    "Copying source file '{}' to '{}' for Azure Function '{}'.",
                    source_file.display(),
                    destination_file.display(),
                    function_name
                );
            }

            fs::copy(&source_file, destination_file).unwrap_or_else(|e| {
                panic!(
                    "failed to copy source file '{}': {}",
                    source_file.display(),
                    e
                )
            });
        } else {
            if self.verbose {
                println!(
                    "Creating empty source file '{}' for Azure Function '{}'.",
                    destination_file.display(),
                    function_name
                );
            }

            fs::write(
                    &destination_file,
                    "// This file is intentionally empty.\n\
                     // The original source file was not available when the Functions Application was initialized.\n"
                ).unwrap_or_else(|e| panic!("failed to create '{}': {}", destination_file.display(), e));
        }
    }

    fn create_function_config_file(&self, function_dir: &Path, info: &'static Function) {
        let function_json = function_dir.join("function.json");

        if self.verbose {
            println!(
                "Creating function configuration file '{}' for Azure Function '{}'.",
                function_json.display(),
                info.name
            );
        }

        let mut output = fs::File::create(&function_json)
            .unwrap_or_else(|e| panic!("failed to create '{}': {}", function_json.display(), e));

        info.serialize(&mut Serializer::pretty(&mut output))
            .unwrap_or_else(|e| {
                panic!(
                    "failed to serialize metadata for function '{}': {}",
                    info.name, e
                )
            });
    }

    // This is a workaround to the issue that `file!` expands to be workspace-relative
    // and cargo does not have an environment variable for the workspace directory.
    // Thus, this walks up the manifest directory until it hits "src" in the file's path.
    // This function is sensitive to cargo and rustc changes.
    fn get_source_file_path(manifest_dir: &Path, file: &Path) -> PathBuf {
        let mut manifest_dir = Path::new(manifest_dir);
        for component in file.components() {
            if component.as_os_str() == "src" {
                break;
            }
            manifest_dir = manifest_dir
                .parent()
                .expect("expected another parent for the manifest directory");
        }

        manifest_dir.join(file)
    }

    fn has_rust_files(directory: &Path) -> bool {
        fs::read_dir(directory)
            .unwrap_or_else(|e| panic!("failed to read directory '{}': {}", directory.display(), e))
            .any(|p| match p {
                Ok(p) => {
                    let p = p.path();
                    p.is_file() && p.extension().map(|x| x == "rs").unwrap_or(false)
                }
                _ => false,
            })
    }
}

impl<'a> From<&'a ArgMatches<'a>> for Init<'a> {
    fn from(args: &'a ArgMatches<'a>) -> Self {
        Init {
            script_root: current_dir()
                .expect("failed to get current directory")
                .join(
                    args.value_of("script_root")
                        .expect("A script root is required."),
                ),
            local_settings: args.value_of("local_settings"),
            host_settings: args.value_of("host_settings"),
            sync_extensions: args.is_present("sync_extensions"),
            no_debug_info: args.is_present("no_debug_info"),
            verbose: args.is_present("verbose"),
        }
    }
}
