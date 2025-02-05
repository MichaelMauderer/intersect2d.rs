/*
Line segment intersection detection library.

Copyright (C) 2021 eadf https://github.com/eadf

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License as published by the Free Software
Foundation, either version 3 of the License, or (at your option) any later
version.

This program is distributed in the hope that it will be useful, but WITHOUT
ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with
this program. If not, see <https://www.gnu.org/licenses/>.

Also add information on how to contact you by electronic and paper mail.

If the program does terminal interaction, make it output a short notice like
this when it starts in an interactive mode:

intersection2d Copyright (C) 2021 eadf

This program comes with ABSOLUTELY NO WARRANTY; for details type `show w'.

This is free software, and you are welcome to redistribute it under certain
conditions; type `show c' for details.

The hypothetical commands `show w' and `show c' should show the appropriate
parts of the General Public License. Of course, your program's commands might
be different; for a GUI interface, you would use an "about box".

You should also get your employer (if you work as a programmer) or school,
if any, to sign a "copyright disclaimer" for the program, if necessary. For
more information on this, and how to apply and follow the GNU GPL, see <https://www.gnu.org/licenses/>.

The GNU General Public License does not permit incorporating your program
into proprietary programs. If your program is a subroutine library, you may
consider it more useful to permit linking proprietary applications with the
library. If this is what you want to do, use the GNU Lesser General Public
License instead of this License. But first, please read <https://www.gnu.org/
licenses /why-not-lgpl.html>.
 */

#![deny(non_camel_case_types)]
#![deny(unused_parens)]
#![deny(non_upper_case_globals)]
#![deny(unused_qualifications)]
#![deny(unused_results)]
#![deny(unused_imports)]

use core::fmt;
use geo::algorithm::intersects::Intersects;
use num_traits::{Float, Zero};
use thiserror::Error;

pub mod algorithm;

#[derive(Error, Debug)]
pub enum IntersectError {
    #[error("Something bad happened")]
    InternalError(String),
    #[error("No NaN, inf etc. are allowed")]
    InvalidData(String),
    #[error("When searching for intersections in LineStrings the 'ignore_end_point_intersections' parameter must be set to 'true'.")]
    InvalidSearchParameter(String),
    #[error("Results already taken from the algorithm data struct")]
    ResultsAlreadyTaken(String),
}

/// Utility function converting an array slice into a vec of Line
#[allow(dead_code)]
pub fn to_lines<U, T>(points: &[[U; 4]]) -> Vec<geo::Line<T>>
where
    U: num_traits::ToPrimitive + Copy,
    T: Float + approx::UlpsEq + geo::CoordFloat,
    T::Epsilon: Copy,
{
    let mut rv = Vec::with_capacity(points.len());
    for p in points.iter() {
        rv.push(geo::Line::<T>::new(
            geo::Coordinate {
                x: T::from(p[0]).unwrap(),
                y: T::from(p[1]).unwrap(),
            },
            geo::Coordinate {
                x: T::from(p[2]).unwrap(),
                y: T::from(p[3]).unwrap(),
            },
        ));
    }
    rv
}

