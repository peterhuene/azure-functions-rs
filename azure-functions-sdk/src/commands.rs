macro_rules! template {
    ( $templates:expr, $dir:expr, $file:expr ) => {
        $templates
            .register_template_string(
                $file,
                include_str!(concat!("templates/", $dir, "/", $file, ".hbs")),
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

        template!(templates, "new-app", "local.settings.json");
        template!(templates, "new-app", "host.json");
        template!(templates, "new-app", "Dockerfile");
        template!(templates, "new-app", "dockerignore");
        template!(templates, "new-app", "functions_mod.rs");
        template!(templates, "new-app", "launch.json");
        template!(templates, "new-app", "main.rs");
        template!(templates, "new-app", "tasks.json");

        template!(templates, "new", "http.rs");
        template!(templates, "new", "blob.rs");
        template!(templates, "new", "queue.rs");
        template!(templates, "new", "timer.rs");
        template!(templates, "new", "eventgrid.rs");
        template!(templates, "new", "eventhub.rs");
        template!(templates, "new", "cosmosdb.rs");
        template!(templates, "new", "servicebus.rs");

        templates
    };
}

mod new;
mod new_app;
mod run;

pub use self::new::*;
pub use self::new_app::*;
pub use self::run::*;
