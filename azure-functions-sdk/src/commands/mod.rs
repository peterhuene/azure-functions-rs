macro_rules! template {
    ( $templates:expr, $dir:expr, $file:expr ) => {
        $templates
            .register_template_string(
                $file,
                include_str!(concat!("../templates/", $dir, "/", $file, ".hbs")),
            )
            .expect(concat!(
                "failed to register ",
                $dir,
                "/",
                $file,
                " template."
            ));
    };
}

lazy_static::lazy_static! {
    static ref TEMPLATES: handlebars::Handlebars = {
        let mut templates = handlebars::Handlebars::new();

        template!(templates, "new-app", "appsettings.json");
        template!(templates, "new-app", "Dockerfile");
        template!(templates, "new-app", "dockerignore");
        template!(templates, "new-app", "functions_mod.rs");
        template!(templates, "new-app", "main.rs");

        template!(templates, "new", "http.rs");
        template!(templates, "new", "blob.rs");
        template!(templates, "new", "queue.rs");
        template!(templates, "new", "timer.rs");

        templates
    };
}

mod deploy;
mod new;
mod new_app;
mod run;

pub use self::deploy::*;
pub use self::new::*;
pub use self::new_app::*;
pub use self::run::*;

fn set_colorization(color: Option<&str>) {
    colored::control::set_override(match color {
        Some("auto") | None => ::atty::is(atty::Stream::Stdout),
        Some("always") => true,
        Some("never") => false,
        _ => panic!("unsupported color option"),
    });
}
