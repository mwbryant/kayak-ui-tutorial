#![allow(clippy::redundant_field_names)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use bevy::{
    prelude::Color as BevyColor, prelude::*, render::camera::ScalingMode, window::PresentMode,
};
use bevy_inspector_egui::{
    Inspectable, RegisterInspectable, WorldInspectorParams, WorldInspectorPlugin,
};

pub const CLEAR: BevyColor = BevyColor::rgb(0.3, 0.3, 0.3);
pub const HEIGHT: f32 = 900.0;
pub const RESOLUTION: f32 = 16.0 / 9.0;
use kayak_ui::{
    bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle},
    core::{
        bind, render, rsx,
        styles::{Edge, StyleProp},
        styles::{Style, Units},
        use_state, widget, Binding, Bound, Color, EventType, Index, MutableBound, OnEvent,
        OnLayout, WidgetProps,
    },
    widgets,
    widgets::Window,
};

#[derive(Component, Clone, PartialEq, Inspectable)]
pub struct Player {
    health: f32,
}

fn create_ui(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn_bundle(UICameraBundle::new())
        .insert(Name::new("UI Camera"));

    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    let context = BevyContext::new(|context| {
        let window = Style {
            background_color: StyleProp::Value(Color::new(0.125, 0.125, 0.125, 1.0)),
            border_color: StyleProp::Value(Color::new(0.0781, 0.0898, 0.101, 1.0)),
            color: StyleProp::Value(Color::new(0.5, 1.0, 1.0, 1.0)),
            ..default()
        };
        let element = Style {
            padding: StyleProp::Value(Edge::axis(Units::Percentage(0.0), Units::Percentage(10.))),
            ..Default::default()
        };

        let font = Style {
            color: StyleProp::Value(Color::new(0.5, 1.0, 0.0, 1.0)),
            ..Default::default()
        };

        let button_event = OnEvent::new(|context, event| {
            if let EventType::Click(..) = event.event_type {
                context.query_world::<Query<&mut Player>, _, _>(|mut player_query| {
                    for mut player in player_query.iter_mut() {
                        player.health += 10.0;
                    }
                });
            }
        });

        render! {
            <widgets::App>
                <widgets::Element styles={Some(element)}>
                    <widgets::Text content={"Text printing".to_string()} size={32.0} styles={Some(font)}/>
                </widgets::Element>
                <Window draggable={true} position={(0.0,0.0)} size={(250.0, 250.0)} title={"Test Window".to_string()} styles={Some(window)}>
                    <widgets::Text content={"Main Menu".to_string()} size={32.0} />
                    <widgets::Button on_event={Some(button_event)}>
                        <widgets::Text content={"Give Player Health".to_string()} size={24.0} />
                    </widgets::Button>
                </Window>
                <CustomWidget />
            </widgets::App>
        }
    });
    commands.insert_resource(context);

    let player = Player { health: 100.0 };
    let binding = bind(player.clone());

    commands.spawn().insert(player);
    commands.insert_resource(binding);
}

fn update_heatlh(player_query: Query<&Player>, binding: Res<Binding<Player>>) {
    let player = player_query.single();
    binding.set(player.clone());
}

fn print_percent(percent: Res<(f32, f32)>) {
    if percent.is_changed() {
        println!("in game: {:?}", percent);
    }
}

#[widget]
fn CustomWidget() {
    let player_binding = context.query_world::<Res<Binding<Player>>, _, _>(|player| player.clone());

    context.bind(&player_binding);

    let health = player_binding.get().health;

    let box_color = Color::WHITE;
    let button_color = Color::new(0.9, 0.1, 0.1, 1.0);

    rsx! {
        <>
            <Window draggable={true} position={(550.0, 50.0)} size={(200.0, 200.0)} title={"Window 2".to_string()}>
                <widgets::Text content={format!("Health : {}", health)} size={24.0}/>
                <SliderBox size={(10.0, 10.0)} box_color={box_color} button_color={button_color}/>
            </Window>
        </>
    }
}

#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub struct SliderBoxProps {
    size: (f32, f32),
    box_color: Color,
    button_color: Color,
}

