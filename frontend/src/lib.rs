#![allow(clippy::non_ascii_literal)]

#[macro_use]
extern crate seed;
extern crate hifitime;
extern crate instant;
extern crate wee_alloc;

use std::convert::TryFrom;

use seed::{prelude::*, *};
// use wasm_bindgen::prelude::*;

use futures::Future;
use seed::{fetch, Method, Request};
use serde::Deserialize;

use hifitime::Epoch;
use humantime;
use instant::Instant;

// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn link(rel: String, href: String) -> Node<Msg> {
    custom![
        Tag::Custom("link".into()),
        attrs! { At::Href => href, At::Rel => rel }
    ]
}

static HOME: &'static str = "/mindustry-mods";
static RGUC: &'static str = "https://raw.githubusercontent.com";

#[derive(Deserialize, Debug, Clone)]
struct Mod {
    name: String,
    stars: u32,
    date_tt: f64,
    desc: String,
    link: String,
    repo: String,
    wiki: Option<String>,
    delta_ago: String,
    icon_raw: Option<String>,
}

impl Mod {
    /// Link to the mod's archive.
    fn archive_link(&self) -> Node<Msg> {
        let l = format!("https://github.com/{}/archive/master.zip", self.repo);
        a![attrs! { At::Href => l }, "zip"]
    }

    /// Endpoint link as a string.
    fn endpoint_href(&self) -> String {
        let path = self.repo.replace("/", "--");
        format!("/{}/m/{}.html", HOME, path).into()
    }

    /// Endpoint link to the locally rendered README.md
    fn endpoint_link(&self) -> Node<Msg> {
        a![attrs! { At::Href => self.endpoint_href() }, self.name]
    }

    /// Link to the mods repository.
    fn repo_link(&self) -> Node<Msg> {
        a![attrs! { At::Href => self.link }, "repository"]
    }

    /// Link to the optional wiki.
    fn wiki_link(&self) -> Node<Msg> {
        match &self.wiki {
            Some(link) => a![attrs! { At::Href => link }, "wiki"],
            None => a![style! { "display" => "none" }],
        }
    }

    fn last_commit(&self) -> Node<Msg> {
        span![self.delta_ago, " ago"]
    }

    /// Returns unicode stars.
    fn fmt_stars(&self) -> String {
        match usize::try_from(self.stars) {
            Err(_) => "err".into(),
            Ok(0) => "☆".into(),
            Ok(x) => "★ ".repeat(x),
        }
    }

    fn icon(&self) -> Node<Msg> {
        match self.icon_raw.as_ref().map(String::as_str) {
            Some("") | None => a![svg![
                attrs! {
                    At::Width => "50",
                    At::Height => "50",
                },
                rect![attrs! {
                    At::Width => "50",
                    At::Height => "50",
                    At::Stroke => "#f0f0f0"
                }]
            ]],
            Some(p) => {
                let i = format!("{}/{}/master/{}", RGUC, self.repo, p);
                a![
                    attrs! { At::Href => self.endpoint_href() },
                    img![attrs! { At::Src => i }, style! { "width" => "50px" }]
                ]
            }
        }
    }

    fn description(&self) -> Node<Msg> {
        p![attrs! { At::Class => "description" }, self.desc]
    }

    // fn version_render(&self) -> Node<Msg> {
    //     if let "" = &self.version {
    //         return span![style! { "display" => "none" }];
    //     }
    // }

    /// Returns the `Node<Msg>` for the listing.
    fn listing_item(&self) -> Node<Msg> {
        div![
            attrs! { At::Class => "wrapper" },
            div![
                attrs! { At::Class => "links" },
                self.icon(),
                self.repo_link(),
                self.archive_link(),
                self.wiki_link(),
            ],
            self.description(),
        ]
    }
}

struct Model {
    count: i32,

    /// instant the app started
    dt: Instant,

    /// number of requests submitted for date updates
    data_requested: u32,

    data: Vec<Mod>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            count: 0,
            dt: Instant::now(),
            data_requested: 0,
            data: vec![],
        }
    }
}

#[derive(Debug, Clone)]
enum Msg {
    FetchData(fetch::ResponseDataResult<Vec<Mod>>),
}

fn fetch_data() -> impl Future<Item = Msg, Error = Msg> {
    Request::new("data/modmeta.1.0.json")
        .method(Method::Get)
        .fetch_json_data(Msg::FetchData)
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::FetchData(data) => model.data = data.unwrap(),
    }
}

mod date {
    use js_sys::Date;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    pub fn from_tt(x: f64) -> SystemTime {
        let secs = (x as u64) / 1_000;
        let nanos = ((x as u32) % 1_000) * 1_000_000;
        UNIX_EPOCH + Duration::new(secs, nanos)
    }

    pub fn now() -> SystemTime {
        let x = Date::now();
        from_tt(x)
    }
}

fn view(model: &Model) -> impl View<Msg> {
    let now = date::now();
    let before = date::from_tt(457.3892);

    div![
        attrs! { At::Class => "app" },
        header![h1!["Mindustry Mods"]],
        link("StyleSheet".into(), "css/listing.css".into()),
        div![
            attrs! { At::Class => "listing-container" },
            model.data.iter().map(|r| r.listing_item())
        ]
    ]
}

fn after_mount(_: Url, orders: &mut impl Orders<Msg>) -> AfterMount<Model> {
    orders.perform_cmd(fetch_data());
    AfterMount::default()
}

#[wasm_bindgen(start)]
pub fn render() {
    seed::App::builder(update, view)
        .after_mount(after_mount)
        .build_and_start();
}
