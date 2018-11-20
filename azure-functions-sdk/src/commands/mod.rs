macro_rules! templates {
    ( $dir:expr => [$( $x:expr ),*] ) => {
        {
            let mut templates = ::handlebars::Handlebars::new();
            $(
                templates.register_template_string($x, include_str!(concat!("../templates/", $dir, "/", $x, ".hbs")))
                    .expect(concat!("failed to register ", $x, " template."));
            )*

            templates
        }
    };
}

mod build;
mod new_app;

pub use self::build::*;
pub use self::new_app::*;
