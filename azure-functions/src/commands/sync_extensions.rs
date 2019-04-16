use crate::registry::Registry;
use clap::{App, Arg, ArgMatches, SubCommand};
use std::env::current_dir;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use xml::{writer::XmlEvent, EmitterConfig};

pub struct SyncExtensions {
    pub script_root: PathBuf,
    pub verbose: bool,
}

impl SyncExtensions {
    pub fn create_subcommand<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("sync-extensions")
            .about("Synchronizes the Azure Function binding extensions used by the worker.")
            .arg(
                Arg::with_name("script_root")
                    .long("script-root")
                    .value_name("SCRIPT_ROOT")
                    .help("The script root to synchronize the binding extensions for.")
                    .required(true),
            )
            .arg(
                Arg::with_name("verbose")
                    .long("verbose")
                    .short("v")
                    .help("Use verbose output."),
            )
    }

    pub fn execute(&self, registry: Registry<'static>) -> Result<(), String> {
        if !registry.has_binding_extensions() {
            if self.verbose {
                println!("No binding extensions are needed.");
            }
            return Ok(());
        }

        let temp_dir = TempDir::new().expect("failed to create temporary directory");
        let extensions_project_path = temp_dir.path().join("extensions.csproj");
        let metadata_project_path = temp_dir.path().join("metadata.csproj");

        Self::write_extensions_project_file(&extensions_project_path, &registry);
        Self::write_generator_project_file(&metadata_project_path);

        if self.verbose {
            println!("Restoring extension assemblies...");
        }

        let status = Command::new("dotnet")
            .args(&[
                "publish",
                "-c",
                "Release",
                "-o",
                self.script_root.join("bin").to_str().unwrap(),
                "/v:q",
                "/nologo",
                extensions_project_path.to_str().unwrap(),
            ])
            .current_dir(temp_dir.path())
            .status()
            .unwrap_or_else(|e| panic!("failed to spawn dotnet: {}", e));

        if !status.success() {
            panic!(
                "failed to restore extensions: dotnet returned non-zero exit code {}.",
                status.code().unwrap()
            );
        }

        if self.verbose {
            println!("Generating extension metadata...");
        }

        let status = Command::new("dotnet")
            .args(&[
                "msbuild",
                "/t:_GenerateFunctionsExtensionsMetadataPostPublish",
                "/v:q",
                "/nologo",
                "/restore",
                "-p:Configuration=Release",
                &format!("-p:PublishDir={}/", self.script_root.to_str().unwrap()),
                metadata_project_path.to_str().unwrap(),
            ])
            .current_dir(temp_dir.path())
            .status()
            .unwrap_or_else(|e| panic!("failed to spawn dotnet: {}", e));

        if !status.success() {
            panic!(
                "failed to generate extension metadata: dotnet returned non-zero exit code {}.",
                status.code().unwrap()
            );
        }

        Ok(())
    }

    fn write_property(writer: &mut xml::EventWriter<&mut fs::File>, name: &str, value: &str) {
        writer.write(XmlEvent::start_element(name)).unwrap();
        writer.write(XmlEvent::characters(value)).unwrap();
        writer.write(XmlEvent::end_element()).unwrap();
    }

    fn write_extensions_project_file(path: &Path, registry: &Registry<'static>) {
        let mut project_file =
            fs::File::create(path).expect("Failed to create extensions project file.");

        let mut writer = EmitterConfig::new()
            .perform_indent(true)
            .create_writer(&mut project_file);

        writer
            .write(XmlEvent::start_element("Project").attr("Sdk", "Microsoft.NET.Sdk"))
            .unwrap();

        writer
            .write(XmlEvent::start_element("PropertyGroup"))
            .unwrap();

        Self::write_property(&mut writer, "TargetFramework", "netstandard2.0");
        Self::write_property(&mut writer, "CopyBuildOutputToPublishDirectory", "false");
        Self::write_property(&mut writer, "CopyOutputSymbolsToPublishDirectory", "false");
        Self::write_property(&mut writer, "GenerateDependencyFile", "false");

        writer.write(XmlEvent::end_element()).unwrap();

        writer.write(XmlEvent::start_element("ItemGroup")).unwrap();

        for extension in registry.iter_binding_extensions() {
            writer
                .write(
                    XmlEvent::start_element("PackageReference")
                        .attr("Include", extension.0)
                        .attr("Version", extension.1),
                )
                .unwrap();
            writer.write(XmlEvent::end_element()).unwrap();
        }

        writer.write(XmlEvent::end_element()).unwrap();
        writer.write(XmlEvent::end_element()).unwrap();
    }

    fn write_generator_project_file(path: &Path) {
        let mut project_file =
            fs::File::create(path).expect("Failed to create generator project file.");

        let mut writer = EmitterConfig::new()
            .perform_indent(true)
            .create_writer(&mut project_file);

        writer
            .write(XmlEvent::start_element("Project").attr("Sdk", "Microsoft.NET.Sdk"))
            .unwrap();

        writer
            .write(XmlEvent::start_element("PropertyGroup"))
            .unwrap();

        Self::write_property(&mut writer, "TargetFramework", "netstandard2.0");

        writer.write(XmlEvent::end_element()).unwrap();

        writer.write(XmlEvent::start_element("ItemGroup")).unwrap();

        writer
            .write(
                XmlEvent::start_element("PackageReference")
                    .attr(
                        "Include",
                        "Microsoft.Azure.WebJobs.Script.ExtensionsMetadataGenerator",
                    )
                    .attr("Version", "1.0.1")
                    .attr("PrivateAssets", "all"),
            )
            .unwrap();

        writer.write(XmlEvent::end_element()).unwrap();
        writer.write(XmlEvent::end_element()).unwrap();
        writer.write(XmlEvent::end_element()).unwrap();
    }
}

impl<'a> From<&ArgMatches<'a>> for SyncExtensions {
    fn from(args: &ArgMatches<'a>) -> Self {
        SyncExtensions {
            script_root: current_dir()
                .expect("failed to get current directory")
                .join(
                    args.value_of("script_root")
                        .expect("A script root is required."),
                ),
            verbose: args.is_present("verbose"),
        }
    }
}
