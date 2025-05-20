use macroquad::prelude::*;

#[macroquad::main("CLicker Game")]
async fn main() {
    let mut score = 0;
    let r = 70.;
    loop {
        let (x, y) = (screen_width() / 2., screen_height() / 2.);
        let circle = Circle::new(x, y, r);
        clear_background(GRAY);

        if is_mouse_button_pressed(MouseButton::Left) {
            let (mouse_x, mouse_y) = mouse_position();
            let mouse_circ = Circle::new(mouse_x, mouse_y, 1.);

            if circle.overlaps(&mouse_circ) {
                score += 1;
            }
        }

        draw_text("Clicker Game", screen_width() / 2. - 100., 100., 50., WHITE);
        draw_text(
            format!("Clicks: {}", score).as_str(),
            screen_width() / 2. - 100.,
            500.,
            50.,
            WHITE,
        );
        draw_circle(x, y, r, RED);
        next_frame().await;

        rand
    }
}