/// Get any intersection point between line segment and point.
/// Inspired by <https://stackoverflow.com/a/17590923>
pub fn intersect_line_point<T>(
    line: &geo::Line<T>,
    point: &geo::Coordinate<T>,
) -> Option<Intersection<T>>
where
    T: Float + Zero + geo::CoordFloat + approx::AbsDiffEq + approx::UlpsEq,
    T::Epsilon: Copy,
{
    // take care of end point equality
    if approx::ulps_eq!(&line.start.x, &point.x) && approx::ulps_eq!(&line.start.y, &point.y) {
        return Some(Intersection::Intersection(*point));
    }
    if approx::ulps_eq!(&line.end.x, &point.x) && approx::ulps_eq!(&line.end.y, &point.y) {
        return Some(Intersection::Intersection(*point));
    }

    let x1 = line.start.x;
    let x2 = line.end.x;
    let y1 = line.start.y;
    let y2 = line.end.y;
    let x = point.x;
    let y = point.y;

    let ab = ((x2 - x1) * (x2 - x1) + (y2 - y1) * (y2 - y1)).sqrt();
    let ap = ((x - x1) * (x - x1) + (y - y1) * (y - y1)).sqrt();
    let pb = ((x2 - x) * (x2 - x) + (y2 - y) * (y2 - y)).sqrt();

    #[cfg(feature = "console_trace")]
    println!("ab={:?}, ap={:?}, pb={:?}, ap+pb={:?}", ab, ap, pb, ap + pb);
    if approx::ulps_eq!(&ab, &(ap + pb)) {
        return Some(Intersection::Intersection(*point));
    }
    None
}

#[allow(dead_code)]
pub enum Intersection<T>
where
    T: Float + Zero + geo::CoordFloat + approx::AbsDiffEq + approx::UlpsEq,
    T::Epsilon: Copy,
{
    // Normal one point intersection
    Intersection(geo::Coordinate<T>),
    // Collinear overlapping
    OverLap(geo::Line<T>),
}

impl<T> Intersection<T>
where
    T: Float + Zero + geo::CoordFloat + approx::AbsDiffEq + approx::UlpsEq,
    T::Epsilon: Copy,
{
    /// return a single, simple intersection point
    pub fn single(&self) -> geo::Coordinate<T> {
        match self {
            Self::OverLap(a) => a.start,
            Self::Intersection(a) => *a,
        }
    }
}

impl<T> fmt::Debug for Intersection<T>
where
    T: Float + Zero + geo::CoordFloat + approx::AbsDiffEq + approx::UlpsEq,
    T::Epsilon: Copy,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OverLap(a) => a.fmt(f),
            Self::Intersection(a) => a.fmt(f),
        }
    }
}

/// Get any intersection point between lines.
/// Note that this function always detects endpoint-to-endpoint intersections.
/// Most of this is from <https://stackoverflow.com/a/565282>
#[allow(clippy::many_single_char_names)]
pub fn intersect<T>(one: &geo::Line<T>, other: &geo::Line<T>) -> Option<Intersection<T>>
where
    T: Float + Zero + geo::CoordFloat + approx::AbsDiffEq + approx::UlpsEq,
    T::Epsilon: Copy,
{
    #[allow(clippy::suspicious_operation_groupings)]
    {
        // AABB tests
        if one.end.x > other.end.x
            && one.end.x > other.start.x
            && one.start.x > other.end.x
            && one.start.x > other.start.x
        {
            return None;
        }
        if one.end.x < other.end.x
            && one.end.x < other.start.x
            && one.start.x < other.end.x
            && one.start.x < other.start.x
        {
            return None;
        }
        if one.end.y > other.end.y
            && one.end.y > other.start.y
            && one.start.y > other.end.y
            && one.start.y > other.start.y
        {
            return None;
        }
        if one.end.y < other.end.y
            && one.end.y < other.start.y
            && one.start.y < other.end.y
            && one.start.y < other.start.y
        {
            return None;
        }
    }
    let p = one.start;
    let q = other.start;
    let r = one.end - p;
    let s = other.end - q;

    let r_cross_s = cross_z(&r, &s);
    let q_minus_p = q - p;
    let q_minus_p_cross_r = cross_z(&q_minus_p, &r);

    // If r × s = 0 then the two lines are parallel
    if approx::ulps_eq!(&r_cross_s, &T::zero()) {
        // one (or both) of the lines may be a point
        let one_is_a_point = ulps_eq_c(&one.start, &one.end);
        let other_is_a_point = ulps_eq_c(&other.start, &other.end);
        if one_is_a_point || other_is_a_point {
            if one_is_a_point && other_is_a_point && ulps_eq_c(&one.start, &other.start) {
                return Some(Intersection::Intersection(one.start));
            }
            return if one_is_a_point {
                intersect_line_point(other, &one.start)
            } else {
                intersect_line_point(one, &other.start)
            };
        }

        // If r × s = 0 and (q − p) × r = 0, then the two lines are collinear.
        if approx::ulps_eq!(&q_minus_p_cross_r, &T::zero()) {
            let r_dot_r = dot(&r, &r);
            let r_div_r_dot_r = div(&r, r_dot_r);
            let s_dot_r = dot(&s, &r);
            let t0 = dot(&q_minus_p, &r_div_r_dot_r);
            let t1 = t0 + s_dot_r / r_dot_r;

            Some(Intersection::OverLap(geo::Line::new(
                scale_to_coordinate(&p, &r, t0),
                scale_to_coordinate(&p, &r, t1),
            )))
        } else {
            // If r × s = 0 and (q − p) × r ≠ 0,
            // then the two lines are parallel and non-intersecting.
            None
        }
    } else {
        // the lines are not parallel
        let t = cross_z(&q_minus_p, &div(&s, r_cross_s));
        let u = cross_z(&q_minus_p, &div(&r, r_cross_s));
        Some(Intersection::Intersection(scale_to_coordinate(&p, &r, t)))
    }
}

