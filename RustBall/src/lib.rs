//! src/lib.rs
//  -------------------------------------------------------------------
//  Punto de arranque de la aplicación: carga módulos, crea la App y
//  registra todos los sistemas / recursos.
//  -------------------------------------------------------------------

// ──────────────── MÓDULOS DEL JUEGO ────────────────────────────────
pub mod components;
pub mod resources;
pub mod events;
pub mod systems;          // contenedor re-export
pub mod setup;
pub mod formation;
pub mod formation_selection;
pub mod game_over;
mod powerup;
pub mod zone;
mod snapshot;

// ──────────────── USE GENÉRICOS ────────────────────────────────────
use bevy::asset::AssetMetaCheck;
use once_cell::sync::OnceCell;
use powerup::*;

use crate::resources::WsInbox;

// 🔐 Caja estática para pasar mensajes JS → Bevy (sólo WASM)
#[cfg(target_arch = "wasm32")]
static WS_INBOX: OnceCell<Mutex<WsInbox>> = OnceCell::new();

// WASM bootstrap
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::wasm_bindgen;

// ─────────────── USE ESPECÍFICOS DEL JUEGO ─────────────────────────
use crate::components::{PlayerDisk, PowerUpLabel};
use crate::events::{FormationChosenEvent, TurnFinishedEvent};
use crate::zone::apply_zone_effects;


#[cfg(target_arch = "wasm32")]
use std::sync::Mutex;

#[cfg(target_arch = "wasm32")]
pub static INIT: once_cell::sync::OnceCell<Mutex<Option<crate::resources::BackendInfo>>> = once_cell::sync::OnceCell::new();

#[cfg(target_arch = "wasm32")]
pub static FORMACIONES: once_cell::sync::OnceCell<Mutex<Option<Vec<crate::snapshot::FormacionData>>>> = once_cell::sync::OnceCell::new();

// ──────────────────────────── WASM entry ───────────────────────────
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    main_internal();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    main_internal();
}

// Función JS para enviar texto por WebSocket
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = sendOverWS)]
    fn send_over_ws(msg: &str);
}

// Entrada JS → cola Bevy
#[wasm_bindgen]
pub fn receive_ws_message(msg: String) {
    web_sys::console::log_1(&format!("📥 Mensaje desde JS: {msg}").into());

    #[cfg(target_arch = "wasm32")]
    if let Some(lock) = WS_INBOX.get() {
        if let Ok(mut inbox) = lock.lock() {
            inbox.0.push(msg);
        }
    }
}

