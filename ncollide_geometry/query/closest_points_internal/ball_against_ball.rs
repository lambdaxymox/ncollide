use na;
use math::Point;
use query::ClosestPoints;
use shape::Ball;

/// Proximity between balls.
#[inline]
pub fn ball_against_ball<P>(
    center1: &P,
    b1: &Ball<P::Real>,
    center2: &P,
    b2: &Ball<P::Real>,
    margin: P::Real,
) -> ClosestPoints<P>
where
    P: Point,
{
    assert!(
        margin >= na::zero(),
        "The proximity margin must be positive or null."
    );

    let r1 = b1.radius();
    let r2 = b2.radius();
    let delta_pos = *center2 - *center1;
    let distance_squared = na::norm_squared(&delta_pos);
    let sum_radius = r1 + r2;
    let sum_radius_with_error = sum_radius + margin;

    if distance_squared <= sum_radius_with_error * sum_radius_with_error {
        if distance_squared <= sum_radius * sum_radius {
            ClosestPoints::Intersecting
        } else {
            let normal = na::normalize(&delta_pos);
            ClosestPoints::WithinMargin(*center1 + normal * r1, *center2 + normal * (-r2))
        }
    } else {
        ClosestPoints::Disjoint
    }
}