#[inline(always)]
pub fn scale_to_coordinate<T>(
    point: &geo::Coordinate<T>,
    vector: &geo::Coordinate<T>,
    scale: T,
) -> geo::Coordinate<T>
where
    T: Float + Zero + geo::CoordFloat + approx::AbsDiffEq + approx::UlpsEq,
    T::Epsilon: Copy,
{
    geo::Coordinate {
        x: point.x + scale * vector.x,
        y: point.y + scale * vector.y,
    }
}

#[inline(always)]
/// Divides a 'vector' by 'b'. Obviously, don't feed this with 'b' == 0
fn div<T>(a: &geo::Coordinate<T>, b: T) -> geo::Coordinate<T>
where
    T: Float + Zero + geo::CoordFloat + approx::AbsDiffEq + approx::UlpsEq,
    T::Epsilon: Copy,
{
    geo::Coordinate {
        x: a.x / b,
        y: a.y / b,
    }
}

#[inline(always)]
/// from https://stackoverflow.com/a/565282 :
///  "Define the 2-dimensional vector cross product v × w to be vx wy − vy wx."
/// This function returns the z component of v × w
fn cross_z<T>(a: &geo::Coordinate<T>, b: &geo::Coordinate<T>) -> T
where
    T: Float + Zero + geo::CoordFloat + approx::AbsDiffEq + approx::UlpsEq,
    T::Epsilon: Copy,
{
    a.x * b.y - a.y * b.x
}

#[inline(always)]
/// calculate the dot product of two lines
fn dot<T>(a: &geo::Coordinate<T>, b: &geo::Coordinate<T>) -> T
where
    T: Float + Zero + geo::CoordFloat + approx::AbsDiffEq + approx::UlpsEq,
    T::Epsilon: Copy,
{
    a.x * b.x + a.y * b.y
}

/// Trait for self intersection tests where the end points are excluded
pub trait SelfIntersectingExclusive<T>
where
    T: Float
        + num_traits::ToPrimitive
        + geo::GeoFloat
        + geo::CoordFloat
        + approx::AbsDiffEq
        + approx::UlpsEq,
    T::Epsilon: Copy,
{
    /// Returns true if any line intersects any other line in the collection.
    fn is_self_intersecting(&self) -> Result<bool, IntersectError>;

    /// Returns a list of intersection points and the involved lines, if any intersections are found.
    #[allow(clippy::type_complexity)]
    fn self_intersections<'a>(
        &self,
    ) -> Result<
        Box<dyn ExactSizeIterator<Item = (geo::Coordinate<T>, Vec<usize>)> + 'a>,
        IntersectError,
    >
    where
        T: 'a;
}

