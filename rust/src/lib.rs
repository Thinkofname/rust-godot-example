#[macro_use]
extern crate godot;
extern crate rand;
use rand::{thread_rng, Rng};

use godot::types;

const PLAYER_SPEED: f32 = 400.0;

gdclass! {
    class RustPlayer: types::Area2D {
        fields {
            velocity: types::Vector2,
            screensize: types::Vector2,
        }
        setup(builder) {
            builder.signal("hit")
                .register();
        }
        constructor(godot_info) {
            RustPlayer {
                godot_info: godot_info,
                velocity: types::Vector2::new(0.0, 0.0),
                // TODO: get_viewport_rect().size in `_ready` when Rect2 is supported
                screensize: types::Vector2::new(480.0, 720.0),
            }
        }

        export fn _ready(&mut self) {
            let p = self.godot_parent();
            p.set_process(true);
            gprint_warn!("Ready called");
        }

        export fn _process(&mut self, delta: f32) {
            let p = self.godot_parent();
            self.velocity = types::Vector2::new(0.0, 0.0);
            let input = types::Input::godot_singleton();
            if input.is_action_pressed("ui_right") {
                self.velocity.set_x(1.0);
            }
            if input.is_action_pressed("ui_left") {
                self.velocity.set_x(-1.0);
            }
            if input.is_action_pressed("ui_down") {
                self.velocity.set_y(1.0);
            }
            if input.is_action_pressed("ui_up") {
                self.velocity.set_y(-1.0);
            }

            let ani = p.get_node(types::NodePath::new("./AnimatedSprite"))
                .and_then(|v| v.cast::<types::AnimatedSprite>())
                .unwrap();

            if self.velocity.length() > 0.0 {
                self.velocity = self.velocity.normalized() * PLAYER_SPEED;
                ani.play("");
            } else {
                ani.stop();
            }
            let mut pos = p.get_position();
            pos = pos + self.velocity * delta;
            let x = pos.x().min(self.screensize.x()).max(0.0);
            pos.set_x(x);
            let y = pos.y().min(self.screensize.y()).max(0.0);
            pos.set_y(y);
            p.set_position(pos);

            if self.velocity.x() != 0.0 {
                ani.set_animation("right");
                ani.set_flip_v(false);
                ani.set_flip_h(self.velocity.x() < 0.0);
            } else if self.velocity.y() != 0.0 {
                ani.set_animation("up");
                ani.set_flip_v(self.velocity.y() > 0.0);
            }
        }

        export fn _on_player_area_entered(&mut self) {
            let p = self.godot_parent();
            gprint_warn!("Player hit!");
            p.hide();
            p.call_deferred("emit_signal", &[
                types::Variant::new_string("hit")
            ]);
            p.call_deferred("set_monitoring", &[types::Variant::new_bool(false)]);
        }

        export fn start(&mut self, pos: types::Vector2) {
            let p = self.godot_parent();
            p.set_position(pos);
            p.show();
            p.set_monitoring(true);
        }
    }
}


const MOB_MIN_SPEED: f32 = 150.0;
const MOB_MAX_SPEED: f32 = 250.0;
const MOB_TYPES: &'static [&'static str] = &[
    "walk",
    "swim",
    "fly"
];

gdclass! {
    class RustMob: types::RigidBody2D {
        fields {

        }
        setup(_builder) {
        }
        constructor(godot_info) {
            RustMob {
                godot_info: godot_info,
            }
        }

        export fn _ready(&mut self) {
            let p = self.godot_parent();

            let ani = p.get_node(types::NodePath::new("./AnimatedSprite"))
                .and_then(|v| v.cast::<types::AnimatedSprite>())
                .unwrap();
            let mut rng = thread_rng();
            ani.set_animation(rng.choose(MOB_TYPES).unwrap());
        }

        export fn _on_visible_screen_exited(&mut self) {
            let p = self.godot_parent();
            p.queue_free();
        }
    }
}

