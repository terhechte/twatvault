#![allow(non_snake_case)]
use std::{collections::HashMap, path::PathBuf, rc::Rc};

use dioxus::desktop::tao::window::WindowBuilder;
use dioxus::events::*;
use dioxus::fermi::{use_atom_state, AtomState};
use dioxus::prelude::*;
use egg_mode::user::TwitterUser;
use tokio::sync::mpsc::channel;
use tracing::warn;

use crate::config::{Config, RequestData};
use crate::crawler::DownloadInstruction;
use crate::storage::{Data, Storage, TweetId, UrlString, UserId};
use crate::types::Message;
use egg_mode::tweet::Tweet;

use super::types::LoadingState;
use super::types::StorageWrapper;

#[inline_props]
pub fn LoadingComponent(
    cx: Scope,
    config: Config,
    loading_state: UseState<LoadingState>,
) -> Element {
    let message_state = use_state(&cx, || Message::Initial);

    let crawl = move |config: Config| {
        let (sender, mut receiver) = channel(256);
        cx.spawn(async move {
            let path = Config::archive_path();
            if let Err(e) = crate::crawler::crawl_new_storage(config, &path, sender).await {
                warn!("Error {e:?}");
            }
        });
        use_future(&cx, (), move |_| {
            let message_state = message_state.clone();
            let loading_state = loading_state.clone();
            async move {
                while let Some(msg) = receiver.recv().await {
                    let finished = match msg {
                        Message::Finished(o) => {
                            // FIXME: Assign owned storage
                            loading_state.set(LoadingState::Loaded(StorageWrapper::new(o)));
                            true
                        }
                        other => {
                            message_state.set(other);
                            false
                        }
                    };
                    if finished {
                        break;
                    }
                }
            }
        });
    };

    let ui = match message_state.get() {
        Message::Error(e) => rsx!(div {
                 "Error: {e:?}"
            }
        ),
        Message::Finished(_) => rsx!(div {
            // This should never appear here
        }),
        Message::Loading(msg) => rsx!(div {
            h3 {
                "Importing"
            }
            "{msg}"
        }),
        Message::Initial => rsx!(div {
            button {
                r#type: "button",
                class: "btn btn-secondary",
                onclick: move |_| crawl(config.clone()),
                "Begin Crawling"
            }
        }),
    };
    cx.render(ui)
}