/// Trait for self intersection tests where the end points are included
pub trait SelfIntersectingInclusive<T>
where
    T: Float
        + num_traits::ToPrimitive
        + geo::GeoFloat
        + geo::CoordFloat
        + approx::AbsDiffEq
        + approx::UlpsEq,
    T::Epsilon: Copy,
{
    /// Returns true if any line intersects any other line in the collection.
    /// If the end points are identical they will be reported too.
    fn is_self_intersecting_inclusive(&self) -> Result<bool, IntersectError>;

    /// Returns a list of intersection points and the involved lines, if any intersections are found.
    /// If the end points are identical they will be reported too.
    #[allow(clippy::type_complexity)]
    fn self_intersections_inclusive<'a>(
        &self,
    ) -> Result<
        Box<dyn ExactSizeIterator<Item = (geo::Coordinate<T>, Vec<usize>)> + 'a>,
        IntersectError,
    >
    where
        T: 'a;
}

impl<T> SelfIntersectingInclusive<T> for Vec<geo::Line<T>>
where
    T: Float
        + num_traits::ToPrimitive
        + geo::GeoFloat
        + geo::CoordFloat
        + approx::AbsDiffEq
        + approx::UlpsEq,
    T::Epsilon: Copy,
{
    /// Returns true if the LineString is self intersecting.
    /// LineStrings.
    /// ```
    /// # use intersect2d::SelfIntersectingInclusive;
    ///
    /// let lines: Vec<geo::Line<_>> = geo::LineString::from(vec![
    ///     (100., 100.),
    ///     (200., 100.),
    ///     (200., 200.),
    ///     (100., 200.),
    ///     (100., 100.),
    /// ]).lines().collect();
    /// assert!(lines.is_self_intersecting_inclusive().unwrap());
    ///
    /// let lines: Vec<geo::Line<_>> = geo::LineString::from(vec![
    ///    (100., 100.),
    ///    (200., 100.),
    ///    (200., 200.),
    ///    (150., 50.),
    ///    (100., 200.),
    ///    (100., 100.),
    /// ]).lines().collect();
    /// assert!(lines.is_self_intersecting_inclusive().unwrap());
    /// ```
    fn is_self_intersecting_inclusive(&self) -> Result<bool, IntersectError> {
        // at around >25 line segments the sweep-line algorithm is faster
        if self.len() < 25 {
            for l1 in self.iter().enumerate() {
                for l2 in self.iter().skip(l1.0 + 1) {
                    if l1.1.intersects(l2) {
                        return Ok(true);
                    }
                }
            }
            Ok(false)
        } else {
            Ok(algorithm::AlgorithmData::<T>::default()
                .with_ignore_end_point_intersections(false)?
                .with_stop_at_first_intersection(true)?
                .with_ref_lines(self.iter())?
                .compute()?
                .next()
                .is_some())
        }
    }

    /// Returns an iterator containing the found intersections.
    /// ```
    /// # use intersect2d::SelfIntersectingInclusive;
    /// # use intersect2d::ulps_eq_c;
    ///
    /// let lines : Vec<geo::Line<_>>= geo::LineString::from(vec![
    ///     (100., 100.),
    ///     (200., 100.),
    ///     (200., 200.),
    ///     (100., 200.),
    ///     (100., 100.),
    /// ]).lines().collect();
    ///
    /// assert!(!lines.self_intersections_inclusive().expect("err").count()>0);
    ///
    /// let lines : Vec<geo::Line<_>> = geo::LineString::from(vec![
    ///    (100., 100.),
    ///    (200., 100.),
    ///    (200., 200.),
    ///    (150., 50.),
    ///    (100., 200.),
    ///    (100., 100.),
    /// ]).lines().collect();
    /// let rv :Vec<(geo::Coordinate<_>,Vec<usize>)> =
    ///     lines.self_intersections_inclusive().expect("err").collect();
    /// for f in rv.iter() {
    ///   println!("{:?}", f);
    /// }
    /// assert_eq!(rv.len(), 7);
    /// assert!(ulps_eq_c(&rv[0].0, &geo::Coordinate{x: 200., y: 100.0}));
    /// assert_eq!(rv[0].1, vec!(0_usize, 1));
    /// assert!(ulps_eq_c(&rv[1].0, &geo::Coordinate{x: 166.66666666666666, y: 100.0}));
    /// assert_eq!(rv[1].1, vec!(0_usize, 2));
    /// assert!(ulps_eq_c(&rv[2].0, &geo::Coordinate{x: 133.33333333333333, y: 100.0}));
    /// assert_eq!(rv[2].1, vec!(0_usize, 3));
    /// assert!(ulps_eq_c(&rv[3].0, &geo::Coordinate{x: 100., y: 100.0}));
    /// assert_eq!(rv[3].1, vec!(0_usize, 4));
    /// // and more...
    ///
    /// ```
    #[allow(clippy::type_complexity)]
    fn self_intersections_inclusive<'a>(
        &self,
    ) -> Result<
        Box<dyn ExactSizeIterator<Item = (geo::Coordinate<T>, Vec<usize>)> + 'a>,
        IntersectError,
    >
    where
        T: 'a,
    {
        if self.len() < 25 {
            // at around <25 line segments the brute force test is faster

            // sanity check for each line
            for a_line in self.iter() {
                if !a_line.start.x.is_finite()
                    || !a_line.start.y.is_finite()
                    || !a_line.end.x.is_finite()
                    || !a_line.end.y.is_finite()
                {
                    return Err(IntersectError::InvalidData(
                        "Can't check for intersections on non-finite data".to_string(),
                    ));
                }
            }
            let mut rv = Vec::<(geo::Coordinate<T>, Vec<usize>)>::new();
            for l1 in self.iter().enumerate() {
                for l2 in self.iter().enumerate().skip(l1.0 + 1) {
                    if let Some(i) = intersect(l1.1, l2.1) {
                        rv.push((i.single(), vec![l1.0, l2.0]));
                    }
                }
            }
            // This will only return intersections between two lines at a single point
            // If more than that are intersecting it will be reported once for each pair.
            // Todo: fix it!
            Ok(Box::new(rv.into_iter()))
        } else {
            // at around >25 line segments the sweep-line algorithm is faster
            algorithm::AlgorithmData::<T>::default()
                .with_ignore_end_point_intersections(false)?
                .with_stop_at_first_intersection(false)?
                .with_ref_lines(self.iter())?
                .compute()
        }
    }
}