gdclass! {
    class RustMain: types::RigidBody2D {
        fields {
            score: i32,
            mob: godot::GodotRef<types::PackedScene>,
        }
        setup(_builder) {
        }
        constructor(godot_info) {
            let res = types::_ResourceLoader::godot_singleton();
            let mob = res.load("res://Mob.tscn", "", false)
                .and_then(|v| v.cast::<types::PackedScene>())
                .unwrap();
            RustMain {
                godot_info: godot_info,
                score: 0,
                mob: mob,
            }
        }

        export fn new_game(&mut self) {
            let p = self.godot_parent();
            self.score = 0;
            gprint_warn!("New game called");
            let player = p.get_node(types::NodePath::new("./Player"))
                .and_then(|v| v.cast_native::<RustPlayer>())
                .unwrap();
            let mut player = player.borrow_mut();
            let start_position = p.get_node(types::NodePath::new("./StartPosition"))
                .and_then(|v| v.cast::<types::Position2D>())
                .unwrap();
            player.start(start_position.get_position());

            let start_timer = p.get_node(types::NodePath::new("./StartTimer"))
                .and_then(|v| v.cast::<types::Timer>())
                .unwrap();
            start_timer.start();

            let hud = p.get_node(types::NodePath::new("./HUD"))
                .and_then(|v| v.cast_native::<RustHUD>())
                .unwrap();
            let mut hud = hud.borrow_mut();
            hud.update_score(self.score as i64);
            hud.show_message("Get Ready");
        }

        export fn game_over(&mut self) {
            let p = self.godot_parent();

            let score_timer = p.get_node(types::NodePath::new("./ScoreTimer"))
                .and_then(|v| v.cast::<types::Timer>())
                .unwrap();
            score_timer.stop();
            let mob_timer = p.get_node(types::NodePath::new("./MobTimer"))
                .and_then(|v| v.cast::<types::Timer>())
                .unwrap();
            mob_timer.stop();

            let hud = p.get_node(types::NodePath::new("./HUD"))
                .and_then(|v| v.cast_native::<RustHUD>())
                .unwrap();
            let mut hud = hud.borrow_mut();
            hud.show_game_over();
        }

        export fn _on_start_timer_timeout(&mut self) {
            let p = self.godot_parent();

            let score_timer = p.get_node(types::NodePath::new("./ScoreTimer"))
                .and_then(|v| v.cast::<types::Timer>())
                .unwrap();
            score_timer.start();
            let mob_timer = p.get_node(types::NodePath::new("./MobTimer"))
                .and_then(|v| v.cast::<types::Timer>())
                .unwrap();
            mob_timer.start();
        }

        export fn _on_score_timer_timeout(&mut self) {
            let p = self.godot_parent();
            self.score += 1;
            let hud = p.get_node(types::NodePath::new("./HUD"))
                .and_then(|v| v.cast_native::<RustHUD>())
                .unwrap();
            let mut hud = hud.borrow_mut();
            hud.update_score(self.score as i64);
        }

        export fn _on_mob_timer_timeout(&mut self) {
            let p = self.godot_parent();
            let mut rng = thread_rng();
            let mob_loc = p.get_node(types::NodePath::new("./MobPath/MobSpawnLocation"))
                .and_then(|v| v.cast::<types::PathFollow2D>())
                .unwrap();
            mob_loc.set_offset(rng.gen::<i64>().abs() as f64);
            let mob = self.mob.instance(0)
                .and_then(|v| v.cast::<types::RigidBody2D>())
                .unwrap();

            p.add_child(mob.cast(), false);

            let mut direction = mob_loc.get_rotation();
            mob.set_position(mob_loc.get_position());
            direction += rng.gen_range(
                -::std::f64::consts::PI/4.0,
                ::std::f64::consts::PI/4.0
            );
            mob.set_rotation(direction + ::std::f64::consts::PI/2.0);
            mob.set_linear_velocity(types::Vector2::new(
                rng.gen_range(MOB_MIN_SPEED, MOB_MAX_SPEED),
                0.0
            ).rotated(direction as f32));
        }
    }
}

gdclass! {
    class RustHUD: types::CanvasLayer {
        fields {
            show_def_message: bool,
        }
        setup(builder) {
            builder.signal("start_game")
                .register();
        }
        constructor(godot_info) {
            RustHUD {
                godot_info: godot_info,
                show_def_message: false,
            }
        }


        export fn show_game_over(&mut self) {
            self.show_message("Game Over");
            self.show_def_message = true;
        }

        export fn reset_message(&mut self) {
            let p = self.godot_parent();
            let message_label = p.get_node(types::NodePath::new("./MessageLabel"))
                .and_then(|v| v.cast::<types::Label>())
                .unwrap();
            if self.show_def_message {
                message_label.set_text("Dodge the Creeps!");
                message_label.show();
                let start_button = p.get_node(types::NodePath::new("./StartButton"))
                    .and_then(|v| v.cast::<types::Button>())
                    .unwrap();
                start_button.show();
                self.show_def_message = false;
            } else {
                message_label.hide();
            }
        }

        export fn _on_start_button_pressed(&mut self) {
            let p = self.godot_parent();
            let start_button = p.get_node(types::NodePath::new("./StartButton"))
                .and_then(|v| v.cast::<types::Button>())
                .unwrap();
            start_button.hide();
            p.call_deferred("emit_signal", &[
                types::Variant::new_string("start_game")
            ]);
        }
    }
}

impl RustHUD {
    fn show_message(&mut self, text: &str) {
        let p = self.godot_parent();
        let message_label = p.get_node(types::NodePath::new("./MessageLabel"))
            .and_then(|v| v.cast::<types::Label>())
            .unwrap();
        message_label.set_text(text);
        message_label.show();

        let message_timer = p.get_node(types::NodePath::new("./MessageTimer"))
            .and_then(|v| v.cast::<types::Timer>())
            .unwrap();
        message_timer.start();
    }
    fn update_score(&mut self, score: i64) {
        let p = self.godot_parent();
        let score_label = p.get_node(types::NodePath::new("./ScoreLabel"))
            .and_then(|v| v.cast::<types::Label>())
            .unwrap();
        score_label.set_text(score.to_string());
    }
}

gd_init! {
    RustPlayer,
    RustMob,
    RustMain,
    RustHUD
}