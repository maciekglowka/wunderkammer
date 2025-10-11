/// Adapted from the official macroquad asteroids example
/// https://github.com/not-fl3/macroquad/blob/master/examples/asteroids.rs
///
/// Example usage of the Wunderkammer component storage
/// [obviously an overkill in case of a simple game like Asteroids ;]
use macroquad::prelude::*;
use wunderkammer::prelude::*;

const SHIP_HEIGHT: f32 = 25.;
const SHIP_BASE: f32 = 22.;
const CYAN: Color = Color::new(0., 1., 1., 1.);

#[derive(Default, ComponentSet)]
struct Components {
    asteroid: ComponentStorage<Asteroid>,
    bullet: ComponentStorage<Bullet>,
    pos: ComponentStorage<Vec2>,
    rot: ComponentStorage<f32>,
    ship: ComponentStorage<()>, // marker component
    vel: ComponentStorage<Vec2>,
}

#[derive(Clone, Copy)]
struct Asteroid {
    rot_speed: f32,
    size: f32,
    sides: u8,
}

#[derive(Clone, Copy)]
struct Bullet {
    shot_at: f64,
}

#[derive(Default)]
struct Resources {
    last_shot: f64,
    gameover: bool,
}

type World = WorldStorage<Components, Resources>;

#[macroquad::main("Asteroids")]
async fn main() {
    let mut world = setup();
    loop {
        clear_background(BLACK);
        if world.res.gameover {
            // Handle game restart
            let text = "Press [enter] to play.";
            let font_size = 30.;
            let text_size = measure_text(text, None, font_size as _, 1.0);
            draw_text(
                text,
                0.5 * (screen_width() - text_size.width),
                0.5 * (screen_height() - text_size.height),
                font_size,
                WHITE,
            );
            if is_key_down(KeyCode::Enter) {
                world = setup();
            }
            next_frame().await;
            continue;
        }

        // UPDATE GAME
        let frame_t = get_time();
        handle_ship_movement(&mut world);

        if is_key_down(KeyCode::Space) {
            shoot(frame_t, &mut world);
        }

        handle_kinematics(&mut world);
        handle_bullets(frame_t, &mut world);
        handle_collisions(&mut world);

        // CHECK WIN
        if query!(world, With(asteroid)).next().is_none() {
            world.res.gameover = true;
        }

        // DRAW
        draw_ship(&world);
        draw_bullets(&world);
        draw_asteroids(&world);
        next_frame().await;
    }
}

fn setup() -> World {
    let mut world = World::default();
    world.res.last_shot = get_time();

    // spawn ship
    let ship_entity = spawn_object(
        Vec2::new(screen_width() / 2., screen_height() / 2.),
        Vec2::splat(0.),
        Some(0.),
        &mut world,
    );
    insert!(world, ship, ship_entity, ());

    // spawn asteroids
    spawn_initial_asteroids(&mut world);

    world
}

fn spawn_initial_asteroids(world: &mut World) {
    let screen_center = Vec2::new(screen_width() / 2., screen_height() / 2.);
    for _ in 0..10 {
        spawn_asteroid(
            screen_center
                + Vec2::new(rand::gen_range(-1., 1.), rand::gen_range(-1., 1.)).normalize()
                    * screen_width().min(screen_height())
                    / 2.,
            Vec2::new(rand::gen_range(-1., 1.), rand::gen_range(-1., 1.)),
            rand::gen_range(3, 8),
            screen_width().min(screen_height()) / 10.,
            world,
        );
    }
}

fn spawn_asteroid(pos: Vec2, vel: Vec2, sides: u8, size: f32, world: &mut World) {
    let entity = spawn_object(pos, vel, Some(0.), world);
    insert!(
        world,
        asteroid,
        entity,
        Asteroid {
            rot_speed: rand::gen_range(-2., 2.),
            sides,
            size,
        }
    );
}

fn spawn_object(pos: Vec2, vel: Vec2, rot: Option<f32>, world: &mut World) -> Entity {
    let entity = world.spawn();
    insert!(world, pos, entity, pos);
    insert!(world, vel, entity, vel);
    if let Some(rot) = rot {
        insert!(world, rot, entity, rot);
    }
    entity
}

fn handle_ship_movement(world: &mut World) -> Option<()> {
    let &entity = query!(world, With(ship)).next()?;
    let vel = world.cmps.vel.get_mut(&entity)?;
    let rot = world.cmps.rot.get_mut(&entity)?;
    let mut acc = -*vel / 100.; // friction

    // accelerate
    if is_key_down(KeyCode::Up) {
        acc = Vec2::new(rot.sin(), -rot.cos()) / 3.;
    };
    *vel += acc;

    // clamp speed
    if vel.length() > 5. {
        *vel = vel.normalize() * 5.;
    }

    // steer
    if is_key_down(KeyCode::Right) {
        *rot += 5. * 3.14 / 180.;
    }
    if is_key_down(KeyCode::Left) {
        *rot -= 5. * 3.14 / 180.;
    }

    Some(())
}