impl<T> SelfIntersectingExclusive<T> for Vec<geo::Line<T>>
where
    T: Float
        + num_traits::ToPrimitive
        + geo::GeoFloat
        + geo::CoordFloat
        + approx::AbsDiffEq
        + approx::UlpsEq,
    T::Epsilon: Copy,
{
    /// Returns true if the LineString is self intersecting.
    /// LineStrings.
    /// ```
    /// # use intersect2d::SelfIntersectingExclusive;
    ///
    /// let lines: Vec<geo::Line<_>> = geo::LineString::from(vec![
    ///     (100., 100.),
    ///     (200., 100.),
    ///     (200., 200.),
    ///     (100., 200.),
    ///     (100., 100.),
    /// ]).lines().collect();
    /// assert!(!lines.is_self_intersecting().unwrap());
    ///
    /// let lines: Vec<geo::Line<_>> = geo::LineString::from(vec![
    ///    (100., 100.),
    ///    (200., 100.),
    ///    (200., 200.),
    ///    (150., 50.),
    ///    (100., 200.),
    ///    (100., 100.),
    /// ]).lines().collect();
    /// assert!(lines.is_self_intersecting().unwrap());
    /// ```
    fn is_self_intersecting(&self) -> Result<bool, IntersectError> {
        // at around >25 line segments the sweep-line algorithm is faster
        if self.len() < 25 {
            // sanity check for each line
            for a_line in self.iter() {
                if !a_line.start.x.is_finite()
                    || !a_line.start.y.is_finite()
                    || !a_line.end.x.is_finite()
                    || !a_line.end.y.is_finite()
                {
                    return Err(IntersectError::InvalidData(
                        "Can't check for intersections on non-finite data".to_string(),
                    ));
                }
            }
            for l1 in self.iter().enumerate() {
                for l2 in self.iter().skip(l1.0 + 1) {
                    if ulps_eq_c(&l1.1.start, &l2.start)
                        || ulps_eq_c(&l1.1.start, &l2.end)
                        || ulps_eq_c(&l1.1.end, &l2.start)
                        || ulps_eq_c(&l1.1.end, &l2.end)
                    {
                        continue;
                    }
                    if l1.1.intersects(l2) {
                        return Ok(true);
                    }
                }
            }
            Ok(false)
        } else {
            Ok(algorithm::AlgorithmData::<T>::default()
                .with_ignore_end_point_intersections(true)?
                .with_stop_at_first_intersection(true)?
                .with_ref_lines(self.iter())?
                .compute()?
                .next()
                .is_some())
        }
    }

    /// Returns an iterator containing the found intersections.
    /// ```
    /// # use intersect2d::SelfIntersectingExclusive;
    /// # use intersect2d::ulps_eq_c;
    ///
    /// let lines : Vec<geo::Line<_>>= geo::LineString::from(vec![
    ///     (100., 100.),
    ///     (200., 100.),
    ///     (200., 200.),
    ///     (100., 200.),
    ///     (100., 100.),
    /// ]).lines().collect();
    ///
    /// let rv :Vec<(geo::Coordinate<_>,Vec<usize>)> =
    ///     lines.self_intersections().expect("err").collect();
    /// assert!(rv.is_empty());
    ///
    /// let lines : Vec<geo::Line<_>> = geo::LineString::from(vec![
    ///    (100., 100.),
    ///    (200., 100.),
    ///    (200., 200.),
    ///    (150., 50.),
    ///    (100., 200.),
    ///    (100., 100.),
    /// ]).lines().collect();
    /// let rv :Vec<(geo::Coordinate<_>,Vec<usize>)> =
    ///     lines.self_intersections().expect("err").collect();
    ///
    /// assert_eq!(rv.len(), 2);
    /// assert_eq!(rv[0].1, vec!(0_usize, 2));
    /// assert!(ulps_eq_c(&rv[0].0, &geo::Coordinate{x: 166.66666666666666, y: 100.0}));
    /// assert_eq!(rv[1].1, vec!(0_usize, 3));
    /// assert!(ulps_eq_c(&rv[1].0, &geo::Coordinate{x: 133.33333333333333, y: 100.0}));
    /// ```
    #[allow(clippy::type_complexity)]
    fn self_intersections<'a>(
        &self,
    ) -> Result<
        Box<dyn ExactSizeIterator<Item = (geo::Coordinate<T>, Vec<usize>)> + 'a>,
        IntersectError,
    >
    where
        T: 'a,
    {
        if self.len() < 25 {
            // at around <25 line segments the brute force test is faster

            // sanity check for each line
            for a_line in self.iter() {
                if !a_line.start.x.is_finite()
                    || !a_line.start.y.is_finite()
                    || !a_line.end.x.is_finite()
                    || !a_line.end.y.is_finite()
                {
                    return Err(IntersectError::InvalidData(
                        "Can't check for intersections on non-finite data".to_string(),
                    ));
                }
            }
            let mut rv = Vec::<(geo::Coordinate<T>, Vec<usize>)>::new();
            for l1 in self.iter().enumerate() {
                for l2 in self.iter().enumerate().skip(l1.0 + 1) {
                    if ulps_eq_c(&l1.1.start, &l2.1.start)
                        || ulps_eq_c(&l1.1.start, &l2.1.end)
                        || ulps_eq_c(&l1.1.end, &l2.1.start)
                        || ulps_eq_c(&l1.1.end, &l2.1.end)
                    {
                        continue;
                    }
                    if let Some(i) = intersect(l1.1, l2.1) {
                        rv.push((i.single(), vec![l1.0, l2.0]));
                    }
                }
            }
            // This will only return intersections between two lines at a single point
            // If more than that are intersecting it will be reported once for each pair.
            // Todo: fix it!
            Ok(Box::new(rv.into_iter()))
        } else {
            // at around >25 line segments the sweep-line algorithm is faster
            algorithm::AlgorithmData::<T>::default()
                .with_ignore_end_point_intersections(true)?
                .with_stop_at_first_intersection(false)?
                .with_ref_lines(self.iter())?
                .compute()
        }
    }
}

