mod server_functions;

use dioxus::prelude::*;
use server_functions::{create_todo, delete_todo, list_todos, toggle_todo};

fn main() {
    #[cfg(all(not(feature = "server"), any(feature = "desktop", feature = "mobile")))]
    dioxus::fullstack::set_server_url(option_env!("SERVER_URL").unwrap_or("http://localhost:8080"));

    #[cfg(feature = "server")]
    dioxus::serve(|| async move { Ok(dioxus::server::router(App)) });

    #[cfg(not(feature = "server"))]
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let initial = use_server_future(list_todos)?;
    let initial_result = initial().unwrap_or_else(|| Ok(Vec::new()));
    let initial_error = initial_result.as_ref().err().map(ToString::to_string);
    let initial_todos = initial_result.unwrap_or_default();

    let mut todos = use_signal(move || initial_todos);
    let mut title = use_signal(String::new);
    let mut error = use_signal(move || initial_error);

    rsx! {
        document::Title { "Todo Template" }
        document::Meta {
            name: "description",
            content: "A server-rendered Dioxus todo application template",
        }
        document::Stylesheet { href: asset!("/assets/main.css") }

        main { class: "shell",
            section { class: "card",
                p { class: "eyebrow", "DIOXUS SSR TEMPLATE" }
                h1 { "A tiny todo list" }
                p { class: "intro",
                    "Server-rendered on the web, interactive after hydration, and reusable on mobile."
                }

                form {
                    class: "new-todo",
                    onsubmit: move |event| {
                        event.prevent_default();
                        let next_title = title().trim().to_owned();
                        if next_title.is_empty() {
                            return;
                        }
                        spawn(async move {
                            match create_todo(next_title).await {
                                Ok(todo) => {
                                    todos.write().push(todo);
                                    title.set(String::new());
                                    error.set(None);
                                }
                                Err(cause) => error.set(Some(cause.to_string())),
                            }
                        });
                    },
                    input {
                        value: title,
                        placeholder: "What needs doing?",
                        aria_label: "Todo title",
                        oninput: move |event| title.set(event.value()),
                    }
                    button { r#type: "submit", "Add todo" }
                }

                if let Some(message) = error() {
                    p { class: "error", "{message}" }
                }

                ul { class: "todos",
                    for todo in todos() {
                        li { key: "{todo.id}", class: if todo.completed { "done" } else { "" },
                            button {
                                class: "toggle",
                                aria_label: "Toggle todo",
                                onclick: move |_| {
                                    let id = todo.id;
                                    spawn(async move {
                                        match toggle_todo(id).await {
                                            Ok(updated) => {
                                                if let Some(item) = todos.write().iter_mut().find(|item| item.id == id) {
                                                    *item = updated;
                                                }
                                            }
                                            Err(cause) => error.set(Some(cause.to_string())),
                                        }
                                    });
                                },
                                span { class: "check", "✓" }
                                span { "{todo.title}" }
                            }
                            button {
                                class: "delete",
                                aria_label: "Delete todo",
                                onclick: move |_| {
                                    let id = todo.id;
                                    spawn(async move {
                                        match delete_todo(id).await {
                                            Ok(()) => todos.write().retain(|item| item.id != id),
                                            Err(cause) => error.set(Some(cause.to_string())),
                                        }
                                    });
                                },
                                "Remove"
                            }
                        }
                    }
                }

                if todos().is_empty() {
                    p { class: "empty", "Nothing here yet. Add the first item." }
                }
            }
        }
    }
}
