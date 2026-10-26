// Implementing core's trait directly on iced's builder, from a third crate.
use d3::path::PathContext;
use iced::widget::canvas::path::Builder;
impl PathContext for Builder {
    fn move_to(&mut self, _: f64, _: f64) {}
    fn line_to(&mut self, _: f64, _: f64) {}
    fn quadratic_curve_to(&mut self, _: f64, _: f64, _: f64, _: f64) {}
    fn bezier_curve_to(&mut self, _: f64, _: f64, _: f64, _: f64, _: f64, _: f64) {}
    fn arc(&mut self, _: f64, _: f64, _: f64, _: f64, _: f64, _: bool) {}
    fn close_path(&mut self) {}
}
fn main() {}
