mod sql;
mod app;
mod plug;

fn main()
{
    app::main();
    if let app::AppType::EDITOR(_) = &*app::APP_TYPE {
        plug::inter_data::DATA.with(
            |data| plug::funcs::start(std::rc::Rc::clone(data)),
        );
    }
}
