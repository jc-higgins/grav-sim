use grav_sim::run;

fn main() {
    tracing_subscriber::fmt::init();
    run().unwrap();
}