impl<T> SelfIntersectingExclusive<T> for geo::LineString<T>
where
    T: Float
        + num_traits::ToPrimitive
        + geo::GeoFloat
        + geo::CoordFloat
        + approx::AbsDiffEq
        + approx::UlpsEq,
    T::Epsilon: Copy,
{
    /// Returns true if the LineString is self intersecting.
    /// The 'ignore_end_point_intersections' parameter must always be set to true when testing
    /// LineStrings.
    /// ```
    /// # use intersect2d::SelfIntersectingExclusive;
    ///
    /// let line_string = geo::LineString::from(vec![
    ///     (100., 100.),
    ///     (200., 100.),
    ///     (200., 200.),
    ///     (100., 200.),
    ///     (100., 100.),
    /// ]);
    /// assert!(!line_string.is_self_intersecting().unwrap());
    ///
    /// let line_string = geo::LineString::from(vec![
    ///    (100., 100.),
    ///    (200., 100.),
    ///    (200., 200.),
    ///    (150., 50.),
    ///    (100., 200.),
    ///    (100., 100.),
    /// ]);
    /// assert!(line_string.is_self_intersecting().unwrap());
    /// ```
    fn is_self_intersecting(&self) -> Result<bool, IntersectError> {
        // at around >25 line segments the sweep-line algorithm is faster
        if self.0.len() < 25 {
            // sanity check for each line
            for point in self.points_iter() {
                if !point.x().is_finite() || !point.y().is_finite() {
                    return Err(IntersectError::InvalidData(
                        "Can't check for intersections on non-finite data".to_string(),
                    ));
                }
            }
            for l1 in self.lines().enumerate() {
                for l2 in self.lines().skip(l1.0 + 1) {
                    if ulps_eq_c(&l1.1.start, &l2.start)
                        || ulps_eq_c(&l1.1.start, &l2.end)
                        || ulps_eq_c(&l1.1.end, &l2.start)
                        || ulps_eq_c(&l1.1.end, &l2.end)
                    {
                        continue;
                    }
                    if l1.1.intersects(&l2) {
                        return Ok(true);
                    }
                }
            }
            Ok(false)
        } else {
            Ok(algorithm::AlgorithmData::<T>::default()
                .with_ignore_end_point_intersections(true)?
                .with_stop_at_first_intersection(true)?
                .with_lines(self.lines())?
                .compute()?
                .next()
                .is_some())
        }
    }

    /// Returns an iterator containing the found intersections.
    /// The 'ignore_end_point_intersections' parameter must always be set to true when testing
    /// LineStrings.
    /// ```
    /// # use intersect2d::SelfIntersectingExclusive;
    /// # use intersect2d::ulps_eq_c;
    ///
    /// let line_string = geo::LineString::from(vec![
    ///     (100., 100.),
    ///     (200., 100.),
    ///     (200., 200.),
    ///     (100., 200.),
    ///     (100., 100.),
    /// ]);
    /// let rv :Vec<(geo::Coordinate<_>,Vec<usize>)> =
    ///     line_string.self_intersections().expect("err").collect();
    /// assert!(rv.is_empty());
    ///
    /// let line_string = geo::LineString::from(vec![
    ///    (100., 100.),
    ///    (200., 100.),
    ///    (200., 200.),
    ///    (150., 50.),
    ///    (100., 200.),
    ///    (100., 100.),
    /// ]);
    /// let rv :Vec<(geo::Coordinate<_>,Vec<usize>)> =
    ///     line_string.self_intersections().expect("err").collect();
    ///
    /// assert_eq!(line_string.0.len(),6);
    /// assert_eq!(rv.len(), 2);
    /// assert_eq!(rv[0].1, vec!(0_usize,2));
    /// assert!(ulps_eq_c(&rv[0].0, &geo::Coordinate{x: 166.66666666666666, y: 100.0}));
    /// assert_eq!(rv[1].1, vec!(0_usize,3));
    /// assert!(ulps_eq_c(&rv[1].0, &geo::Coordinate{x: 133.33333333333334, y: 100.0}));
    /// ```
    #[allow(clippy::type_complexity)]
    fn self_intersections<'a>(
        &self,
    ) -> Result<
        Box<dyn ExactSizeIterator<Item = (geo::Coordinate<T>, Vec<usize>)> + 'a>,
        IntersectError,
    >
    where
        T: 'a,
    {
        if self.0.len() < 25 {
            // at around <25 line segments the brute force test is faster
            // sanity check for each line
            for point in self.points_iter() {
                if !point.x().is_finite() || !point.y().is_finite() {
                    return Err(IntersectError::InvalidData(
                        "Can't check for intersections on non-finite data".to_string(),
                    ));
                }
            }
            let mut rv = Vec::<(geo::Coordinate<T>, Vec<usize>)>::new();
            for l1 in self.lines().enumerate() {
                for l2 in self.lines().enumerate().skip(l1.0 + 1) {
                    if ulps_eq_c(&l1.1.start, &l2.1.start)
                        || ulps_eq_c(&l1.1.start, &l2.1.end)
                        || ulps_eq_c(&l1.1.end, &l2.1.start)
                        || ulps_eq_c(&l1.1.end, &l2.1.end)
                    {
                        continue;
                    }
                    if let Some(i) = intersect(&l1.1, &l2.1) {
                        rv.push((i.single(), vec![l1.0, l2.0]));
                    }
                }
            }
            // This will only return intersections between two lines at a single point
            // If more than that are intersecting it will be reported once for each pair.
            // Todo: fix it!
            Ok(Box::new(rv.into_iter()))
        } else {
            // at around >25 line segments the sweep-line algorithm is faster
            algorithm::AlgorithmData::<T>::default()
                .with_ignore_end_point_intersections(true)?
                .with_stop_at_first_intersection(false)?
                .with_lines(self.lines())?
                .compute()
        }
    }
}

/// returns true if the two coordinates are virtually identical
///
#[inline(always)]
pub fn ulps_eq_c<T>(a: &geo::Coordinate<T>, b: &geo::Coordinate<T>) -> bool
where
    T: Float + geo::CoordFloat + approx::AbsDiffEq + approx::UlpsEq,
    T::Epsilon: Copy,
{
    approx::ulps_eq!(&a.x, &b.x) && approx::ulps_eq!(&a.y, &b.y)
}

#[inline(always)]
#[allow(dead_code)]
#[deprecated(since = "0.3.2", note = "please use `approx::ulps_eq!` instead")]
pub fn ulps_eq<T>(a: &T, b: &T) -> bool
where
    T: Float + geo::CoordFloat + approx::AbsDiffEq + approx::UlpsEq,
    T::Epsilon: Copy,
{
    approx::ulps_eq!(a, b)
}
