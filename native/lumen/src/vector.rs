type Point = (f32, f32);

const PIXELS_PER_STEP: f32 = 3.0;
const MAX_STEPS: f32 = 256.0;

pub fn stroke(path: &str, view: f32, size: u32, width: f32) -> Vec<u8> {
    let scale = size as f32 / view;
    let half = width * scale / 2.0;
    let side = size as usize;
    let mut nearest = vec![f32::INFINITY; side * side];
    for line in polylines(path, scale) {
        let pixels: Vec<Point> = line.iter().map(|&(x, y)| (x * scale, y * scale)).collect();
        for pair in pixels.windows(2) {
            reach_segment(&mut nearest, side, pair[0], pair[1], half + 1.0);
        }
    }
    nearest.iter().map(|&distance| coverage(half, distance)).collect()
}

fn coverage(half: f32, distance: f32) -> u8 {
    ((half - distance + 0.5).clamp(0.0, 1.0) * 255.0).round() as u8
}

fn reach_segment(nearest: &mut [f32], side: usize, a: Point, b: Point, reach: f32) {
    let limit = side as f32;
    let left = (a.0.min(b.0) - reach).floor().clamp(0.0, limit) as usize;
    let right = (a.0.max(b.0) + reach).ceil().clamp(0.0, limit) as usize;
    let top = (a.1.min(b.1) - reach).floor().clamp(0.0, limit) as usize;
    let bottom = (a.1.max(b.1) + reach).ceil().clamp(0.0, limit) as usize;
    for y in top..bottom {
        for x in left..right {
            let distance = distance_to_segment((x as f32 + 0.5, y as f32 + 0.5), a, b);
            let cell = &mut nearest[y * side + x];
            if distance < *cell {
                *cell = distance;
            }
        }
    }
}

fn distance_to_segment(p: Point, a: Point, b: Point) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let length_squared = dx * dx + dy * dy;
    let along = if length_squared > 0.0 {
        (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / length_squared).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (nx, ny) = (a.0 + along * dx - p.0, a.1 + along * dy - p.1);
    (nx * nx + ny * ny).sqrt()
}

fn polylines(path: &str, scale: f32) -> Vec<Vec<Point>> {
    let mut tokens = Tokens { bytes: path.as_bytes(), at: 0 };
    let mut pen = Pen::new(scale);
    let mut previous: Option<u8> = None;
    loop {
        let letter = if let Some(letter) = tokens.letter() {
            letter
        } else if tokens.starts_number() {
            match previous {
                Some(b'M') => b'L',
                Some(b'm') => b'l',
                Some(b'Z' | b'z') | None => break,
                Some(letter) => letter,
            }
        } else {
            break;
        };
        if pen.apply(letter, &mut tokens).is_none() {
            break;
        }
        previous = Some(letter);
    }
    pen.lines.into_iter().filter(|line| line.len() >= 2).collect()
}

struct Tokens<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Tokens<'_> {
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }

    fn skip_separators(&mut self) {
        while matches!(self.peek(), Some(b' ' | b',' | b'\t' | b'\n' | b'\r')) {
            self.at += 1;
        }
    }

    fn letter(&mut self) -> Option<u8> {
        self.skip_separators();
        let letter = self.peek().filter(|byte| byte.is_ascii_alphabetic())?;
        self.at += 1;
        Some(letter)
    }

    fn starts_number(&mut self) -> bool {
        self.skip_separators();
        matches!(self.peek(), Some(b'0'..=b'9' | b'.' | b'-' | b'+'))
    }

    fn skip_digits(&mut self) -> usize {
        let start = self.at;
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.at += 1;
        }
        self.at - start
    }

    fn number(&mut self) -> Option<f32> {
        self.skip_separators();
        let start = self.at;
        if matches!(self.peek(), Some(b'-' | b'+')) {
            self.at += 1;
        }
        let mut digits = self.skip_digits();
        if self.peek() == Some(b'.') {
            self.at += 1;
            digits += self.skip_digits();
        }
        if digits == 0 {
            self.at = start;
            return None;
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            let before_exponent = self.at;
            self.at += 1;
            if matches!(self.peek(), Some(b'-' | b'+')) {
                self.at += 1;
            }
            if self.skip_digits() == 0 {
                self.at = before_exponent;
            }
        }
        std::str::from_utf8(&self.bytes[start..self.at]).ok()?.parse().ok()
    }

    fn flag(&mut self) -> Option<bool> {
        self.skip_separators();
        let flag = match self.peek()? {
            b'0' => false,
            b'1' => true,
            _ => return None,
        };
        self.at += 1;
        Some(flag)
    }

    fn point(&mut self, origin: Point) -> Option<Point> {
        let x = self.number()?;
        let y = self.number()?;
        Some((origin.0 + x, origin.1 + y))
    }
}

