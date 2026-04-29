use components::dots_menu::{DotsMenu, MenuAction};
use components::folder_picker::FolderPickerModal;
use config::AppConfig;
use dioxus::prelude::*;
use reader::{Library, PlaylistStore};

#[component]
pub fn LocalPlaylists(
    mut playlist_store: Signal<PlaylistStore>,
    library: Signal<Library>,
    config: Signal<AppConfig>,
    mut selected_playlist_id: Signal<Option<String>>,
    on_select_folder: EventHandler<String>,
) -> Element {
    let mut active_menu = use_signal(|| Option::<String>::None);
    let mut open_folder_id = use_signal(|| Option::<String>::None);
    let mut move_target_id = use_signal(|| Option::<String>::None);

    let store = playlist_store.read();
    let lib = library.read();

    let folders = store.folders.clone();
    let all_playlists = store.playlists.clone();

    let root_playlists: Vec<_> = all_playlists
        .iter()
        .filter(|p| !folders.iter().any(|f| f.playlist_ids.contains(&p.id)))
        .cloned()
        .collect();

    let open_folder = open_folder_id
        .read()
        .as_ref()
        .and_then(|id| folders.iter().find(|f| f.id == *id).cloned());

    let folder_playlists: Vec<_> = if let Some(ref folder) = open_folder {
        folder
            .playlist_ids
            .iter()
            .filter_map(|pid| all_playlists.iter().find(|p| p.id == *pid).cloned())
            .collect()
    } else {
        vec![]
    };

    let delete_text = i18n::t("delete_playlist").to_string();
    let move_text = i18n::t("move_to_folder").to_string();
    let remove_folder_text = i18n::t("remove_from_folder").to_string();
    let delete_folder_text = i18n::t("delete_playlist").to_string(); // reuse

    let playlist_actions = vec![
        MenuAction::new(move_text.as_str(), "fa-solid fa-folder-open"),
        MenuAction::new(delete_text.as_str(), "fa-solid fa-trash").destructive(),
    ];
    let folder_playlist_actions = vec![
        MenuAction::new(move_text.as_str(), "fa-solid fa-folder-open"),
        MenuAction::new(remove_folder_text.as_str(), "fa-solid fa-folder-minus"),
        MenuAction::new(delete_text.as_str(), "fa-solid fa-trash").destructive(),
    ];
    let folder_actions = vec![
        MenuAction::new(delete_folder_text.as_str(), "fa-solid fa-trash").destructive(),
    ];

    drop(store);
    drop(lib);

    let lib = library.read();

    let cover_for = |pid: &str| -> Option<String> {
        let store = playlist_store.read();
        let playlist = store.playlists.iter().find(|p| p.id == pid)?;
        let first_path = playlist.tracks.first()?;
        let track = lib.tracks.iter().find(|t| t.path == *first_path)?;
        let album = lib.albums.iter().find(|a| a.id == track.album_id)?;
        utils::format_artwork_url(album.cover_path.as_ref())
    };

    rsx! {
        div {
            if let Some(target_id) = move_target_id.read().clone() {
                FolderPickerModal {
                    playlist_store,
                    playlist_id: target_id,
                    on_close: move |_| move_target_id.set(None),
                }
            }

            if let Some(ref folder) = open_folder {
                div {
                    div { class: "flex items-center gap-3 mb-8",
                        button {
                            class: "flex items-center gap-2 text-slate-400 hover:text-white transition-colors",
                            onclick: move |_| open_folder_id.set(None),
                            i { class: "fa-solid fa-arrow-left" }
                            "{i18n::t(\"back_to_playlists\")}"
                        }
                        span { class: "text-white/30", "/" }
                        span { class: "text-white font-semibold", "{folder.name}" }
                    }

                    if folder_playlists.is_empty() {
                        div { class: "flex flex-col items-center justify-center h-48 text-slate-500",
                            i { class: "fa-regular fa-folder-open text-4xl mb-4 opacity-50" }
                            p { "{i18n::t(\"no_playlists_yet\")}" }
                        }
                    } else {
                        div { class: "grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6",
                            {folder_playlists.iter().map(|playlist| {
                                let cover_url = cover_for(&playlist.id);
                                let pid = playlist.id.clone();
                                let pid_click = playlist.id.clone();
                                let pid_menu = playlist.id.clone();
                                let pid_action = playlist.id.clone();
                                let fid_remove = folder.id.clone();
                                let is_menu_open = active_menu.read().as_deref() == Some(playlist.id.as_str());

                                rsx! {
                                    div {
                                        key: "{pid}",
                                        class: "bg-white/5 border border-white/5 rounded-2xl p-6 hover:bg-white/10 transition-all cursor-pointer group relative",
                                        onclick: move |_| selected_playlist_id.set(Some(pid_click.clone())),
                                        div {
                                            class: "mb-4 w-full aspect-square rounded-xl flex items-center justify-center overflow-hidden transition-all bg-white/5",
                                            if let Some(url) = cover_url {
                                                img { src: "{url}", class: "w-full h-full object-cover group-hover:scale-105 transition-transform duration-500" }
                                            } else {
                                                div {
                                                    class: "w-full h-full flex items-center justify-center",
                                                    style: "background: color-mix(in srgb, var(--color-indigo-500), transparent 80%); color: var(--color-indigo-400)",
                                                    i { class: "fa-solid fa-list-ul text-2xl" }
                                                }
                                            }
                                        }
                                        div { class: "flex items-start justify-between gap-2",
                                            div { class: "min-w-0 flex-1",
                                                h3 { class: "text-xl font-bold text-white mb-1 truncate", "{playlist.name}" }
                                                {
                                                    let count = playlist.tracks.len();
                                                    let track_text = i18n::t_with("playlist_track_count", &[("count", count.to_string())]);
                                                    rsx! { p { class: "text-sm text-slate-400", "{track_text}" } }
                                                }
                                            }
                                            div {
                                                onclick: move |evt| evt.stop_propagation(),
                                                DotsMenu {
                                                    actions: folder_playlist_actions.clone(),
                                                    is_open: is_menu_open,
                                                    on_open: move |_| active_menu.set(Some(pid_menu.clone())),
                                                    on_close: move |_| active_menu.set(None),
                                                    button_class: "opacity-0 group-hover:opacity-100 focus:opacity-100".to_string(),
                                                    anchor: "right".to_string(),
                                                    on_action: move |idx: usize| {
                                                        match idx {
                                                            0 => { move_target_id.set(Some(pid_action.clone())); active_menu.set(None); }
                                                            1 => {
                                                                let mut store = playlist_store.write();
                                                                if let Some(f) = store.folders.iter_mut().find(|f| f.id == fid_remove) {
                                                                    f.playlist_ids.retain(|id| id != &pid_action);
                                                                }
                                                                active_menu.set(None);
                                                            }
                                                            _ => {
                                                                playlist_store.write().playlists.retain(|p| p.id != pid_action);
                                                                for f in &mut playlist_store.write().folders {
                                                                    f.playlist_ids.retain(|id| id != &pid_action);
                                                                }
                                                                active_menu.set(None);
                                                            }
                                                        }
                                                    },
                                                }
                                            }
                                        }
                                    }
                                }
                            })}
                        }
                    }
                }
            } else {
                div {
                    if folders.is_empty() && root_playlists.is_empty() {
                        div { class: "flex flex-col items-center justify-center h-64 text-slate-500",
                            i { class: "fa-regular fa-folder-open text-4xl mb-4 opacity-50" }
                            p { "{i18n::t(\"no_playlists_yet\")}" }
                        }
                    } else {
                        if !folders.is_empty() {
                            div { class: "grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6 mb-8",
                                {folders.iter().map(|folder| {
                                    let fid = folder.id.clone();
                                    let fid_open = folder.id.clone();
                                    let fid_menu = folder.id.clone();
                                    let fid_del = folder.id.clone();
                                    let fname = folder.name.clone();
                                    let count = folder.playlist_ids.len();
                                    let is_menu_open = active_menu.read().as_deref() == Some(folder.id.as_str());

                                    let cover_url = folder.playlist_ids.first()
                                        .and_then(|pid| cover_for(pid));

                                    rsx! {
                                        div {
                                            key: "{fid}",
                                            class: "bg-white/5 border border-white/5 rounded-2xl p-6 hover:bg-white/10 transition-all cursor-pointer group relative",
                                            onclick: move |_| open_folder_id.set(Some(fid_open.clone())),
                                            div {
                                                class: "mb-4 w-full aspect-square rounded-xl flex items-center justify-center overflow-hidden transition-all bg-white/5",
                                                if let Some(url) = cover_url {
                                                    img { src: "{url}", class: "w-full h-full object-cover group-hover:scale-105 transition-transform duration-500" }
                                                } else {
                                                    div {
                                                        class: "w-full h-full flex items-center justify-center",
                                                        style: "background: color-mix(in srgb, var(--color-amber-500), transparent 80%); color: var(--color-amber-400)",
                                                        i { class: "fa-solid fa-folder text-2xl" }
                                                    }
                                                }
                                            }
                                            div { class: "flex items-start justify-between gap-2",
                                                div { class: "min-w-0 flex-1",
                                                    h3 { class: "text-xl font-bold text-white mb-1 truncate", "{fname}" }
                                                    p { class: "text-sm text-slate-400", "{count} playlists" }
                                                }
                                                div {
                                                    onclick: move |evt| evt.stop_propagation(),
                                                    DotsMenu {
                                                        actions: folder_actions.clone(),
                                                        is_open: is_menu_open,
                                                        on_open: move |_| active_menu.set(Some(fid_menu.clone())),
                                                        on_close: move |_| active_menu.set(None),
                                                        button_class: "opacity-0 group-hover:opacity-100 focus:opacity-100".to_string(),
                                                        anchor: "right".to_string(),
                                                        on_action: move |_| {
                                                            let mut store = playlist_store.write();
                                                            store.folders.retain(|f| f.id != fid_del);
                                                            active_menu.set(None);
                                                        },
                                                    }
                                                }
                                            }
                                        }
                                    }
                                })}
                            }
                        }

                        if !root_playlists.is_empty() {
                            if !folders.is_empty() {
                                h2 { class: "text-sm font-semibold text-white/40 uppercase tracking-widest mb-4", "{i18n::t(\"playlists\")}" }
                            }
                            div { class: "grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6",
                                {root_playlists.iter().map(|playlist| {
                                    let cover_url = cover_for(&playlist.id);
                                    let pid = playlist.id.clone();
                                    let pid_click = playlist.id.clone();
                                    let pid_menu = playlist.id.clone();
                                    let pid_action = playlist.id.clone();
                                    let is_menu_open = active_menu.read().as_deref() == Some(playlist.id.as_str());

                                    rsx! {
                                        div {
                                            key: "{pid}",
                                            class: "bg-white/5 border border-white/5 rounded-2xl p-6 hover:bg-white/10 transition-all cursor-pointer group relative",
                                            onclick: move |_| selected_playlist_id.set(Some(pid_click.clone())),
                                            div {
                                                class: "mb-4 w-full aspect-square rounded-xl flex items-center justify-center overflow-hidden transition-all bg-white/5",
                                                if let Some(url) = cover_url {
                                                    img { src: "{url}", class: "w-full h-full object-cover group-hover:scale-105 transition-transform duration-500" }
                                                } else {
                                                    div {
                                                        class: "w-full h-full flex items-center justify-center",
                                                        style: "background: color-mix(in srgb, var(--color-indigo-500), transparent 80%); color: var(--color-indigo-400)",
                                                        i { class: "fa-solid fa-list-ul text-2xl" }
                                                    }
                                                }
                                            }
                                            div { class: "flex items-start justify-between gap-2",
                                                div { class: "min-w-0 flex-1",
                                                    h3 { class: "text-xl font-bold text-white mb-1 truncate", "{playlist.name}" }
                                                    {
                                                        let count = playlist.tracks.len();
                                                        let track_text = i18n::t_with("playlist_track_count", &[("count", count.to_string())]);
                                                        rsx! { p { class: "text-sm text-slate-400", "{track_text}" } }
                                                    }
                                                }
                                                div {
                                                    onclick: move |evt| evt.stop_propagation(),
                                                    DotsMenu {
                                                        actions: playlist_actions.clone(),
                                                        is_open: is_menu_open,
                                                        on_open: move |_| active_menu.set(Some(pid_menu.clone())),
                                                        on_close: move |_| active_menu.set(None),
                                                        button_class: "opacity-0 group-hover:opacity-100 focus:opacity-100".to_string(),
                                                        anchor: "right".to_string(),
                                                        on_action: move |idx: usize| {
                                                            match idx {
                                                                0 => { move_target_id.set(Some(pid_action.clone())); active_menu.set(None); }
                                                                _ => {
                                                                    playlist_store.write().playlists.retain(|p| p.id != pid_action);
                                                                    active_menu.set(None);
                                                                }
                                                            }
                                                        },
                                                    }
                                                }
                                            }
                                        }
                                    }
                                })}
                            }
                        }
                    }
                }
            }
        }
    }
}
