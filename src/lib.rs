#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

#[get("/<id>")]
fn get_puzzle(id: String) -> String {
    format!("{}", id)
}

pub fn start_server() {
    rocket::ignite().mount("/", routes![get_puzzle]).launch();
}
