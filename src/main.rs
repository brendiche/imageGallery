// use dioxus::prelude::*;

// use components::Navbar;
// use views::{Blog, Home};

// mod components;
// mod views;

// #[derive(Debug, Clone, Routable, PartialEq)]
// #[rustfmt::skip]
// enum Route {
//     #[layout(Navbar)]
//     #[route("/")]
//     Home {},
//     #[route("/blog/:id")]
//     Blog { id: i32 },
// }

// const FAVICON: Asset = asset!("/assets/favicon.ico");
// const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
// const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

// fn main() {
//     dioxus::launch(App);
// }

// #[component]
// fn App() -> Element {
//     // Build cool things ✌️

//     rsx! {
//         // Global app resources
//         document::Link { rel: "icon", href: FAVICON }
//         document::Link { rel: "stylesheet", href: MAIN_CSS }
//         document::Link { rel: "stylesheet", href: TAILWIND_CSS }

//         Router::<Route> {}
//     }
// }


use dioxus::prelude::*;
use image_gallery::{get_stories,get_story, CommentData, StoryItem, StoryPageData};


fn main() {
    launch(App);
}

pub fn App() -> Element {
    use_context_provider(|| Signal::new(PreviewState::Unset));
    rsx! {
        div { display: "flex", flex_direction: "row", width: "100%",
            div { width: "50%", Stories {} }
            div { width: "50%", Preview {} }
        }
    }
}

fn Stories() -> Element {
    // Fetch the top 10 stories on Hackernews
    let stories = use_resource(move || get_stories(10));

    // check if the future is resolved
    match &*stories.read_unchecked() {
        Some(Ok(list)) => {
            // if it is, render the stories
            rsx! {
                div {
                    // iterate over the stories with a for loop
                    for story in list {
                        // render every story with the StoryListing component
                        StoryListing { story: story.clone() }
                    }
                }
            }
        }
        Some(Err(err)) => {
            // if there was an error, render the error
            rsx! {"An error occurred while fetching stories {err}"}
        }
        None => {
            // if the future is not resolved yet, render a loading message
            rsx! {"Loading items"}
        }
    }
}


fn Preview() -> Element {
    let preview_state = consume_context::<Signal<PreviewState>>();
    match preview_state() {
        PreviewState::Unset => rsx! {"Hover over a story to preview it here"},
        PreviewState::Loading => rsx! {"Loading..."},
        PreviewState::Loaded(story) => {
            rsx! {
                div { padding: "0.5rem",
                    div { font_size: "1.5rem", a { href: story.item.url, "{story.item.title}" } }
                    div { dangerous_inner_html: story.item.text }
                    for comment in &story.comments {
                        Comment { comment: comment.clone() }
                    }
                }
            }
        }
    }
}

#[component]
fn Comment(comment: CommentData) -> Element {
    rsx! {
        div { padding: "0.5rem",
            div { color: "gray", "by {comment.by}" }
            div { dangerous_inner_html: "{comment.text}" }
            for kid in &comment.sub_comments {
                Comment { comment: kid.clone() }
            }
        }
    }
}

#[component]
fn StoryListing(story: ReadOnlySignal<StoryItem>) -> Element {
    let mut preview_state = consume_context::<Signal<PreviewState>>();
    let StoryItem {
        title,
        url,
        by,
        score,
        time,
        kids,
        id,
        ..
    } = story();
    let full_story = use_signal(|| None);

    let url = url.as_deref().unwrap_or_default();
    let hostname = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.");
    let score = format!("{score} {}", if score == 1 { " point" } else { " points" });
    let comments = format!(
        "{} {}",
        kids.len(),
        if kids.len() == 1 {
            " comment"
        } else {
            " comments"
        }
    );
    let time = time.format("%D %l:%M %p");

    rsx! {
        div { padding: "0.5rem", position: "relative",onmouseenter: move |_event| {resolve_story(full_story, preview_state, id)
        },
            div { font_size: "1.5rem",
                a { href: url, "{title}" }
                a {
                    color: "gray",
                    href: "https://news.ycombinator.com/from?site={hostname}",
                    text_decoration: "none",
                    " ({hostname})"
                }
            }
            div { display: "flex", flex_direction: "row", color: "gray",
                div { "{score}" }
                div { padding_left: "0.5rem", "by {by}" }
                div { padding_left: "0.5rem", "{time}" }
                div { padding_left: "0.5rem", "{comments}" }
            }
        }
    }
}

#[derive(Clone, Debug)]
enum PreviewState {
    Unset,
    Loading,
    Loaded(StoryPageData),
}


async fn resolve_story(
    mut full_story: Signal<Option<StoryPageData>>,
    mut preview_state: Signal<PreviewState>,
    story_id: i64,
) {
    if let Some(cached) = full_story.as_ref() {
        *preview_state.write() = PreviewState::Loaded(cached.clone());
        return;
    }

    *preview_state.write() = PreviewState::Loading;
    if let Ok(story) = get_story(story_id).await {
        *preview_state.write() = PreviewState::Loaded(story.clone());
        *full_story.write() = Some(story);
    }
}
