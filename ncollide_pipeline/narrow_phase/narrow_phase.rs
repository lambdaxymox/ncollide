use std::slice::Iter;
use utils::data::hash_map::Entry;
use utils::data::pair::Pair;
use utils::data::uid_remap::{UidRemap, FastKey};
use geometry::geometry::Contact;
use narrow_phase::{CollisionAlgorithm, ContactSignal, CollisionDetector, ProximitySignal};
use world::CollisionObject;
use math::Point;

/// Trait implemented by the narrow phase manager.
///
/// The narrow phase manager is responsible for creating, updating and generating contact pairs
/// between objects identified by the broad phase.
pub trait NarrowPhase<P: Point, M, T> {
    /// Updates this narrow phase.
    fn update(&mut self,
              objects:          &UidRemap<CollisionObject<P, M, T>>,
              contact_signal:   &mut ContactSignal<T>,
              proximity_signal: &mut ProximitySignal<T>,
              timestamp:        usize);

    /// Called when the broad phase detects that two objects are, or stop to be, in close proximity.
    ///
    /// It is expected not to be called if `self.is_proximity_allowed(...)` return `false` for the
    /// given pair of objects.
    fn handle_proximity(&mut self,
                        contact_signal:   &mut ContactSignal<T>,
                        proximity_signal: &mut ProximitySignal<T>,
                        objects:          &UidRemap<CollisionObject<P, M, T>>,
                        fk1:              &FastKey,
                        fk2:              &FastKey,
                        started:          bool);

    // FIXME: the fact that the return type is imposed is not as generic as it could be.
    /// Returns all the potential contact pairs found during the broad phase, and validated by the
    /// narrow phase.
    fn contact_pairs<'a>(&'a self, objects: &'a UidRemap<CollisionObject<P, M, T>>)
                         -> ContactPairs<'a, P, M, T>;
}

/// Iterator through contact pairs.
pub struct ContactPairs<'a, P: Point + 'a, M: 'a, T: 'a> {
    objects: &'a UidRemap<CollisionObject<P, M, T>>,
    pairs:   Iter<'a, Entry<Pair, Box<CollisionDetector<P, M> + 'static>>>
}

impl<'a, P: 'a + Point, M: 'a, T: 'a> ContactPairs<'a, P, M, T> {
    #[doc(hidden)]
    #[inline]
    pub fn new(objects: &'a UidRemap<CollisionObject<P, M, T>>,
               pairs:   Iter<'a, Entry<Pair, Box<CollisionDetector<P, M> + 'static>>>)
               -> ContactPairs<'a, P, M, T> {
        ContactPairs {
            objects: objects,
            pairs:   pairs
        }
    }

    /// Transforms contact-pairs iterator to an iterator through each individual contact.
    #[inline]
    pub fn contacts(self) -> Contacts<'a, P, M, T> {
        Contacts {
            objects:      self.objects,
            co1:          None,
            co2:          None,
            pairs:        self.pairs,
            collector:    Vec::new(), // FIXME: avoid allocations.
            curr_contact: 0
        }
    }
}

impl<'a, P: Point, M, T> Iterator for ContactPairs<'a, P, M, T> {
    type Item = (&'a CollisionObject<P, M, T>,
                 &'a CollisionObject<P, M, T>,
                 &'a CollisionAlgorithm<P, M>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.pairs.next() {
            Some(p) => {
                let co1 = &self.objects[p.key.first];
                let co2 = &self.objects[p.key.second];

                Some((&co1, &co2, &p.value))
            }
            None => None
        }
    }
}

/// An iterator through contacts.
pub struct Contacts<'a, P: 'a + Point, M: 'a, T: 'a> {
    objects:      &'a UidRemap<CollisionObject<P, M, T>>,
    co1:          Option<&'a CollisionObject<P, M, T>>,
    co2:          Option<&'a CollisionObject<P, M, T>>,
    pairs:        Iter<'a, Entry<Pair, Box<CollisionDetector<P, M>>>>,
    collector:    Vec<Contact<P>>,
    curr_contact: usize
}

impl<'a, P: Point, M, T> Iterator for Contacts<'a, P, M, T> {
    type Item = (&'a CollisionObject<P, M, T>, &'a CollisionObject<P, M, T>, Contact<P>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // FIXME: is there a more efficient way to do this (i-e. avoid using an index)?
        if self.curr_contact < self.collector.len() {
            self.curr_contact = self.curr_contact + 1;

            // FIXME: would be nice to avoid the `clone` and return a reference
            // instead (but what would be its lifetime?).
            Some((self.co1.unwrap(), self.co2.unwrap(), self.collector[self.curr_contact - 1].clone()))
        }
        else {
            self.collector.clear();

            while let Some(p) = self.pairs.next() {
                p.value.colls(&mut self.collector);

                if !self.collector.is_empty() {
                    self.co1 = Some(&self.objects[p.key.first]);
                    self.co2 = Some(&self.objects[p.key.second]);
                    self.curr_contact = 1; // Start at 1 instead of 0 because we will return the first one here.

                    // FIXME: would be nice to avoid the `clone` and return a reference
                    // instead (but what would be its lifetime?).
                    return Some((self.co1.unwrap(), self.co2.unwrap(), self.collector[0].clone()))
                }
            }

            None
        }
    }
}