struct Pen {
    scale: f32,
    lines: Vec<Vec<Point>>,
    current: Point,
    start: Point,
    cubic_control: Option<Point>,
    quad_control: Option<Point>,
}

impl Pen {
    fn new(scale: f32) -> Pen {
        Pen {
            scale,
            lines: Vec::new(),
            current: (0.0, 0.0),
            start: (0.0, 0.0),
            cubic_control: None,
            quad_control: None,
        }
    }

    fn apply(&mut self, letter: u8, tokens: &mut Tokens) -> Option<()> {
        let relative = letter.is_ascii_lowercase();
        let origin = if relative { self.current } else { (0.0, 0.0) };
        let previous_cubic = self.cubic_control.take();
        let previous_quad = self.quad_control.take();
        let reflect = |control: Option<Point>, current: Point| match control {
            Some(c) => (2.0 * current.0 - c.0, 2.0 * current.1 - c.1),
            None => current,
        };
        match letter.to_ascii_uppercase() {
            b'M' => {
                let point = tokens.point(origin)?;
                self.move_to(point);
            }
            b'L' => {
                let point = tokens.point(origin)?;
                self.line_to(point);
            }
            b'H' => {
                let x = tokens.number()? + origin.0;
                self.line_to((x, self.current.1));
            }
            b'V' => {
                let y = tokens.number()? + origin.1;
                self.line_to((self.current.0, y));
            }
            b'C' | b'S' => {
                let first = if letter.to_ascii_uppercase() == b'C' {
                    tokens.point(origin)?
                } else {
                    reflect(previous_cubic, self.current)
                };
                let second = tokens.point(origin)?;
                let end = tokens.point(origin)?;
                self.cubic_to(first, second, end);
                self.cubic_control = Some(second);
            }
            b'Q' | b'T' => {
                let control = if letter.to_ascii_uppercase() == b'Q' {
                    tokens.point(origin)?
                } else {
                    reflect(previous_quad, self.current)
                };
                let end = tokens.point(origin)?;
                self.quad_to(control, end);
                self.quad_control = Some(control);
            }
            b'A' => {
                let rx = tokens.number()?;
                let ry = tokens.number()?;
                let degrees = tokens.number()?;
                let large = tokens.flag()?;
                let sweep = tokens.flag()?;
                let end = tokens.point(origin)?;
                self.arc_to(rx, ry, degrees, large, sweep, end);
            }
            b'Z' => {
                let start = self.start;
                self.line_to(start);
            }
            _ => return None,
        }
        Some(())
    }

    fn move_to(&mut self, point: Point) {
        self.lines.push(vec![point]);
        self.current = point;
        self.start = point;
    }

    fn line_to(&mut self, point: Point) {
        if self.lines.is_empty() {
            self.lines.push(vec![self.current]);
        }
        if let Some(line) = self.lines.last_mut() {
            line.push(point);
        }
        self.current = point;
    }

    fn steps(&self, length: f32) -> usize {
        (length * self.scale / PIXELS_PER_STEP).ceil().clamp(1.0, MAX_STEPS) as usize
    }

    fn cubic_to(&mut self, first: Point, second: Point, end: Point) {
        let start = self.current;
        let steps = self.steps(span(start, first) + span(first, second) + span(second, end));
        for step in 1..steps {
            let t = step as f32 / steps as f32;
            let u = 1.0 - t;
            let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            self.line_to((
                a * start.0 + b * first.0 + c * second.0 + d * end.0,
                a * start.1 + b * first.1 + c * second.1 + d * end.1,
            ));
        }
        self.line_to(end);
    }

    fn quad_to(&mut self, control: Point, end: Point) {
        let start = self.current;
        let steps = self.steps(span(start, control) + span(control, end));
        for step in 1..steps {
            let t = step as f32 / steps as f32;
            let u = 1.0 - t;
            let (a, b, c) = (u * u, 2.0 * u * t, t * t);
            self.line_to((
                a * start.0 + b * control.0 + c * end.0,
                a * start.1 + b * control.1 + c * end.1,
            ));
        }
        self.line_to(end);
    }