// 🎮 Juego real
pub fn main_internal() {
    use bevy::prelude::*;
    use bevy::audio::{PlaybackSettings, GlobalVolume};
    use bevy_rapier2d::prelude::*;
    use crate::resources::*;
    use crate::events::GoalEvent;
    use crate::setup::{setup, cleanup_cameras};
    use crate::systems::*;
    use crate::formation_selection::{handle_formation_click, cleanup_formation_ui};
    use crate::setup::ui::cleanup_power_bar;
    use crate::game_over::{show_game_over_screen, cleanup_game_over_ui};
    use crate::zone::{
        update_zone_lifetime,
        update_active_effect_text,
        hide_effect_text_if_none,
    };

    #[derive(Resource)]
    pub struct TeamSelectionMusic(pub Handle<AudioSource>);
    #[derive(Resource)]
    pub struct InGameMusic(pub Handle<AudioSource>);
    #[derive(Resource)]
    pub struct GoalSound(pub Handle<AudioSource>);
    #[derive(Resource)]
    pub struct GameOverMusic(pub Handle<AudioSource>);

    #[derive(Resource)]
    pub struct BackgroundImage(pub Handle<Image>);
    use crate::resources::GameOverBackground;

    #[derive(Component)]
    struct FormationMusicTag;
    #[derive(Component)]
    struct InGameMusicTag;
    #[derive(Component)]
    struct GameOverMusicTag;

    #[derive(Component)]
    struct BackgroundTag;
    #[derive(Component)]
    struct GameOverBackgroundTag;

    fn load_team_selection_music(mut commands: Commands, asset_server: Res<AssetServer>) {
        let menu = asset_server.load("audio/uefa-champions-league-theme.mp3");
        let game = asset_server.load("audio/love_me_again.ogg");
        let goal = asset_server.load("audio/mariano-closs-ahi-estaaaaa-gooool.ogg");
        commands.insert_resource(TeamSelectionMusic(menu));
        commands.insert_resource(InGameMusic(game));
        commands.insert_resource(GoalSound(goal));
    }

    fn load_game_over_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
        let music = asset_server.load("audio/Avicii_-_The_Nights_CeeNaija.com_.ogg");
        let image = asset_server.load("Camp-Nou.png");
        commands.insert_resource(GameOverMusic(music));
        commands.insert_resource(GameOverBackground(image));
    }

    fn play_selection_music(music: Res<TeamSelectionMusic>, mut commands: Commands) {
        commands.spawn((AudioBundle {
            source: music.0.clone(),
            settings: PlaybackSettings::LOOP,
        }, FormationMusicTag));
    }

    fn stop_selection_music(mut commands: Commands, query: Query<Entity, With<FormationMusicTag>>) {
        for entity in &query {
            commands.entity(entity).despawn_recursive();
        }
    }

    fn play_ingame_music(music: Res<InGameMusic>, mut commands: Commands) {
        commands.spawn((AudioBundle {
            source: music.0.clone(),
            settings: PlaybackSettings::LOOP,
        }, InGameMusicTag));
    }

    fn stop_ingame_music(mut commands: Commands, query: Query<Entity, With<InGameMusicTag>>) {
        for entity in &query {
            commands.entity(entity).despawn_recursive();
        }
    }

    fn play_goal_sound(audio: Res<GoalSound>, mut commands: Commands) {
        commands.spawn(AudioBundle {
            source: audio.0.clone(),
            settings: PlaybackSettings::ONCE,
        });
    }

    fn play_game_over_music(music: Res<GameOverMusic>, mut commands: Commands) {
        commands.spawn((AudioBundle {
            source: music.0.clone(),
            settings: PlaybackSettings::LOOP,
        }, GameOverMusicTag));
    }

    fn stop_game_over_music(mut commands: Commands, query: Query<Entity, With<GameOverMusicTag>>) {
        for entity in &query {
            commands.entity(entity).despawn_recursive();
        }
    }

    fn load_background_image(mut commands: Commands, asset_server: Res<AssetServer>) {
        let image = asset_server.load("championsfondo3.png");
        commands.insert_resource(BackgroundImage(image));
    }

    fn spawn_selection_background(mut commands: Commands, background: Res<BackgroundImage>) {
        commands.spawn((NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            background_color: Color::WHITE.into(),
            ..default()
        }, BackgroundTag)).with_children(|parent| {
            parent.spawn(ImageBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                image: UiImage::new(background.0.clone()),
                ..default()
            });
        });
    }

    fn despawn_selection_background(mut commands: Commands, query: Query<Entity, With<BackgroundTag>>) {
        for entity in &query {
            commands.entity(entity).despawn_recursive();
        }
    }

    fn spawn_game_over_background(mut commands: Commands, bg: Res<GameOverBackground>) {
        commands.spawn((NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            background_color: Color::BLACK.into(),
            ..default()
        }, GameOverBackgroundTag)).with_children(|parent| {
            parent.spawn(ImageBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                image: UiImage::new(bg.0.clone()),
                transform: Transform::from_xyz(0.0, 0.0, -1.0),
                ..default()
            });
        });
    }

    fn cleanup_game_over_background(mut commands: Commands, query: Query<Entity, With<GameOverBackgroundTag>>) {
        for entity in &query {
            commands.entity(entity).despawn_recursive();
        }
    }

    fn show_formation_ui_system(mut commands: Commands, asset_server: Res<AssetServer>) {
        formation_selection::show_formation_ui(&mut commands, &asset_server);
    }

    fn cleanup_cameras_on_enter(mut commands: Commands, query: Query<Entity, With<Camera>>) {
        cleanup_cameras(&mut commands, query);
    }

    pub fn debug_powerup_system(
        labels: Query<Entity, With<PowerUpLabel>>,
        disks: Query<(Entity, &Children), With<PlayerDisk>>,
    ) {
        println!("PowerUpLabels en el juego: {}", labels.iter().count());

        for (disk_entity, children) in &disks {
            let has_label = children.iter().any(|&child| labels.contains(child));
            println!("Disco {:?} tiene label: {}", disk_entity, has_label);
        }
    }

    /*pub fn check_both_formations_chosen(
        formations: Res<PlayerFormations>,
        mut next_state: ResMut<NextState<AppState>>,
    ) {
        if formations.player1.is_some() && formations.player2.is_some() {
            next_state.set(AppState::InGame);
        }
    }*/

    #[cfg(target_arch = "wasm32")]
    {
        INIT.set(Mutex::new(None)).ok();
        FORMACIONES.set(Mutex::new(None)).ok();
    }

    // --------------------------------------------------------------------
    //  IMPORTA LOS SystemSet QUE ACABAS DE CREAR
    // --------------------------------------------------------------------
    use crate::snapshot::ApplySnapshotSet;   // systems que aplican snapshot
    use crate::systems::CheckTurnEndSet;     // systems que cierran un turno
    use crate::systems::{maybe_send_pending_turn, PendingTurn}; // ✅ ya importados

    // --------------------------------------------------------------------
    //  CREA Y CONFIGURA LA APLICACIÓN
    // --------------------------------------------------------------------
    let mut app = App::new();

    // ───────── Recursos básicos ─────────────────────────────────────────
    app.insert_resource(AssetMetaCheck::Never)
        .insert_resource(GlobalVolume::new(1.0))
        .insert_resource(ClearColor(Color::BLACK))
        .add_state::<AppState>()
        .insert_resource(TurnState::default())
        .insert_resource(Scores::default())
        .insert_resource(PlayerFormations { player1: None, player2: None })
        .insert_resource(PowerUpControl::default())
        .insert_resource(EventControl::default())
        .insert_resource(snapshot::MyTurn::default())
        .insert_resource(PendingTurn::default())
        .insert_resource(CurrentPlayerId::default())
        .insert_resource(poll_turn::PollState::default())
        .insert_resource(UltimoTurnoAplicado::default());

    // ───────── Sección ESPECÍFICA WASM ──────────────────────────────────
    #[cfg(target_arch = "wasm32")]
    {
        use std::sync::Mutex;

        use crate::{
            resources::{LatestSnapshot, SnapshotPollTimer, WsInbox},
            snapshot::{
                poll_snapshot_when_forming,        // ⏱  polling mientras falta la 2ª formación
                fetch_snapshot_on_ws_message,      // 📡 reacciona a “start” / “turno_finalizado”
            },
        };

        // Inbox global para que JS empuje aquí los mensajes del WebSocket
        WS_INBOX.set(Mutex::new(WsInbox::default())).ok();

        app
            /* ───── Recursos compartidos ───────────────────────────── */
            .insert_resource(WsInbox::default())
            .insert_resource(LatestSnapshot::default())
            .insert_resource(SnapshotPollTimer::default())

            /* ───── 1) Polling de snapshot SOLO en FormationSelection ─ */
            .add_systems(
                Update,
                poll_snapshot_when_forming
                    .run_if(in_state(AppState::FormationSelection)),
            )
            // Al salir de ese estado ya no necesitamos más el timer
            .add_systems(
                OnExit(AppState::FormationSelection),
                |mut c: Commands| {
                    c.remove_resource::<SnapshotPollTimer>();
                },
            )

            /* ───── 2) WebSocket activo en los tres estados clave ───── */
            .add_systems(
                Update,
                (
                    process_ws_messages,         // lee y coloca en WsInbox
                    fetch_snapshot_on_ws_message // actúa cuando llega “start” o “turno_finalizado”
                )
                    .run_if(
                        in_state(AppState::FormationSelection)
                            .or_else(in_state(AppState::FormationChange))
                            .or_else(in_state(AppState::InGame)),
                    ),
            );
    }

    // ───────── Plugins (Bevy + Rapier) ──────────────────────────────────
    app.add_plugins((
        DefaultPlugins.set(AssetPlugin {
            watch_for_changes_override: Some(false),
            ..default()
        }),
        RapierPhysicsPlugin::<NoUserData>::default(),
    ))
        .insert_resource(RapierConfiguration { gravity: Vec2::ZERO, ..default() });

    // ───────── Eventos globales ─────────────────────────────────────────
    app.add_event::<GoalEvent>()
        .add_event::<FormationChosenEvent>()
        .add_event::<TurnFinishedEvent>();

    // ───────── STARTUP ──────────────────────────────────────────────────
    app.add_systems(Startup, insert_backend_info)
        .add_systems(
            Startup,
            (
                setup_fonts,
                load_team_selection_music,
                load_background_image,
                load_game_over_assets,
            ),
        );

    // 🔁 Carga los datos del backend (snapshot, uid, pid) desde el OnceCell
    #[cfg(target_arch = "wasm32")]
    use crate::systems::load_backend_info_if_available;

    #[cfg(target_arch = "wasm32")]
    app.add_systems(OnEnter(AppState::FormationSelection), load_backend_info_if_available);


    // ───────── STATE: FormationSelection ────────────────────────────────
    app.add_systems(
        OnEnter(AppState::FormationSelection),
        (show_formation_ui_system, spawn_selection_background, play_selection_music),
    )
        .add_systems(
            OnExit(AppState::FormationSelection),
            (despawn_selection_background, stop_selection_music),
        )
        .add_systems(
            Update,
            (
                handle_formation_click,
                animate_selection_buttons,
                send_formacion_to_backend,
            )
                .run_if(
                    in_state(AppState::FormationSelection)
                        .or_else(in_state(AppState::FormationChange)),
                ),
        );

    // ───────── STATE: FormationChange ───────────────────────────────────
    app.add_systems(
        OnEnter(AppState::FormationChange),
        (show_formation_ui_system, reset_for_formation, cleanup_power_bar),
    );

    // ───────── STATE: InGame – enter / exit ─────────────────────────────
    app.add_systems(
        OnEnter(AppState::InGame),
        (
            cleanup_formation_ui,
            cleanup_cameras_on_enter,
            play_ingame_music,
            setup,
            attach_powerup_label_once,
        ),
    )
        .add_systems(OnExit(AppState::InGame), stop_ingame_music);

    // ───────── SISTEMAS PRINCIPALES (snapshot y turno) ──────────────────
    app.add_systems(
        Update,
        snapshot::snapshot_apply_system
            .run_if(
                in_state(AppState::FormationSelection)
                    .or_else(in_state(AppState::FormationChange))
                    .or_else(in_state(AppState::InGame)),
            )
            .in_set(ApplySnapshotSet),                    // ← NUEVO SystemSet
    )
        .add_systems(
            Update,
            maybe_send_pending_turn // ✅ sin "systems::"
                .after(snapshot::snapshot_apply_system)
                .run_if(in_state(AppState::InGame)),
        )

        .add_systems(
            Update,
            poll_turn::poll_turn_tick_system.run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            (
                auto_select_first_disk,
                cycle_disk_selection,
                aim_with_keyboard,
                charge_shot_power,
                debug_powerup_system,
                send_goal_to_backend,
            )
                .run_if(|t: Res<snapshot::MyTurn>| t.0)
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            (
                fire_selected_disk,
                apply_zone_effects,
                check_turn_end.in_set(CheckTurnEndSet),       // ← NUEVO SystemSet
                send_turn_to_backend.after(CheckTurnEndSet),
                detect_goal,
                handle_goal,
            )
                .run_if(in_state(AppState::InGame)),
        );

    // ───────── HUD / UI / Visual ────────────────────────────────────────
    app.add_systems(
        PostUpdate,
        (
            update_turn_text,
            update_score_text,
            animate_selected_disk,
            spawn_power_up_if_needed,
            detect_powerup_collision,
        )
            .after(ApplySnapshotSet)
            .run_if(in_state(AppState::InGame)),
    )
        .add_systems(
            Update,
            (
                trigger_random_event_system,
                update_zone_lifetime,
                update_active_effect_text,
                hide_effect_text_if_none,
            )
                .run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            PostUpdate,
            (
                update_powerup_labels,
                remove_powerup_label,
                draw_aim_direction_gizmo,
                update_power_bar,
            )
                .run_if(in_state(AppState::InGame)),
        );

    // ───────── STATE: GoalScored ────────────────────────────────────────
    app.add_systems(
        OnEnter(AppState::GoalScored),
        (setup_goal_timer, play_goal_sound),
    )
        .add_systems(
            Update,
            (goal_banner_fadeout, wait_and_change_state).run_if(in_state(AppState::GoalScored)),
        );

    // ───────── STATE: GameOver ──────────────────────────────────────────
    app.add_systems(
        OnEnter(AppState::GameOver),
        (
            despawn_game_entities,
            spawn_game_over_background,
            play_game_over_music,
            show_game_over_screen,
        ),
    )
        .add_systems(
            OnExit(AppState::GameOver),
            (cleanup_game_over_background, stop_game_over_music, cleanup_game_over_ui),
        );

    // ───────── ARRANCA EL JUEGO ─────────────────────────────────────────
    app.run();
}
