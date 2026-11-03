//! Isolated signed wire/native spatial/direction translation of 4271D92F type5. Not registered in the product.
//! 4B92D0 normalizes native X/2 and Y; 42F300 advances unsigned fixed16 fields,
//! rounds X to an even native coordinate and compares the original destination.
//! No clock, actor lookup, damage, sound or rendering here.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativePoint {
    pub x: i32,
    pub y: i32,
}

/// 4A5F89..4A5FC9: preserve the complete signed32 X result.
pub fn from_wire(x: u16, y: u16) -> NativePoint {
    NativePoint { x: 2 * i32::from(x) - 32768, y: i32::from(y) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Active,
    Arrived,
    Exhausted,
    Retired,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sample {
    pub position: NativePoint,
    pub state: State,
    pub remaining: i32,
}

pub struct Flight {
    destination: NativePoint,
    fixed: [u32; 2],
    vector: [i32; 2],
    remaining: i32,
    state: State,
    position: NativePoint,
}

/// 4B9210 integer direction selection, before vector even-X alignment.
/// This returns the native index only; image-direction mapping is not qualified.
pub fn native_direction(origin: NativePoint, destination: NativePoint, previous: u8) -> u8 {
    let dx = destination.x - origin.x;
    let y20 = (destination.y - origin.y) * 20;
    if dx == 0 && y20 == 0 { return previous; }
    if y20 > dx * 24 {
        if y20 > -dx * 24 { 4 }
        else if y20 > -dx * 4 { 5 }
        else { 6 + u8::from(y20 <= dx * 4) }
    } else if y20 > dx * 4 { 3 }
    else if y20 > -dx * 4 { 2 }
    else { u8::from(y20 > -dx * 24) }
}

impl Flight {
    /// Inputs are already native coordinates sampled by the admitted entry.
    /// The host must call `dispatch` immediately once at entry, then only for
    /// admitted timer callbacks. Captured points must be in the recorded wire domain.
    pub fn from_captured_points(origin: NativePoint, destination: NativePoint) -> Self {
        assert!((-32768..=98302).contains(&origin.x) && (-32768..=98302).contains(&destination.x));
        assert!((0..=65535).contains(&origin.y) && (0..=65535).contains(&destination.y));
        let dx = (destination.x & !1) - origin.x;
        let dy = destination.y - origin.y;
        let nx = dx.unsigned_abs() >> 1;
        let ny = dy.unsigned_abs();
        let (mut vx, mut vy) = if nx == 0 && ny == 0 {
            (0, 0)
        } else if nx > ny {
            (0x10000, (ny as i32).wrapping_shl(16) / nx as i32)
        } else {
            ((nx as i32).wrapping_shl(16) / ny as i32, 0x10000)
        };
        if dx < 0 {
            vx = -vx;
        }
        if dy < 0 {
            vy = -vy;
        }
        Self {
            destination,
            fixed: [(origin.x as u32) << 16, (origin.y as u32) << 16],
            vector: [vx << 1, vy],
            remaining: (origin.x - destination.x)
                .abs()
                .max((origin.y - destination.y).abs()),
            state: State::Active,
            position: origin,
        }
    }

    pub fn state(&self) -> State {
        self.state
    }

    /// One native advance; no elapsed-time catch-up or target re-sampling.
    /// A terminal result supplies no future samples. Native queue insertion,
    /// frame selection and impact construction are separate, pending host work.
    pub fn dispatch(&mut self) -> Option<Sample> {
        if self.state != State::Active {
            return None;
        }
        self.fixed[0] = self.fixed[0].wrapping_add(self.vector[0] as u32);
        self.fixed[1] = self.fixed[1].wrapping_add(self.vector[1] as u32);
        self.position = NativePoint {
            x: ((self.fixed[0].wrapping_add(0x8000) >> 16) as i32) & !1,
            y: (self.fixed[1].wrapping_add(0x8000) >> 16) as i32,
        };
        self.remaining -= 1;
        if self.position == self.destination {
            self.remaining = 0;
            self.state = State::Arrived;
        } else if self.remaining <= 0 {
            self.state = State::Exhausted;
        }
        Some(Sample {
            position: self.position,
            state: self.state,
            remaining: self.remaining,
        })
    }

    /// Retirement is permanent for this object; it never creates an impact.
    pub fn retire(&mut self) {
        self.state = State::Retired;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn p(x: i32, y: i32) -> NativePoint { NativePoint { x, y } }
    fn trace(origin: NativePoint, destination: NativePoint) -> (Vec<NativePoint>, State) {
        let mut flight = Flight::from_captured_points(origin, destination);
        let mut samples = Vec::new();
        while let Some(sample) = flight.dispatch() {
            samples.push(sample.position);
            assert!(samples.len() <= 131070);
        }
        (samples, flight.state())
    }
    #[test]
    fn eight_native_directions_reach_on_the_third_dispatch() {
        let cases = [
            (p(100, 97), vec![p(100, 99), p(100, 98), p(100, 97)]),
            (p(106, 97), vec![p(102, 99), p(104, 98), p(106, 97)]),
            (p(106, 100), vec![p(102, 100), p(104, 100), p(106, 100)]),
            (p(106, 103), vec![p(102, 101), p(104, 102), p(106, 103)]),
            (p(100, 103), vec![p(100, 101), p(100, 102), p(100, 103)]),
            (p(94, 103), vec![p(98, 101), p(96, 102), p(94, 103)]),
            (p(94, 100), vec![p(98, 100), p(96, 100), p(94, 100)]),
            (p(94, 97), vec![p(98, 99), p(96, 98), p(94, 97)]),
        ];
        for (destination, expected) in cases {
            assert_eq!(trace(p(100, 100), destination), (expected, State::Arrived));
        }
    }
    #[test]
    fn asymmetric_positive_rounding_preserves_native_even_x() {
        assert_eq!(trace(p(100, 100), p(110, 103)),
            (vec![p(102,101),p(104,101),p(106,102),p(108,102),p(110,103)],State::Arrived));
    }
    #[test]
    fn asymmetric_negative_rounding_preserves_native_bias() {
        assert_eq!(trace(p(110, 103), p(100, 100)),
            (vec![p(108,102),p(106,102),p(104,101),p(102,101),p(100,100)],State::Arrived));
    }
    #[test]
    fn y_dominant_motion_has_the_native_repeated_x_samples() {
        assert_eq!(trace(p(100, 100), p(104, 105)),
            (vec![p(100,101),p(102,102),p(102,103),p(102,104),p(104,105)],State::Arrived));
    }
    #[test]
    fn odd_destination_is_not_silently_snapped_or_clamped() {
        assert_eq!(trace(p(100, 100), p(103, 100)),
            (vec![p(102,100),p(104,100),p(106,100)],State::Exhausted));
    }
    #[test]
    fn fixed16_wrap_is_retained_in_the_nonarrival_boundary_case() {
        assert_eq!(trace(p(65530, 10), p(65533, 10)),
            (vec![p(65532,10),p(65534,10),p(0,10)],State::Exhausted));
    }
    #[test]
    fn coincident_points_arrive_once_on_the_immediate_dispatch() {
        let mut flight=Flight::from_captured_points(p(100,100),p(100,100));
        assert_eq!(flight.dispatch(),Some(Sample{position:p(100,100),state:State::Arrived,remaining:0}));
        assert_eq!(flight.dispatch(),None);
    }
    #[test]
    fn retirement_stops_late_callbacks_without_an_arrival() {
        let mut flight=Flight::from_captured_points(p(100,100),p(106,103));
        assert_eq!(flight.dispatch().unwrap().position,p(102,101));
        flight.retire();
        assert_eq!(flight.dispatch(),None);
        assert_eq!(flight.state(),State::Retired);
    }

    #[test]
    fn direction_octants_match_native_indices() {
        let destinations=[p(100,97),p(106,97),p(106,100),p(106,103),p(100,103),p(94,103),p(94,100),p(94,97)];
        for (expected,destination) in destinations.into_iter().enumerate() {
            assert_eq!(native_direction(p(100,100),destination,255),expected as u8);
        }
    }
    #[test]
    fn direction_threshold_ties_are_not_replaced_with_atan_rounding() {
        let cases=[(p(110,102),2),(p(110,103),3),(p(110,112),3),(p(110,113),4),
            (p(110,98),1),(p(110,99),2),(p(110,88),0),(p(110,89),1),
            (p(90,102),6),(p(90,103),5),(p(90,112),5),(p(90,113),4),
            (p(90,98),7),(p(90,99),6),(p(90,88),0),(p(90,89),7)];
        for (destination,expected) in cases { assert_eq!(native_direction(p(100,100),destination,255),expected); }
    }
    #[test]
    fn coincident_direction_preserves_the_callers_full_byte() {
        for previous in [0,1,7,8,127,255] { assert_eq!(native_direction(p(100,100),p(100,100),previous),previous); }
    }
    #[test]
    fn direction_uses_raw_destination_before_vector_even_x_alignment() {
        assert_eq!(native_direction(p(100,100),p(105,101),0),2);
        assert_eq!(native_direction(p(100,100),p(104,101),0),3);
    }

    #[test]
    fn wire_conversion_preserves_full_signed_domain_without_clamping() {
        for (wire,native) in [(0,-32768),(16383,-2),(16384,0),(32768,32768),(49151,65534),(49152,65536),(65535,98302)] {
            assert_eq!(from_wire(wire,65535),p(native,65535));
        }
    }
    #[test]
    fn translated_short_wire_path_retains_three_native_dispatches() {
        let (points,state)=trace(from_wire(32768,32768),from_wire(32771,32771));
        assert_eq!((points,state),(vec![p(32770,32769),p(32772,32770),p(32774,32771)],State::Arrived));
    }
    #[test]
    fn negative_coincident_destination_is_not_rewritten_to_wrapped_position() {
        assert_eq!(trace(from_wire(16383,10),from_wire(16383,10)),(vec![p(65534,10)],State::Exhausted));
    }
    #[test]
    fn high_coincident_destination_is_not_rewritten_to_wrapped_position() {
        assert_eq!(trace(from_wire(49152,10),from_wire(49152,10)),(vec![p(0,10)],State::Exhausted));
    }
    #[test]
    fn x_dominant_large_minor_shift_uses_native_signed_division() {
        let mut f=Flight::from_captured_points(p(-32768,0),p(98302,32768));
        assert_eq!(f.vector,[131072,-32768]);
        assert_eq!(f.dispatch().unwrap().position,p(32770,0));
        assert_eq!(f.dispatch().unwrap().position,p(32772,65535));
    }
    #[test]
    fn y_dominant_large_minor_shift_uses_native_signed_division() {
        let mut f=Flight::from_captured_points(p(0,0),p(65536,65535));
        assert_eq!(f.vector,[-65536,65536]);
        assert_eq!(f.dispatch().unwrap().position,p(65534,1));
    }
}