    fn arc_to(&mut self, rx: f32, ry: f32, degrees: f32, large: bool, sweep: bool, end: Point) {
        let start = self.current;
        if start == end {
            return;
        }
        let (mut rx, mut ry) = (rx.abs(), ry.abs());
        if rx == 0.0 || ry == 0.0 {
            self.line_to(end);
            return;
        }
        let (sin, cos) = degrees.to_radians().sin_cos();
        let (half_dx, half_dy) = ((start.0 - end.0) / 2.0, (start.1 - end.1) / 2.0);
        let x1 = cos * half_dx + sin * half_dy;
        let y1 = -sin * half_dx + cos * half_dy;
        let overflow = (x1 * x1) / (rx * rx) + (y1 * y1) / (ry * ry);
        if overflow > 1.0 {
            rx *= overflow.sqrt();
            ry *= overflow.sqrt();
        }
        let numerator = rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1;
        let denominator = rx * rx * y1 * y1 + ry * ry * x1 * x1;
        let mut root = (numerator / denominator).max(0.0).sqrt();
        if large == sweep {
            root = -root;
        }
        let centre_x1 = root * rx * y1 / ry;
        let centre_y1 = -root * ry * x1 / rx;
        let centre = (
            cos * centre_x1 - sin * centre_y1 + (start.0 + end.0) / 2.0,
            sin * centre_x1 + cos * centre_y1 + (start.1 + end.1) / 2.0,
        );
        let from = ((x1 - centre_x1) / rx, (y1 - centre_y1) / ry);
        let to = ((-x1 - centre_x1) / rx, (-y1 - centre_y1) / ry);
        let first_angle = angle_between((1.0, 0.0), from);
        let mut sweep_angle = angle_between(from, to);
        if !sweep && sweep_angle > 0.0 {
            sweep_angle -= std::f32::consts::TAU;
        } else if sweep && sweep_angle < 0.0 {
            sweep_angle += std::f32::consts::TAU;
        }
        let steps = self.steps(sweep_angle.abs() * rx.max(ry));
        for step in 1..steps {
            let (s, c) = (first_angle + sweep_angle * step as f32 / steps as f32).sin_cos();
            self.line_to((
                centre.0 + rx * c * cos - ry * s * sin,
                centre.1 + rx * c * sin + ry * s * cos,
            ));
        }
        self.line_to(end);
    }
}

fn span(a: Point, b: Point) -> f32 {
    ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt()
}

fn angle_between(u: Point, v: Point) -> f32 {
    (u.0 * v.1 - u.1 * v.0).atan2(u.0 * v.0 + u.1 * v.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(pixels: &[u8], size: u32, x: u32, y: u32) -> u8 {
        pixels[(y * size + x) as usize]
    }

    #[test]
    fn a_horizontal_line_covers_the_pixels_on_it_and_not_those_far_away() {
        let pixels = stroke("M4 12h16", 24.0, 24, 2.0);
        assert_eq!(at(&pixels, 24, 12, 11), 255);
        assert_eq!(at(&pixels, 24, 12, 12), 255);
        assert_eq!(at(&pixels, 24, 12, 4), 0);
        assert_eq!(at(&pixels, 24, 1, 12), 0);
    }

    #[test]
    fn a_dot_path_draws_a_round_dot() {
        let pixels = stroke("M12 12h.01", 24.0, 48, 4.0);
        assert_eq!(at(&pixels, 48, 23, 23), 255);
        assert!(at(&pixels, 48, 20, 23) > 200);
        assert!(at(&pixels, 48, 23, 20) > 200);
        assert_eq!(at(&pixels, 48, 20, 20), 0);
    }

    #[test]
    fn relative_and_absolute_forms_of_a_path_give_the_same_pixels() {
        let relative = "m4 6h16v8H4zm4 3.2v2.2M3.5 7.5a2 2 0 0 1 2-2h4c1 0 2 1 2 2s1 2 2 2q1 1 2 0t2 0";
        let absolute = "M4 6L20 6L20 14L4 14ZM8 9.2L8 11.4M3.5 7.5A2 2 0 0 1 5.5 5.5H9.5C10.5 5.5 11.5 6.5 11.5 7.5S12.5 9.5 13.5 9.5Q14.5 10.5 15.5 9.5T17.5 9.5";
        assert_eq!(stroke(relative, 24.0, 64, 1.6), stroke(absolute, 24.0, 64, 1.6));
    }

    #[test]
    fn a_full_circle_of_arcs_puts_ink_at_radius_eight_and_none_at_the_centre() {
        let pixels = stroke("M12 4a8 8 0 1 0 0 16 8 8 0 0 0 0-16z", 24.0, 48, 1.6);
        for (x, y) in [(39, 23), (8, 23), (23, 7), (23, 40), (35, 12)] {
            assert_eq!(at(&pixels, 48, x, y), 255, "at {x},{y}");
        }
        assert_eq!(at(&pixels, 48, 23, 23), 0);
        assert_eq!(at(&pixels, 48, 30, 23), 0);
    }

    #[test]
    fn the_parser_reads_numbers_written_without_separators() {
        assert_eq!(polylines("M1.5.5l-2-3", 1.0), vec![vec![(1.5, 0.5), (-0.5, -2.5)]]);
        assert_eq!(polylines("M1e1-2E-1L0 0", 1.0), vec![vec![(10.0, -0.2), (0.0, 0.0)]]);
    }
}