//A 2d slider which is heavily inspired by the window widget
#[widget]
fn SliderBox(props: SliderBoxProps) {
    //Set up slider internal state
    let (is_dragging, set_is_dragging, ..) = use_state!(false);
    let (offset, set_offset, ..) = use_state!((0.0, 0.0));
    let (pos, set_pos, ..) = use_state!((0.0, 0.0));
    let (percent, set_percent, ..) = use_state!((0.0, 0.0));
    let (layout, set_layout, ..) = use_state!((100.0, 100.0));

    //Handle dragging
    let drag_handler = Some(OnEvent::new(move |ctx, event| match event.event_type {
        EventType::MouseDown(data) => {
            ctx.capture_cursor(event.current_target);
            set_is_dragging(true);
            set_offset((pos.0 - data.position.0, pos.1 - data.position.1));
        }
        EventType::MouseUp(..) => {
            ctx.release_cursor(event.current_target);
            set_is_dragging(false);
        }
        EventType::Hover(data) => {
            if is_dragging {
                set_pos((offset.0 + data.position.0, offset.1 + data.position.1));
            }
        }
        _ => {}
    }));

    //Get width and height on every layout
    let on_layout = OnLayout::new(move |_, event| {
        let layout = event.layout;
        set_layout((layout.width, layout.height));
    });

    let (width, height) = props.size;

    //Calculate max allowed percent
    //(position is set at top left corner of button so max percent is less than 100)
    let max_percent = (
        (1.0 - width / layout.0) * 100.0,
        (1.0 - height / layout.1) * 100.0,
    );

    //Update percent
    set_percent((
        (pos.0 / layout.0 * 100.0).clamp(0.0, max_percent.0),
        (pos.1 / layout.1 * 100.0).clamp(0.0, max_percent.1),
    ));

    //Report setting back to game ECS world
    let true_percent = (percent.0 / max_percent.0, percent.1 / max_percent.1);
    context.query_world::<Commands, _, _>(|mut commands| {
        commands.insert_resource(true_percent);
    });

    let background = Style {
        background_color: StyleProp::Value(props.box_color),
        ..default()
    };

    let button = Style {
        background_color: StyleProp::Value(props.button_color),
        height: StyleProp::Value(Units::Pixels(width)),
        width: StyleProp::Value(Units::Pixels(height)),
        left: StyleProp::Value(Units::Percentage(percent.0)),
        top: StyleProp::Value(Units::Percentage(percent.1)),
        ..default()
    };

    rsx! {
        <widgets::Background styles={Some(background)} on_layout={Some(on_layout)} >
            <widgets::Background on_event={drag_handler} styles={Some(button) } />
        </widgets::Background>
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(CLEAR))
        .insert_resource(WindowDescriptor {
            width: HEIGHT * RESOLUTION,
            height: HEIGHT,
            title: "Bevy Template".to_string(),
            present_mode: PresentMode::Fifo,
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .insert_resource(WorldInspectorParams {
            enabled: false,
            ..Default::default()
        })
        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(spawn_camera)
        .add_system(toggle_inspector)
        .add_system(print_percent)
        .add_plugin(BevyKayakUIPlugin)
        .add_startup_system(create_ui)
        .add_system(update_heatlh)
        .register_inspectable::<Player>()
        .run();
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = OrthographicCameraBundle::new_2d();

    camera.orthographic_projection.right = 1.0 * RESOLUTION;
    camera.orthographic_projection.left = -1.0 * RESOLUTION;

    camera.orthographic_projection.top = 1.0;
    camera.orthographic_projection.bottom = -1.0;

    camera.orthographic_projection.scaling_mode = ScalingMode::None;

    commands.spawn_bundle(camera);
}

fn toggle_inspector(
    input: ResMut<Input<KeyCode>>,
    mut window_params: ResMut<WorldInspectorParams>,
) {
    if input.just_pressed(KeyCode::Grave) {
        window_params.enabled = !window_params.enabled
    }
}

#[allow(dead_code)]
fn slow_down() {
    std::thread::sleep(std::time::Duration::from_secs_f32(1.000));
}
