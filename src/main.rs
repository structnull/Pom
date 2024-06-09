use eframe::{run_native, App, NativeOptions};
use egui::{CentralPanel, Color32, Context, Pos2, Shape, Stroke, Vec2};
use std::time::{Duration, Instant};

const TIME: u64 = 25;


struct Pom {
    state: TimerState,
    last_update: Instant,
    remaining_time: Duration,
    total_duration: Duration,
}


enum TimerState {
    Ready,
    Running,
    Paused,
    Finished,
}

impl Pom {
    fn new() -> Self {
        let total_duration = Duration::new(TIME * 60, 0);
        Self {
            state: TimerState::Ready,
            last_update: Instant::now(),
            remaining_time: total_duration,
            total_duration,
        }
    }

    // Start the timer
    fn start_timer(&mut self) {
        self.state = TimerState::Running;
        self.last_update = Instant::now();
    }

    fn pause_timer(&mut self) {
        if let TimerState::Running = self.state {
            self.state = TimerState::Paused;
        }
    }

    fn reset_timer(&mut self) {
        self.state = TimerState::Ready;
        self.remaining_time = self.total_duration;
    }

    fn update_timer(&mut self) {
        if let TimerState::Running = self.state {
            let now = Instant::now();
            let elapsed = now - self.last_update;
            if self.remaining_time > elapsed {
                self.remaining_time -= elapsed;
            } else {
                self.remaining_time = Duration::new(0, 0);
                self.state = TimerState::Finished;
            }
            self.last_update = now;
        }
    }

    // Format the duration into a MM:SS string
    fn format_duration(duration: Duration) -> String {
        let minutes = duration.as_secs() / 60;
        let seconds = duration.as_secs() % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    // Calculate the progress of the timer as a fraction
    fn progress(&self) -> f32 {
        (self.total_duration.as_secs_f32() - self.remaining_time.as_secs_f32())
            / self.total_duration.as_secs_f32()
    }

    // Draw an arc representing the progress of the timer
    fn draw_arc(
        painter: &egui::Painter,
        center: Pos2,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        stroke: Stroke,
    ) {
        let segments = 20; // Reduced number of segments
        let angle_step = (end_angle - start_angle) / segments as f32;
        let points: Vec<Pos2> = (0..=segments)
            .map(|i| {
                let angle = start_angle + i as f32 * angle_step;
                Pos2 {
                    x: center.x + radius * angle.cos(),
                    y: center.y + radius * angle.sin(),
                }
            })
            .collect();
        painter.add(Shape::line(points, stroke));
    }
}

impl App for Pom {
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {}

    fn auto_save_interval(&self) -> Duration {
        Duration::from_secs(30)
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        Color32::from_rgba_unmultiplied(12, 12, 12, 180).to_normalized_gamma_f32()
    }

    fn persist_egui_memory(&self) -> bool {
        true
    }

    fn raw_input_hook(&mut self, _ctx: &Context, _raw_input: &mut egui::RawInput) {}

    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.update_timer();

        // Request a repaint to ensure the UI continuously updates (fixes the hover issue)
        ctx.request_repaint();

        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Pomodoro Timer");

            // Calculate the size and position of the circular timer display
            let available_size = ui.available_size();
            let size = Vec2::new(
                available_size.x.min(available_size.y),
                available_size.x.min(available_size.y),
            );
            let (rect, _response) = ui.allocate_at_least(size, egui::Sense::hover());

            let painter = ui.painter();
            let center = rect.center();
            let radius = rect.width() / 2.0;
            let progress_angle = self.progress() * std::f32::consts::TAU; // wth

            // Draw the circular progress bar background (need rework)
            painter.circle_stroke(center, radius, Stroke::new(10.0, Color32::from_gray(80)));

            // Draw the progress arcccccccccccccccccccc
            Pom::draw_arc(
                painter,
                center,
                radius,
                0.0,
                progress_angle,
                Stroke::new(10.0, Color32::from_rgb(100, 200, 100)),
            );


            let text = Pom::format_duration(self.remaining_time);
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                text,
                egui::TextStyle::Heading.resolve(ui.style()),
                Color32::WHITE,
            );

            // Add buttons to control the timer
            ui.horizontal(|ui| {
                if ui.button("Start").clicked() {
                    self.start_timer();
                }

                if ui.button("Pause").clicked() {
                    self.pause_timer();
                }

                if ui.button("Reset").clicked() {
                    self.reset_timer();
                }
            });

            // Display status message based on timer state
            match self.state {
                TimerState::Finished => {
                    ui.label("Time's up!");
                }
                TimerState::Paused => {
                    ui.label("Paused");
                }
                _ => {}
            }
        });
    }
}

fn main() {
    let app = "Pom";
    let win_opt = NativeOptions::default();

    let _ = run_native(app, win_opt, Box::new(|_cc| Box::new(Pom::new())));
}