fn shoot(frame_t: f64, world: &mut World) {
    if frame_t - world.res.last_shot < 0.5 {
        return;
    };
    let Some((_, &pos, &rot, _)) = query_iter!(world, With(pos, rot, ship)).next() else {
        return;
    };
    let rot_vec = Vec2::new(rot.sin(), -rot.cos());
    let entity = spawn_object(pos + rot_vec * SHIP_HEIGHT / 2., rot_vec * 7., None, world);
    insert!(world, bullet, entity, Bullet { shot_at: frame_t });
    world.res.last_shot = frame_t;
}

fn handle_bullets(frame_t: f64, world: &mut World) {
    // bullet lifetime
    let to_remove = query_iter!(world, With(bullet))
        .filter(|(_, b)| b.shot_at + 1.5 < frame_t)
        .map(|(e, _)| e)
        .collect::<Vec<_>>();

    for entity in to_remove {
        world.despawn(entity);
    }
}

fn handle_collisions(world: &mut World) {
    let Some((_, _, &ship_pos)) = query_iter!(world, With(ship, pos)).next() else {
        return;
    };

    let mut to_split = Vec::new();

    for (a_entity, a_pos, asteroid) in query_iter!(world, With(pos, asteroid)) {
        // player collision
        if (*a_pos - ship_pos).length() < asteroid.size + SHIP_HEIGHT / 3. {
            world.res.gameover = true;
        }

        // bullet collisions
        for (b_entity, b_pos, b_vel, _) in query_iter!(world, With(pos, vel, bullet)) {
            if (*a_pos - *b_pos).length() < asteroid.size {
                // cache data to generate child asteroids
                to_split.push((a_entity, *a_pos, *asteroid, b_entity, *b_vel));
                break;
            }
        }
    }

    // despawn and split
    for (a_entity, a_pos, asteroid, b_entity, b_vel) in to_split {
        world.despawn(a_entity);
        world.despawn(b_entity);

        if asteroid.sides > 3 {
            spawn_asteroid(
                a_pos,
                Vec2::new(b_vel.y, -b_vel.x).normalize() * rand::gen_range(1., 3.),
                asteroid.sides - 1,
                0.8 * asteroid.size,
                world,
            );
            spawn_asteroid(
                a_pos,
                Vec2::new(-b_vel.y, b_vel.x).normalize() * rand::gen_range(1., 3.),
                asteroid.sides - 1,
                0.8 * asteroid.size,
                world,
            );
        }
    }
}

fn handle_kinematics(world: &mut World) {
    // move objects
    query_execute!(world, With(pos, vel), |_, pos: &mut Vec2, vel: &Vec2| {
        *pos += *vel;
        *pos = wrap_around(pos);
    });

    // rotate asteroids
    query_execute!(
        world,
        With(rot, asteroid),
        |_, rot: &mut f32, asteroid: &Asteroid| {
            *rot += asteroid.rot_speed;
        }
    );
}

fn wrap_around(v: &Vec2) -> Vec2 {
    let mut vr = Vec2::new(v.x, v.y);
    if vr.x > screen_width() {
        vr.x = 0.;
    }
    if vr.x < 0. {
        vr.x = screen_width()
    }
    if vr.y > screen_height() {
        vr.y = 0.;
    }
    if vr.y < 0. {
        vr.y = screen_height()
    }
    vr
}

fn draw_ship(world: &World) {
    for (_, pos, rot, _) in query_iter!(world, With(pos, rot, ship)) {
        let v1 = Vec2::new(
            pos.x + rot.sin() * SHIP_HEIGHT / 2.,
            pos.y - rot.cos() * SHIP_HEIGHT / 2.,
        );
        let v2 = Vec2::new(
            pos.x - rot.cos() * SHIP_BASE / 2. - rot.sin() * SHIP_HEIGHT / 2.,
            pos.y - rot.sin() * SHIP_BASE / 2. + rot.cos() * SHIP_HEIGHT / 2.,
        );
        let v3 = Vec2::new(
            pos.x + rot.cos() * SHIP_BASE / 2. - rot.sin() * SHIP_HEIGHT / 2.,
            pos.y + rot.sin() * SHIP_BASE / 2. + rot.cos() * SHIP_HEIGHT / 2.,
        );
        draw_triangle_lines(v1, v2, v3, 2., CYAN);
    }
}

fn draw_bullets(world: &World) {
    for (_, pos, _) in query_iter!(world, With(pos, bullet)) {
        draw_circle(pos.x, pos.y, 2., CYAN);
    }
}

fn draw_asteroids(world: &World) {
    for (_, pos, rot, asteroid) in query_iter!(world, With(pos, rot, asteroid)) {
        draw_poly_lines(
            pos.x,
            pos.y,
            asteroid.sides,
            asteroid.size,
            *rot,
            2.,
            MAGENTA,
        );
    }
}
